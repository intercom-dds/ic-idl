// Copyright 2025 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::print_stdout,
    clippy::print_stderr
)]

use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ic_cli::Command;
use ic_cli::color::Colorize;
use ic_fuzz::{Fuzzer, FuzzerConfig, Grammar};
use ic_preproc::ProcArgs;
use ic_vfs::SourceMap;
use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};

const UPDATE_INTERVAL: Duration = Duration::from_millis(120);

/// Fuzz test the IDL parser
#[derive(Command, Debug, Default)]
#[command]
struct Options {
    /// Path to the JSON grammar file
    #[option(positional)]
    grammar: PathBuf,

    /// Output directory for failed test cases (default: current directory)
    #[option(short, long, arg = "path")]
    output: Option<String>,

    /// Random seed for reproducible generation
    #[option(short, long, arg = "num")]
    seed: Option<u64>,

    /// Number of threads (default: number of CPUs)
    #[option(short = 'j', long, arg = "num")]
    threads: Option<usize>,

    /// Maximum recursion depth
    #[option(long, arg = "num")]
    max_depth: Option<usize>,

    /// Minimum repetitions for * and + modifiers
    #[option(long, arg = "num")]
    min_rep: Option<usize>,

    /// Maximum repetitions for * and + modifiers
    #[option(long, arg = "num")]
    max_rep: Option<usize>,

    /// Maximum tokens to generate
    #[option(long, arg = "num")]
    max_tokens: Option<usize>,

    /// Probability of including optional elements (0.0-1.0)
    #[option(long, arg = "prob")]
    optional_prob: Option<f64>,

    /// Probability of injecting annotations (0.0-1.0)
    #[option(long, arg = "prob")]
    annotation_prob: Option<f64>,
}

fn main() {
    let args = Options::parse();

    let grammar_json = fs::read_to_string(&args.grammar).unwrap_or_else(|e| {
        eprintln!("error: {}: {e}", args.grammar.display());
        std::process::exit(1);
    });

    let grammar = Arc::new(Grammar::from_json(&grammar_json).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    }));

    let output_dir = Arc::new(PathBuf::from(args.output.as_deref().unwrap_or(".")));
    fs::create_dir_all(output_dir.as_ref()).unwrap_or_else(|e| {
        eprintln!("error: failed to create output directory: {e}");
        std::process::exit(1);
    });

    let config = FuzzerConfig {
        max_depth: args.max_depth.unwrap_or(8),
        min_repetitions: args.min_rep.unwrap_or(1),
        max_repetitions: args.max_rep.unwrap_or(5),
        max_tokens: args.max_tokens,
        annotation_probability: args.annotation_prob,
        optional_probability: args.optional_prob.unwrap_or(0.7),
        seed: args.seed,
    };

    let base_seed = args.seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64)
    });

    let num_threads = args
        .threads
        .unwrap_or_else(|| thread::available_parallelism().map_or(4, std::num::NonZero::get));

    let total_passed = Counter::new();
    let total_failed = Counter::new();
    let mp = MultiProgress::new();

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            spawn_worker(
                WorkerContext {
                    grammar: grammar.clone(),
                    config: config.clone(),
                    output_dir: output_dir.clone(),
                    total_passed: total_passed.clone(),
                    total_failed: total_failed.clone(),
                    mp: mp.clone(),
                    base_seed,
                },
                thread_id,
            )
        })
        .collect();

    let summary_style = summary_progress_style(total_passed, total_failed);
    let summary_pb = mp.add(ProgressBar::new_spinner().with_style(summary_style));
    summary_pb.enable_steady_tick(UPDATE_INTERVAL / 2);

    for handle in handles {
        let _ = handle.join();
    }
}

#[derive(Clone)]
struct Counter(Arc<AtomicU64>);

impl Counter {
    fn new() -> Self {
        Self(Arc::new(AtomicU64::new(0)))
    }

    fn inc(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    fn add(&self, val: u64) {
        self.0.fetch_add(val, Ordering::Relaxed);
    }

    fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

fn format_tokens_per_sec(tokens_per_sec: f64) -> String {
    if tokens_per_sec >= 1_000_000.0 {
        format!("{:.2}M tok", tokens_per_sec / 1_000_000.0)
    } else if tokens_per_sec >= 1_000.0 {
        format!("{:.1}K tok", tokens_per_sec / 1_000.0)
    } else {
        format!("{tokens_per_sec:.0} tok")
    }
}

fn thread_progress_style(thread_id: usize, tokens: Counter, start: Instant) -> ProgressStyle {
    ProgressStyle::default_spinner()
        .tick_strings(&["·", "✢", "*", "∗", "✻", "✽", "✻", "∗", "*", "✢", "·"])
        .template("  {spinner:.red} thread {thread_id}: {rate}/s")
        .unwrap()
        .with_key(
            "thread_id",
            move |_: &ProgressState, w: &mut dyn fmt::Write| {
                let _ = write!(w, "{thread_id:>2}");
            },
        )
        .with_key("rate", move |_: &ProgressState, w: &mut dyn fmt::Write| {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = if elapsed > 0.0 {
                tokens.get() as f64 / elapsed
            } else {
                0.0
            };
            let _ = write!(w, "{}", format_tokens_per_sec(rate));
        })
}

fn summary_progress_style(passed: Counter, failed: Counter) -> ProgressStyle {
    ProgressStyle::default_spinner()
        .tick_strings(&[
            "010010", "001100", "100101", "111010", "111101", "010111", "101011", "111000",
            "110011", "110101",
        ])
        .template(&format!(
            "{{spinner:.red}} {{total}} {} | {{passed_count}} {} | {{failed_count}} {}",
            "tests".yellow(),
            "passed".green(),
            "failed".red(),
        ))
        .unwrap()
        .with_key("total", {
            let passed = passed.clone();
            let failed = failed.clone();
            move |_: &ProgressState, w: &mut dyn fmt::Write| {
                let _ = write!(w, "{}", passed.get() + failed.get());
            }
        })
        .with_key(
            "passed_count",
            move |_: &ProgressState, w: &mut dyn fmt::Write| {
                let _ = write!(w, "{}", passed.get());
            },
        )
        .with_key(
            "failed_count",
            move |_: &ProgressState, w: &mut dyn fmt::Write| {
                let _ = write!(w, "{}", failed.get());
            },
        )
}

struct WorkerContext {
    grammar: Arc<Grammar>,
    config: FuzzerConfig,
    output_dir: Arc<PathBuf>,
    total_passed: Counter,
    total_failed: Counter,
    mp: MultiProgress,
    base_seed: u64,
}

fn spawn_worker(ctx: WorkerContext, thread_id: usize) -> JoinHandle<()> {
    let thread_seed = ctx.base_seed.wrapping_add((thread_id as u64) << 48);
    let thread_tokens = Counter::new();
    let thread_start = Instant::now();

    let style = thread_progress_style(thread_id, thread_tokens.clone(), thread_start);
    let pb = ctx.mp.add(ProgressBar::new_spinner().with_style(style));
    pb.tick();

    thread::spawn(move || {
        let mut fuzzer = Fuzzer::new(ctx.grammar.clone(), ctx.config);
        let mut i: u64 = 0;
        let mut last_tick = Instant::now();

        loop {
            let seed = thread_seed.wrapping_add(i);
            let generated = fuzzer.generate_with_seed(seed);

            let mut source_map = SourceMap::default();
            let file_id = source_map.embed_with_name("<fuzz>", generated.source.as_str());
            let result = ic_parse::from_file(file_id, ProcArgs::default(), &mut source_map);

            thread_tokens.add(generated.token_count as u64);

            if result.errors.is_empty() {
                ctx.total_passed.inc();
            } else {
                ctx.total_failed.inc();
                save_failure(&ctx.output_dir, seed, &generated.source)
                    .expect("failed to save failure");
                _ = ctx.mp.println(format!(
                    "{} {} error(s) -> {}/{}.idl",
                    "FAILED".red(),
                    result.errors.len(),
                    ctx.output_dir.display(),
                    seed
                ));
            }

            i += 1;
            if last_tick.elapsed() >= UPDATE_INTERVAL {
                pb.tick();
                last_tick = Instant::now();
            }
        }
    })
}

fn save_failure(output_dir: &std::path::Path, seed: u64, idl: &str) -> std::io::Result<()> {
    let path = output_dir.join(format!("{seed}.idl"));
    let mut file = File::create(&path)?;
    writeln!(file, "// @generated by ic-fuzz")?;
    writeln!(file, "// seed: {seed}")?;
    writeln!(file)?;
    write!(file, "{idl}")?;
    Ok(())
}
