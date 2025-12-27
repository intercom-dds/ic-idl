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

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use ic_cli::Command;
use ic_fuzz::{Fuzzer, FuzzerConfig};

/// Generate random IDL from a JSON grammar specification
#[derive(Command, Debug, Default)]
struct Options {
    /// Path to the JSON grammar file
    #[option(positional)]
    grammar: PathBuf,

    /// Output file (stdout if not specified)
    #[option(short, long, arg = "path")]
    output: Option<String>,

    /// Random seed for reproducible generation
    #[option(short, long, arg = "num")]
    seed: Option<u64>,

    /// Number of IDL samples to generate
    #[option(short = 'n', long, arg = "num")]
    count: Option<usize>,

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

    // Read the grammar file
    let grammar_json = match fs::read_to_string(&args.grammar) {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "error: failed to read grammar file {}: {e}",
                args.grammar.display(),
            );
            std::process::exit(1);
        }
    };

    let config = FuzzerConfig {
        max_depth: args.max_depth.unwrap_or(8),
        min_repetitions: args.min_rep.unwrap_or(1),
        max_repetitions: args.max_rep.unwrap_or(5),
        max_tokens: args.max_tokens,
        annotation_probability: args.annotation_prob,
        optional_probability: args.optional_prob.unwrap_or(0.7),
        seed: args.seed,
    };

    // Create the fuzzer
    let mut fuzzer = match Fuzzer::from_json(&grammar_json, config) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: failed to parse grammar: {e}");
            std::process::exit(1);
        }
    };

    // Generate the output
    let count = args.count.unwrap_or(1);
    let mut output: Box<dyn Write> = match &args.output {
        Some(path) => match fs::File::create(path) {
            Ok(f) => Box::new(f),
            Err(e) => {
                eprintln!("error: failed to create output file {path:?}: {e}");
                std::process::exit(1);
            }
        },
        None => Box::new(io::stdout()),
    };

    for i in 0..count {
        let result = if let Some(base_seed) = args.seed {
            fuzzer.generate_with_seed(base_seed.wrapping_add(i as u64))
        } else {
            fuzzer.generate()
        };

        writeln!(output, "{}", result.source).ok();
    }
}
