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
use ic_cli::color::{ColorMode, set_color_override};

macro_rules! args {
    ($($arg:literal),* $(,)?) => {{
        vec![$($arg.to_string(),)*]
    }};
}

#[derive(Default, Command)]
struct Shared {
    /// Alpha flag
    #[option(long)]
    alpha: bool,

    /// Beta value
    #[option(long, arg = "n")]
    beta: usize,
}

#[derive(Default, Command)]
struct Deep {
    /// Gamma flag
    #[option(long)]
    gamma: bool,
}

#[derive(Default, Command)]
struct Inner {
    #[option(section = "inner options")]
    deep: Deep,
}

#[derive(Default, Command)]
struct App {
    /// First flag
    #[option(long)]
    first: bool,

    #[option(flatten)]
    shared: Shared,

    #[option(flatten)]
    inner: Inner,

    /// Last flag
    #[option(long)]
    last: bool,
}

#[derive(Default, Command)]
struct Other {
    /// Other flag
    #[option(long)]
    other: bool,

    #[option(section = "shared options")]
    shared: Shared,
}

fn help<T: Command>() -> String {
    set_color_override(ColorMode::Never);
    T::command().help()
}

fn headers(help: &str) -> Vec<&str> {
    help.lines()
        .filter(|line| !line.starts_with(' ') && line.ends_with(':'))
        .collect()
}

fn section_options<'a>(help: &'a str, header: &str) -> Vec<&'a str> {
    help.lines()
        .skip_while(|line| *line != header)
        .skip(1)
        .take_while(|line| !line.is_empty())
        .filter_map(|line| line.split_whitespace().next())
        .filter(|token| token.starts_with("--"))
        .collect()
}

#[test]
fn flatten_order() {
    let help = help::<App>();

    assert_eq!(
        section_options(&help, "options:"),
        ["--first", "--alpha", "--beta", "--last"],
        "{help}"
    );
}

#[test]
fn flatten_section() {
    let help = help::<App>();

    assert_eq!(
        headers(&help),
        ["usage:", "options:", "inner options:"],
        "{help}"
    );

    assert_eq!(
        section_options(&help, "inner options:"),
        ["--gamma"],
        "{help}"
    );
}

#[test]
fn flatten_from_result() {
    let parsed = App::from_args(args!["--first", "--alpha", "--beta", "3", "--gamma"]);

    assert!(parsed.first);
    assert!(parsed.shared.alpha);
    assert_eq!(parsed.shared.beta, 3);
    assert!(parsed.inner.deep.gamma);
    assert!(!parsed.last);
}

#[test]
fn flatten_from_result_mut() {
    let mut opts = App {
        last: true,
        shared: Shared {
            alpha: true,
            beta: 7,
        },
        ..Default::default()
    };

    App::from_args_mut(args!["--beta", "9", "--gamma"], &mut opts);

    assert!(opts.last);
    assert!(opts.shared.alpha);
    assert_eq!(opts.shared.beta, 9);
    assert!(opts.inner.deep.gamma);
    assert!(!opts.first);
}

#[test]
fn flatten_and_section() {
    let app = help::<App>();
    let other = help::<Other>();

    assert_eq!(
        section_options(&app, "options:"),
        ["--first", "--alpha", "--beta", "--last"],
        "{app}"
    );

    assert_eq!(section_options(&other, "options:"), ["--other"], "{other}");

    assert_eq!(
        section_options(&other, "shared options:"),
        ["--alpha", "--beta"],
        "{other}"
    );

    let app = App::from_args(args!["--alpha", "--beta", "1"]);
    let other = Other::from_args(args!["--alpha", "--beta", "2"]);

    assert!(app.shared.alpha);
    assert_eq!(app.shared.beta, 1);
    assert!(other.shared.alpha);
    assert_eq!(other.shared.beta, 2);
}
