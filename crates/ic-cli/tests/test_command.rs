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
fn test_negated_flag() {
    #[derive(Default, Command)]
    struct Foo {
        #[option(long)]
        foo: bool,
    }

    let parsed = Foo::from_args(args!["--foo=false"]);
    assert!(!parsed.foo);

    let parsed = Foo::from_args(args!["--foo=true"]);
    assert!(parsed.foo);
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
fn test_short_option_no_space() {
    #[derive(Default, Command)]
    struct Foo {
        #[option(short = 'I')]
        include: String,
    }

    let parsed = Foo::from_args(args!["-I."]);
    assert_eq!(parsed.include, ".");
}

#[test]
fn test_long_equals() {
    #[derive(Default, Command)]
    struct Foo {
        #[option(long)]
        value: Vec<String>,
    }

    let parsed = Foo::from_args(args!["--value=1,2,3"]);
    assert_eq!(parsed.value.len(), 3);
    assert_eq!(parsed.value, vec!["1", "2", "3"]);
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

#[test]
fn test_enum_external_fallback() {
    #[derive(Default, Command)]
    struct Foo {
        #[option(long)]
        value: bool,
    }

    #[derive(Command)]
    enum App {
        Foo(Foo),
        #[command(external)]
        External(ic_cli::ParseResult),
    }

    let mut cmd = App::command();
    cmd = cmd.category(ic_cli::Category {
        name: "plugins",
        commands: vec![ic_cli::CommandLine::new("deploy").external(true)],
    });

    let result = cmd.parse_args(args!["deploy", "--foo", "bar", "-x"].into_iter());
    let App::External(r) = App::from_result(&result) else {
        panic!("expected external variant");
    };
    assert_eq!(r.name(), "deploy");
    assert_eq!(r.positionals(), &args!["--foo", "bar", "-x"]);

    let result = App::command().parse_args(args!["foo", "--value"].into_iter());
    let App::Foo(parsed) = App::from_result(&result) else {
        panic!("expected built-in variant");
    };
    assert!(parsed.value);
}

#[test]
fn test_enum_categories() {
    #[derive(Default, Command)]
    struct Empty {}

    #[derive(Command)]
    enum App {
        Init(Empty),
        #[command(category = "building")]
        Bundle(Empty),
        Clean(Empty),
        #[command(category = "testing")]
        Check(Empty),
        Bench(Empty),
    }

    let help = App::command().help();
    let commands = help.find("commands:").expect("default category");
    let building = help.find("building:").expect("building category");
    let testing = help.find("testing:").expect("testing category");
    assert!(commands < building && building < testing);

    let clean = help.find("clean").expect("clean command");
    assert!(clean > building && clean < testing);

    let result = App::command().parse_args(args!["bench"].into_iter());
    assert!(matches!(App::from_result(&result), App::Bench(_)));
}
