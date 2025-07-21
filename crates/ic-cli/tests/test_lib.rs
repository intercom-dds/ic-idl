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

use ic_cli::{CommandLine, Opt, Value};

#[test]
fn test_command_line_builder() {
    let cmd = CommandLine::new("myapp")
        .desc("My application")
        .version("1.0.0");

    // Test help output contains description and version
    let help = cmd.help();
    // Basic check that help was generated
    assert!(!help.is_empty());
}

#[test]
fn test_opt_from_str() {
    let opt1: Opt = "-v".into();
    let opt2: Opt = ["-h", "--help"].into();
    let opt3: Opt = ["-o", "--output", "-O"].into();

    // Options should be created with proper defaults
    let cmd = CommandLine::new("myapp").opt(opt1).opt(opt2).opt(opt3);

    let help = cmd.help();
    // Just check that help contains something
    assert!(!help.is_empty());
}

#[test]
fn test_opt_builder() {
    let opt: Opt = ["-o", "--output"].into();
    let opt = opt
        .desc("Output file")
        .value(Value::Single, "FILE")
        .required(true);

    let cmd = CommandLine::new("myapp").opt(opt);
    let help = cmd.help();
    assert!(!help.is_empty());
}

#[test]
fn test_multiple_value_opt() {
    let opt: Opt = ["-i", "--include"].into();
    let opt = opt.desc("Include paths").value(Value::Multiple, "PATH");

    let cmd = CommandLine::new("myapp").opt(opt);
    let help = cmd.help();
    assert!(!help.is_empty());
}

#[test]
fn test_section_command() {
    let opt_a: Opt = "-a".into();
    let opt_a = opt_a.desc("Option A");

    let opt_b: Opt = "-b".into();
    let opt_b = opt_b.desc("Option B");

    let section_cmd = CommandLine::new("section").opt(opt_a).opt(opt_b);

    let main_cmd = CommandLine::new("myapp").section("Advanced Options", section_cmd);

    let help = main_cmd.help();
    assert!(!help.is_empty());
}

#[test]
fn test_merge_command() {
    let opt_a: Opt = "-a".into();
    let opt_b: Opt = "-b".into();

    let cmd1 = CommandLine::new("cmd1").opt(opt_a);
    let cmd2 = CommandLine::new("cmd2").opt(opt_b);

    let merged = cmd1.merge(cmd2);

    let help = merged.help();
    assert!(!help.is_empty());
}

#[test]
fn test_positionals() {
    let cmd = CommandLine::new("myapp")
        .positionals(true)
        .arg_name("FILES");

    // Test creates a command with positionals and arg_name
    let help = cmd.help();
    assert!(!help.is_empty());
}

#[test]
fn test_after_help() {
    let after_help_text = "See 'myapp help <command>' for more information on a specific command.";
    let cmd = CommandLine::new("myapp").after_help(after_help_text);

    let help = cmd.help();
    assert!(!help.is_empty());
}

#[test]
fn test_command_line_flags() {
    let cmd = CommandLine::new("myapp")
        .split_flags(false)
        .hide_flags(true, true)
        .align_sections(true)
        .external(true);

    // These affect help formatting
    let _help = cmd.help();
}

#[test]
fn test_command_line_parsing() {
    let opt: Opt = ["v", "verbose"].into();
    let opt = opt.desc("Enable verbose output");

    let cmd = CommandLine::new("myapp").opt(opt);

    // Test successful parse
    let result = cmd.parse_args(vec!["--verbose".to_string()].into_iter());
    assert!(result.is_present("verbose"));
}

#[test]
fn test_parse_with_value() {
    let opt: Opt = ["o", "output"].into();
    let opt = opt.desc("Output file").value(Value::Single, "FILE");

    let cmd = CommandLine::new("myapp").opt(opt);

    let result = cmd.parse_args(vec!["--output".to_string(), "file.txt".to_string()].into_iter());
    assert!(result.is_present("output"));
    assert_eq!(
        result.get("output").map(std::string::String::as_str),
        Some("file.txt")
    );
}

#[test]
fn test_parse_multiple_values() {
    let opt: Opt = ["i", "include"].into();
    let opt = opt.desc("Include paths").value(Value::Multiple, "PATH");

    let cmd = CommandLine::new("myapp").opt(opt);

    let result = cmd.parse_args(
        vec![
            "--include".to_string(),
            "path1".to_string(),
            "--include".to_string(),
            "path2".to_string(),
        ]
        .into_iter(),
    );

    assert!(result.is_present("include"));
    let values = result.get_vec("include");
    assert!(values.is_some());
    let values = values.unwrap();
    assert_eq!(values.len(), 2);
    assert!(values.contains(&"path1".to_string()));
    assert!(values.contains(&"path2".to_string()));
}

#[test]
fn test_get_name() {
    let cmd = CommandLine::new("myapp").name("renamed");
    assert_eq!(cmd.get_name(), "renamed");
}
