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

#![allow(clippy::struct_excessive_bools)]

use std::env;

use color::Colorize;
pub use ic_cli_derive::Command;
pub mod color;
pub mod convert;

mod index;
use index::IndexMap;

mod parse;
pub use parse::{ParseError, ParseResult};

const PAD: usize = 3;

#[must_use]
#[derive(Default)]
pub struct CommandLine {
    name: String,
    desc: Option<String>,
    version: Option<String>,
    options: IndexMap<String, Opt>,
    section: Option<String>,
    hide_flags: bool,
    hide_options: bool,
    arg_name: Option<String>,
    after_help: Option<String>,
    positionals: bool,
    external: bool,
    parent: Option<String>,
    subcommands: IndexMap<String, Vec<CommandLine>>,
}

impl CommandLine {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            desc: None,
            version: None,
            options: IndexMap::new(),
            section: None,
            hide_flags: false,
            hide_options: false,
            arg_name: None,
            after_help: None,
            external: false,
            positionals: false,
            parent: None,
            subcommands: IndexMap::new(),
        }
    }

    pub fn desc(mut self, desc: impl Into<String>) -> Self {
        self.desc = Some(desc.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn section(mut self, section: impl Into<String>) -> Self {
        self.section = Some(section.into());
        self
    }

    pub fn opt(mut self, option: Opt) -> Self {
        for token in &option.tokens {
            assert!(
                !self.options.contains_key(token),
                "duplicate registration of option {token}"
            );
        }
        self.options.insert(option.tokens.clone(), option);
        self
    }

    pub fn opts<I>(mut self, options: I) -> Self
    where
        I: IntoIterator<Item = Opt>,
    {
        for opt in options {
            self = self.opt(opt);
        }
        self
    }

    pub fn subcommand(self, command: CommandLine) -> Self {
        drop(command);
        todo!();
    }

    pub fn category(mut self, category: Category) -> Self {
        let commands = category
            .commands
            .into_iter()
            .map(|mut v| {
                v.parent = Some(self.name.clone());
                v.default_opts()
            })
            .collect();

        self.subcommands
            .insert(vec![category.name.to_string()], commands);
        self
    }

    pub fn positionals(mut self, takes_positionals: bool) -> Self {
        self.positionals = takes_positionals;
        self
    }

    pub fn external(mut self, external: bool) -> Self {
        self.external = external;
        self
    }

    pub fn hide_flags(mut self, hide_flags: bool, hide_options: bool) -> Self {
        self.hide_flags = hide_flags;
        self.hide_options = hide_options;
        self
    }

    pub fn arg_name(mut self, name: impl Into<String>) -> Self {
        self.arg_name = Some(name.into());
        self
    }

    pub fn after_help(mut self, desc: impl Into<String>) -> Self {
        self.after_help = Some(desc.into());
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn try_parse(self) -> Result<ParseResult, ParseError> {
        parse::from_env(&mut self.default_opts())
    }

    pub fn parse(self) -> ParseResult {
        self.parse_args(std::env::args().skip(1))
    }

    pub fn parse_args<I>(mut self, iter: I) -> ParseResult
    where
        I: Iterator<Item = String>,
    {
        self = self.default_opts();
        let result = match parse::from_args(iter, &mut self) {
            Ok(v) => v,
            Err(e) => {
                match e {
                    ParseError::Help(msg) => println!("{msg}"),
                    ParseError::Status(msg) => {
                        let error = "error:".red();
                        eprintln!("{error} {msg}");
                    }
                };
                std::process::exit(1);
            }
        };

        if !self.subcommands.is_empty() && result.subcommand().is_none() {
            println!("{}", self.help());
            std::process::exit(0);
        }
        Self::validate(&result);
        result
    }

    pub fn help(&self) -> String {
        let mut lines: Vec<String> = vec![];

        let version = self.version.clone().unwrap_or_default();
        lines.push(format!(
            "{} {version}",
            self.qualified_name('-', false).green()
        ));
        if let Some(desc) = &self.desc {
            lines.push(format!("\n{desc}"));
        }

        lines.push("\nusage:".yellow());
        let mut usage = format!("{:PAD$}{}", " ", self.qualified_name(' ', true));
        if !self.subcommands.is_empty() {
            usage = format!("{usage} [command]");
        }
        if !self.options.is_empty() {
            usage = format!("{usage} <options>");
        }
        if self.positionals {
            usage = format!("{usage} <files>...");
        }
        lines.push(usage);

        if !self.hide_flags {
            let flags = self.format_args(|v| v.kind == Value::Flag);
            if !flags.is_empty() {
                lines.push("\nflags:".yellow());
                lines.extend(flags);
            }
        }

        if !self.hide_options {
            let options = self.format_args(|v| v.kind != Value::Flag);
            if !options.is_empty() {
                lines.push("\noptions:".yellow());
                lines.extend(options);
            }
        }

        if !self.subcommands.is_empty() {
            lines.extend(self.format_commands());
        }

        if let Some(after) = &self.after_help {
            lines.push(format!("\n{after}"));
        }
        lines.join("\n")
    }

    fn qualified_name(&self, sep: char, exe: bool) -> String {
        let mut name = String::new();
        if let Some(p) = &self.parent {
            if exe {
                name.push_str(&self.exe_name());
            } else {
                name.push_str(p);
            }
            name.push(sep);
        }
        name.push_str(self.name());
        name
    }

    fn exe_name(&self) -> String {
        if let Ok(exe) = env::current_exe() {
            if let Some(stem) = exe.file_stem() {
                return stem.to_string_lossy().to_string();
            }
        }
        self.name.to_string()
    }

    pub fn format_args<P>(&self, filter: P) -> Vec<String>
    where
        P: FnMut(&&Opt) -> bool + Clone,
    {
        let mut lines = vec![];
        let matches: Vec<_> = self.options.values().iter().filter(filter).collect();

        // Find the highest number of short options
        let n_short: usize = matches
            .iter()
            .map(|v| v.tokens.iter().filter(|v| v.len() == 1).count())
            .max()
            .unwrap_or(0);

        let width = matches
            .iter()
            .map(|v| {
                let short = v.tokens.iter().filter(|v| v.len() == 1).count();
                v.formatted().len() + 4 * short
            })
            .max()
            .unwrap_or(0);

        for opt in matches {
            let current_n_short = opt.tokens.iter().filter(|v| v.len() == 1).count();

            // 4 is the number of characters that separate short options
            let indent_by = PAD + 4 * (n_short - current_n_short);
            let width = width + 4 * current_n_short;

            let tokens = opt.formatted();
            let desc = opt.desc.clone().unwrap_or_default();
            let line = format!("{:indent_by$}{tokens:width$}{desc}", " ");
            lines.push(line);
        }
        lines
    }

    fn format_commands(&self) -> Vec<String> {
        let mut lines = vec![];
        let width = self
            .subcommands
            .values()
            .iter()
            .flatten()
            .map(|v| v.name.green().len())
            .max()
            .unwrap_or(0);

        for (section, cmds) in self.subcommands.iter() {
            lines.push(format!("\n{section}:").yellow());

            let width = width + PAD;
            for cmd in cmds {
                let desc = cmd.desc.clone().unwrap_or_default();
                let line = format!("{:PAD$}{:width$} {desc}", " ", cmd.name.green());
                lines.push(line);
            }
        }
        lines
    }

    fn validate(result: &ParseResult) {
        for opt in result.options.values() {
            if let Some(token) = opt.tokens.last() {
                if opt.required && result.get_vec(token).is_none() {
                    let error = "error:".red();
                    eprintln!("{error} required option '{token}' was not specified");
                    std::process::exit(1);
                }
            }
        }

        if let Some(cmd) = result.subcommand() {
            Self::validate(cmd);
        }
    }

    fn default_opts(self) -> Self {
        let help = Opt::from(["h", "help"]).desc("Display help information");
        self.opt(help)
    }
}

pub struct Category {
    pub name: &'static str,
    pub commands: Vec<CommandLine>,
}

impl Category {
    pub fn with_commands(name: &'static str, commands: Vec<CommandLine>) -> Self {
        Self { name, commands }
    }
}

#[must_use]
#[derive(Debug, Clone)]
pub struct Opt {
    tokens: Vec<String>,
    desc: Option<String>,
    kind: Value,
    value_name: Option<String>,
    required: bool,
    count: usize,
    values: Vec<String>,
}

impl Opt {
    pub fn desc(mut self, desc: impl Into<String>) -> Self {
        self.desc = Some(desc.into());
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn value(mut self, value: Value, name: impl Into<String>) -> Self {
        self.kind = value;
        self.value_name = Some(name.into());
        self
    }

    fn formatted(&self) -> String {
        let mut tokens = vec![];
        for token in &self.tokens {
            let sep = if token.len() > 1 { "--" } else { "-" };
            tokens.push(format!("{sep}{token}"));
        }

        let mut line = tokens.join(", ");
        if self.kind != Value::Flag {
            let name = self.value_name.clone().unwrap_or_else(|| "arg".into());
            line = format!("{line} <{name}>");
        }
        line.green()
    }
}

impl<'a> From<&'a str> for Opt {
    fn from(value: &'a str) -> Self {
        Self::from([value])
    }
}

impl<'a, const N: usize> From<[&'a str; N]> for Opt {
    fn from(value: [&'a str; N]) -> Self {
        Self {
            tokens: value.into_iter().map(ToString::to_string).collect(),
            desc: None,
            kind: Value::Flag,
            value_name: None,
            required: false,
            count: 0,
            values: vec![],
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Flag,
    Single,
    Multiple,
}

pub trait Command {
    fn command() -> CommandLine;

    fn from_args<I>(args: I) -> Self
    where
        Self: Sized,
        I: IntoIterator<Item = String>,
    {
        let result = Self::command().parse_args(args.into_iter());
        Self::from_result(&result)
    }

    fn parse() -> Self
    where
        Self: Sized,
    {
        Self::from_args(env::args().skip(1))
    }

    fn from_result(result: &ParseResult) -> Self;
}
