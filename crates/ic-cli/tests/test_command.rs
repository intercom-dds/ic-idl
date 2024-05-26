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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
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

macro_rules! args {
    ($($arg:literal),* $(,)?) => {{
        vec![$($arg.to_string(),)*]
    }};
}

#[test]
fn test_flags() {
    #[derive(Default, Command)]
    struct Foo {
        #[option(short, long)]
        foo: bool,

        #[option(short, long)]
        bar: bool,
    }

    let parsed = Foo::from_args(args!["-f", "-b"]);
    assert!(parsed.foo);
    assert!(parsed.bar);

    let parsed = Foo::from_args(args!["--foo", "--bar"]);
    assert!(parsed.foo);
    assert!(parsed.bar);

    let parsed = Foo::from_args(args!["-b"]);
    assert!(!parsed.foo);
    assert!(parsed.bar);
}

#[test]
fn test_options() {
    #[derive(Default, Command)]
    struct Foo {
        #[option(short, long)]
        string: String,

        #[option(short, long)]
        number: usize,
    }

    let parsed = Foo::from_args(args!["-s", "abc", "-n", "123"]);
    assert_eq!(parsed.string, "abc");
    assert_eq!(parsed.number, 123);

    let parsed = Foo::from_args(args!["--string", "abc", "--number", "123"]);
    assert_eq!(parsed.string, "abc");
    assert_eq!(parsed.number, 123);
}

#[test]
fn test_positionals() {
    #[derive(Default, Command)]
    struct Foo {
        #[option(positional)]
        args: Vec<String>,
    }

    let parsed = Foo::from_args(args!["foo", "bar", "baz"]);
    assert!(parsed.args.contains(&"foo".to_string()));
    assert!(parsed.args.contains(&"bar".to_string()));
    assert!(parsed.args.contains(&"baz".to_string()));
}

#[test]
fn test_replaced_char() {
    #[derive(Default, Command)]
    struct Foo {
        #[option(long)]
        my_opt_var: bool,
    }

    let parsed = Foo::from_args(args!["--my-opt-var"]);
    assert!(parsed.my_opt_var);
}

#[test]
fn test_enum() {
    #[derive(Default, Command)]
    struct Foo {
        #[option(long)]
        value: bool,
    }

    #[derive(Command)]
    enum Command {
        FooBarBaz(Foo),
    }

    let Command::FooBarBaz(parsed) = Command::from_args(args!["foo-bar-baz", "--value"]);
    assert!(parsed.value);
}
