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

#![allow(
    clippy::struct_excessive_bools,
    clippy::print_stdout,
    clippy::print_stderr
)]

use std::collections::HashMap;
use std::env;

use color::Colorize;
use ic_alloc::index::IndexMap;
pub use ic_cli_derive::Command;
pub mod color;
pub mod convert;
pub mod terminal;

mod parse;
pub use parse::{ParseError, ParseResult};

const LEFT_MARGIN: usize = 3;
const DESC_SPACING: usize = 4;

fn display_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;

    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape && ch == 'm' {
            in_escape = false;
        } else if !in_escape {
            width += 1;
        }
    }

    width
}

#[must_use]
#[derive(Default)]
pub struct CommandLine {
    name: String,
    desc: String,
    version: Option<String>,
    options: IndexMap<String, Opt>,
    split_flags: bool,
    hide_flags: bool,
    hide_options: bool,
    align_sections: bool,
    arg_name: Option<String>,
    after_help: Option<String>,
    positionals: bool,
    external: bool,
    parent: Option<String>,
    subcommands: HashMap<String, Vec<CommandLine>>,
}

impl CommandLine {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            desc: String::new(),
            version: None,
            options: IndexMap::new(),
            split_flags: true,
            hide_flags: false,
            hide_options: false,
            align_sections: false,
            arg_name: None,
            after_help: None,
            external: false,
            positionals: false,
            parent: None,
            subcommands: HashMap::new(),
        }
    }

    pub fn desc(mut self, desc: impl Into<String>) -> Self {
        self.desc = desc.into();
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn section(mut self, section: impl Into<String>, mut cmd: CommandLine) -> Self {
        let section = section.into();
        for opt in cmd.options.values_mut() {
            opt.section = Some(section.clone());
        }
        self = self.opts(cmd.options.values().cloned());
        self
    }

    pub fn opt(mut self, option: Opt) -> Self {
        self.options.insert_multi(option.tokens.clone(), option);
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

    #[allow(clippy::needless_pass_by_value)]
    pub fn merge(self, command: CommandLine) -> Self {
        self.opts(command.options.values().cloned())
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

        self.subcommands.insert(category.name.to_string(), commands);
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

    pub fn split_flags(mut self, split_flags: bool) -> Self {
        self.split_flags = split_flags;
        self
    }

    pub fn hide_flags(mut self, hide_flags: bool, hide_options: bool) -> Self {
        self.hide_flags = hide_flags;
        self.hide_options = hide_options;
        self
    }

    pub fn align_sections(mut self, align_sections: bool) -> Self {
        self.align_sections = align_sections;
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

    /// Overrides the name set in `CommandLine::new`.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Returns the name of the application.
    #[must_use]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Attempts to parse the command-line arguments.
    ///
    /// # Errors
    ///
    /// May fail due to syntax errors in the input.
    pub fn try_parse(self) -> Result<ParseResult, ParseError> {
        self.try_parse_args(std::env::args().skip(1))
    }

    pub fn parse(self) -> ParseResult {
        self.parse_args(std::env::args().skip(1))
    }

    pub fn parse_args<I>(self, iter: I) -> ParseResult
    where
        I: Iterator<Item = String>,
    {
        match self.try_parse_args(iter) {
            Ok(v) => v,
            Err(e) => match e {
                ParseError::Help(msg) => {
                    println!("{msg}");
                    std::process::exit(0);
                }
                ParseError::Status(msg) => {
                    let error = "error:".red().bold();
                    eprintln!("{error} {msg}");
                    std::process::exit(1);
                }
            },
        }
    }

    /// # Errors
    ///
    /// Returns an error if parsing failed due to a syntactic error, or if one
    /// of the help flags (e.g. `-h`) were specified by the user.
    pub fn try_parse_args<I>(mut self, iter: I) -> Result<ParseResult, ParseError>
    where
        I: Iterator<Item = String>,
    {
        self = self.default_opts();
        let result = parse::from_args(iter, &mut self)?;

        if !self.subcommands.is_empty() && result.subcommand().is_none() {
            Err(ParseError::Help(self.help()))
        } else {
            Self::validate(&result);
            Ok(result)
        }
    }

    #[must_use]
    pub fn help(&self) -> String {
        let mut lines: Vec<String> = vec![];

        let version = self.version.clone().unwrap_or_default();
        lines.push(format!(
            "{} {version}",
            self.qualified_name('-', false).yellow().bold()
        ));

        if !self.desc.is_empty() {
            lines.push(format!("\n{}", self.desc));
        }

        lines.push("\nusage:".yellow().bold().to_string());

        let mut usage = format!("{:LEFT_MARGIN$}{}", " ", self.qualified_name(' ', true));
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

        if self.split_flags {
            if !self.hide_flags {
                let flags = self.format_args(|v| v.kind == Value::Flag && v.section.is_none());
                if !flags.is_empty() {
                    lines.push("\nflags:".yellow().bold().to_string());
                    lines.extend(flags);
                }
            }
            if !self.hide_options {
                let options = self.format_args(|v| v.kind != Value::Flag && v.section.is_none());
                if !options.is_empty() {
                    lines.push("\noptions:".yellow().bold().to_string());
                    lines.extend(options);
                }
            }
        } else {
            let options = self.format_args(|v| {
                ((!self.hide_flags && v.kind == Value::Flag)
                    || (!self.hide_options && v.kind != Value::Flag))
                    && v.section.is_none()
            });
            if !options.is_empty() {
                lines.push("\noptions:".yellow().bold().to_string());
                lines.extend(options);
            }
        }

        {
            // Group options by their section
            let mut sections = IndexMap::<_, Vec<_>>::new();
            let options = self.options.values();
            for opt in options {
                if let Some(v) = &opt.section {
                    if let Some(v) = sections.get_mut(v) {
                        v.push(opt);
                    } else {
                        sections.insert(v.clone(), vec![opt]);
                    }
                }
            }

            for section in &sections {
                let flags = self.format_args(|v| {
                    if let Some(name) = &v.section {
                        name == section.0
                    } else {
                        false
                    }
                });

                lines.push(format!("\n{}:", section.0).yellow().bold().to_string());
                lines.extend(flags);
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
        name.push_str(&self.name);
        name
    }

    fn exe_name(&self) -> String {
        if let Ok(exe) = env::current_exe()
            && let Some(stem) = exe.file_stem()
        {
            return stem.to_string_lossy().to_string();
        }
        self.name.clone()
    }

    fn format_args<P>(&self, filter: P) -> Vec<String>
    where
        P: FnMut(&&Opt) -> bool + Clone,
    {
        self.format_args_common(filter, None)
    }

    pub fn format_args_prefix<P>(&self, prefix: &str, filter: P) -> Vec<String>
    where
        P: FnMut(&&Opt) -> bool + Clone,
    {
        self.format_args_common(filter, Some(prefix))
    }

    /// Common implementation for formatting command line arguments.
    /// If prefix is None, uses default formatting with -/-- prefixes.
    /// If prefix is Some, uses the provided custom prefix.
    fn format_args_common<P>(&self, filter: P, prefix: Option<&str>) -> Vec<String>
    where
        P: FnMut(&&Opt) -> bool + Clone,
    {
        let mut lines = vec![];
        let matches: Vec<_> = self.options.values().filter(filter).collect();

        // Check if this specific section has any short options
        let section_has_short_opts = matches
            .iter()
            .any(|v| v.tokens.iter().any(|t| t.len() == 1));

        // Find the maximum number of short options in this section
        let n_short_section: usize = matches
            .iter()
            .map(|v| v.tokens.iter().filter(|v| v.len() == 1).count())
            .max()
            .unwrap_or(0);

        // Calculate the maximum width including indentation for alignment
        let max_width_with_indent = if self.align_sections {
            self.options
                .values()
                .map(|v| match prefix {
                    Some(p) => display_width(&v.with_prefix(p)),
                    None => display_width(&v.formatted()),
                })
                .max()
                .unwrap_or(0)
        } else {
            matches
                .iter()
                .map(|v| {
                    let current_n_short = v.tokens.iter().filter(|t| t.len() == 1).count();
                    let indent = if section_has_short_opts {
                        4 * (n_short_section.saturating_sub(current_n_short))
                    } else {
                        0
                    };
                    let token_width = match prefix {
                        Some(p) => display_width(&v.with_prefix(p)),
                        None => display_width(&v.formatted()),
                    };
                    indent + token_width
                })
                .max()
                .unwrap_or(0)
        };

        // Calculate the column where descriptions should start
        let desc_column = LEFT_MARGIN + max_width_with_indent + DESC_SPACING;

        for opt in matches {
            let current_n_short = opt.tokens.iter().filter(|v| v.len() == 1).count();

            // 4 is the number of characters that separate short options
            let current_width = if section_has_short_opts {
                4 * (n_short_section.saturating_sub(current_n_short))
            } else {
                0
            };
            let indent_by = LEFT_MARGIN + current_width;

            let tokens = match prefix {
                Some(p) => opt.with_prefix(p),
                None => opt.formatted(),
            };

            let tokens_display_width = display_width(&tokens);
            let current_position = indent_by + tokens_display_width;
            let padding_width = desc_column.saturating_sub(current_position);
            let desc = opt.desc.clone().unwrap_or_default();
            let line = format!(
                "{:indent_by$}{tokens}{}{desc}",
                " ",
                " ".repeat(padding_width)
            );
            lines.push(line);
        }
        lines
    }

    fn format_commands(&self) -> Vec<String> {
        let mut lines = vec![];
        let max_name_width = self
            .subcommands
            .values()
            .flatten()
            .map(|v| display_width(&v.name))
            .max()
            .unwrap_or(0);

        let desc_column = LEFT_MARGIN + max_name_width + DESC_SPACING;

        for (section, cmds) in &self.subcommands {
            lines.push(format!("\n{section}:").yellow().bold().to_string());

            for cmd in cmds {
                let name_styled = cmd.name.cyan().to_string();
                let name_display_width = display_width(&cmd.name);
                let current_position = LEFT_MARGIN + name_display_width;
                let padding_width = desc_column.saturating_sub(current_position);
                let line = format!(
                    "{:LEFT_MARGIN$}{}{}{}",
                    " ",
                    name_styled,
                    " ".repeat(padding_width),
                    cmd.desc
                );
                lines.push(line);
            }
        }
        lines
    }

    fn validate(result: &ParseResult) {
        for opt in result.options.values() {
            if let Some(token) = opt.tokens.last()
                && opt.required
                && result.get_vec(token).is_none()
            {
                let error = "error:".red().bold();
                eprintln!("{error} required option '{token}' was not specified");
                std::process::exit(1);
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
    #[must_use]
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
    section: Option<String>,
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

    pub fn section(mut self, section: impl Into<String>) -> Self {
        self.section = Some(section.into());
        self
    }

    fn formatted(&self) -> String {
        let mut tokens = vec![];
        for token in &self.tokens {
            let sep = if token.len() > 1 { "--" } else { "-" };
            tokens.push(format!("{sep}{token}"));
        }

        let mut line = tokens.join(", ").cyan().to_string();
        if self.kind != Value::Flag {
            let name = self.value_name.clone().unwrap_or_else(|| "arg".into());
            line = format!("{line} {}", format!("<{name}>").cyan().dim());
        }
        line
    }

    fn with_prefix(&self, prefix: &str) -> String {
        let mut tokens = vec![];
        for token in &self.tokens {
            tokens.push(format!("{prefix}{token}"));
        }

        let mut line = tokens.join(", ").cyan().to_string();
        if self.kind != Value::Flag {
            let name = self.value_name.clone().unwrap_or_else(|| "arg".into());
            line = format!("{line} {}", format!("<{name}>").cyan().dim());
        }
        line
    }

    pub(crate) fn insert_value(&mut self, value: String) {
        self.count += 1;
        match self.kind {
            Value::Flag | Value::Single => {
                if let Some(v) = self.values.first_mut() {
                    *v = value;
                } else {
                    self.values.push(value);
                }
            }
            Value::Multiple => self.values.push(value),
        }
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
            section: None,
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

    #[must_use]
    fn from_args<I>(args: I) -> Self
    where
        Self: Sized,
        I: IntoIterator<Item = String>,
    {
        let result = Self::command().parse_args(args.into_iter());
        Self::from_result(&result)
    }

    #[must_use]
    fn parse() -> Self
    where
        Self: Sized,
    {
        Self::from_args(env::args().skip(1))
    }

    #[must_use]
    fn from_result(result: &ParseResult) -> Self;
}
