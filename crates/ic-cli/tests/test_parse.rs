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

use ic_cli::{Category, CommandLine, Opt, ParseError, Value};

#[test]
fn test_parse_short_flag() {
    let opt: Opt = "v".into();
    let cmd = CommandLine::new("myapp").opt(opt);

    let result = cmd.parse_args(vec!["-v".to_string()].into_iter());
    assert!(result.is_present("v"));
    assert_eq!(result.count("v"), 1);
}

#[test]
fn test_parse_long_flag() {
    let opt: Opt = "verbose".into();
    let cmd = CommandLine::new("myapp").opt(opt);

    let result = cmd.parse_args(vec!["--verbose".to_string()].into_iter());
    assert!(result.is_present("verbose"));
}

#[test]
fn test_parse_short_with_value() {
    let opt: Opt = "o".into();
    let opt = opt.value(Value::Single, "FILE");
    let cmd = CommandLine::new("myapp").opt(opt);

    let result = cmd.parse_args(vec!["-o".to_string(), "output.txt".to_string()].into_iter());
    assert_eq!(
        result.get("o").map(std::string::String::as_str),
        Some("output.txt")
    );
}

#[test]
fn test_parse_long_with_equals() {
    let opt: Opt = "output".into();
    let opt = opt.value(Value::Single, "FILE");
    let cmd = CommandLine::new("myapp").opt(opt);

    let result = cmd.parse_args(vec!["--output=output.txt".to_string()].into_iter());
    assert_eq!(
        result.get("output").map(std::string::String::as_str),
        Some("output.txt")
    );
}

#[test]
fn test_parse_multiple_flags() {
    let opt1: Opt = "v".into();
    let opt2: Opt = "d".into();
    let cmd = CommandLine::new("myapp").opt(opt1).opt(opt2);

    let result = cmd.parse_args(vec!["-v".to_string(), "-d".to_string()].into_iter());
    assert!(result.is_present("v"));
    assert!(result.is_present("d"));
}

#[test]
fn test_parse_positionals() {
    let cmd = CommandLine::new("myapp").positionals(true);

    let result = cmd.parse_args(vec!["file1".to_string(), "file2".to_string()].into_iter());
    let positionals = result.positionals();
    assert_eq!(positionals.len(), 2);
    assert_eq!(positionals[0], "file1");
    assert_eq!(positionals[1], "file2");
}

#[test]
fn test_parse_with_double_dash() {
    let opt: Opt = "v".into();
    let cmd = CommandLine::new("myapp").opt(opt).positionals(true);

    let result = cmd.parse_args(
        vec![
            "-v".to_string(),
            "--".to_string(),
            "-v".to_string(),
            "file".to_string(),
        ]
        .into_iter(),
    );

    assert!(result.is_present("v"));
    let positionals = result.positionals();
    assert_eq!(positionals.len(), 2);
    assert_eq!(positionals[0], "-v"); // Should be treated as positional after --
    assert_eq!(positionals[1], "file");
}

#[test]
fn test_parse_help_flag() {
    let cmd1 = CommandLine::new("myapp");
    let result = cmd1.try_parse_args(vec!["--help".to_string()].into_iter());
    assert!(matches!(result, Err(ParseError::Help(_))));

    let cmd2 = CommandLine::new("myapp");
    let result = cmd2.try_parse_args(vec!["-h".to_string()].into_iter());
    assert!(matches!(result, Err(ParseError::Help(_))));
}

#[test]
fn test_parse_subcommand() {
    let sub_opt: Opt = "v".into();
    let subcommand = CommandLine::new("sub").opt(sub_opt);

    let category = Category::with_commands("Commands", vec![subcommand]);
    let cmd = CommandLine::new("myapp").category(category);

    let result = cmd.parse_args(vec!["sub".to_string(), "-v".to_string()].into_iter());
    assert!(result.subcommand().is_some());

    let sub_result = result.subcommand().unwrap();
    assert_eq!(sub_result.name(), "sub");
    assert!(sub_result.is_present("v"));
}

#[test]
fn test_parse_count() {
    let opt: Opt = "v".into();
    let cmd = CommandLine::new("myapp").opt(opt);

    let result =
        cmd.parse_args(vec!["-v".to_string(), "-v".to_string(), "-v".to_string()].into_iter());
    assert_eq!(result.count("v"), 3);
}

#[test]
fn test_parse_external_command() {
    let cmd = CommandLine::new("myapp").external(true);

    let result = cmd.parse_args(vec!["external".to_string(), "-unknown".to_string()].into_iter());
    let positionals = result.positionals();
    assert_eq!(positionals.len(), 2);
    assert_eq!(positionals[0], "external");
    assert_eq!(positionals[1], "-unknown");
}

#[test]
fn test_parse_error_unexpected_value() {
    let cmd = CommandLine::new("myapp").positionals(false); // No positionals allowed

    let result = cmd.try_parse_args(vec!["unexpected".to_string()].into_iter());
    assert!(matches!(result, Err(ParseError::Status(_))));
}

#[test]
fn test_parse_error_missing_value() {
    let opt: Opt = "o".into();
    let opt = opt.value(Value::Single, "FILE");
    let cmd = CommandLine::new("myapp").opt(opt);

    let result = cmd.try_parse_args(vec!["-o".to_string()].into_iter());
    assert!(result.is_err());
}

#[test]
fn test_parse_bool_flag_negation() {
    let opt: Opt = "enable".into();
    let cmd = CommandLine::new("myapp").opt(opt);

    let result = cmd.parse_args(vec!["--enable=false".to_string()].into_iter());
    assert!(result.is_present("enable"));
    assert_eq!(
        result.get("enable").map(std::string::String::as_str),
        Some("false")
    );
}

#[test]
fn test_parse_unknown_option() {
    let cmd = CommandLine::new("myapp");

    let result = cmd.try_parse_args(vec!["--unknown".to_string()].into_iter());
    assert!(result.is_err());
}

#[test]
fn test_result_name() {
    let cmd = CommandLine::new("myapp");
    let result = cmd.parse_args(Vec::<String>::new().into_iter());
    assert_eq!(result.name(), "myapp");
}
