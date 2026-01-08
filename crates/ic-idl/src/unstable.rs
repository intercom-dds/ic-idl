// Copyright 2024 KONGSBERG
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

use ic_cli::Command;
use ic_cli::color::Colorize as _;
use ic_idl::{Unstable, Warnings};

pub fn unstable_help() {
    let command = Unstable::command();
    let flags = command.format_args_prefix("-Z ", |_| true).join("\n");

    println!("{}", "unstable flags:".yellow());
    println!("{flags}");
    println!("\nRun with `{}`\n", "ic-idl -Z [FLAG] <files>...".green());
    println!(
        "{} unstable flags may change at any time in backward-incompatible ways",
        "warning:".yellow(),
    );
}

pub fn warning_help() {
    let command = Warnings::command();
    let flags = command.format_args_prefix("-W", |_| true).join("\n");

    println!("{}", "warning groups:".yellow());
    println!("{flags}");

    println!("\n{}", "available lints:".yellow());
    let mut lints = ic_lint::all_lints();
    // Filter out semantic and syntax lints since they can't be disabled
    lints.retain(|lint| {
        !matches!(
            lint.category,
            ic_lint::Category::Semantic | ic_lint::Category::Syntax
        )
    });
    lints.sort_by(|a, b| {
        let cat_cmp = format!("{:?}", a.category).cmp(&format!("{:?}", b.category));
        if cat_cmp == std::cmp::Ordering::Equal {
            a.name.cmp(b.name)
        } else {
            cat_cmp
        }
    });

    let max_name_len = lints.iter().map(|l| l.name.len() + 2).max().unwrap_or(0);
    let mut current_category = None;

    for lint in lints {
        if current_category != Some(lint.category) {
            if current_category.is_some() {
                println!();
            }
            println!("  {}:", lint.category.cyan());
            current_category = Some(lint.category);
        }

        println!(
            "    {:<width$}  {}",
            lint.name,
            lint.description,
            width = max_name_len
        );
    }

    println!("\nRun with `{}`\n", "ic-idl -W [WARN] <files>...".green());
    println!(
        "To disable a warning, add '{}' before the warning text (e.g. '{}')",
        "no-".yellow(),
        "-Wno-all".yellow(),
    );
}
