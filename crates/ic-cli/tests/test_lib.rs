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
