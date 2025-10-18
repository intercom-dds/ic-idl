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

use ic_alloc::index::IndexMap;

use crate::color::Colorize;
use crate::{CommandLine, Opt, Value};

#[must_use]
#[derive(Debug)]
pub struct ParseResult {
    pub(crate) name: String,
    pub(crate) options: IndexMap<String, Opt>,
    pub(crate) subcommand: Option<Box<ParseResult>>,
    pub(crate) positionals: Vec<String>,
}

impl ParseResult {
    fn from(command: &CommandLine) -> Self {
        Self {
            name: command.get_name().to_string(),
            options: command.options.clone(),
            subcommand: None,
            positionals: vec![],
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.options
            .get(key)
            .map(|v| v.values.last())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn get_vec(&self, key: &str) -> Option<&Vec<String>> {
        if let Some(value) = self.options.get(key).map(|v| &v.values) {
            if !value.is_empty() {
                return Some(value);
            }
        }
        None
    }

    #[must_use]
    pub fn count(&self, key: &str) -> usize {
        self.options.get(key).map_or(0, |v| v.count)
    }

    #[must_use]
    pub fn is_present(&self, key: &str) -> bool {
        self.count(key) != 0
    }

    #[must_use]
    pub fn subcommand(&self) -> Option<&ParseResult> {
        self.subcommand.as_deref()
    }

    #[must_use]
    pub fn positionals(&self) -> &Vec<String> {
        &self.positionals
    }
}

#[derive(Clone, Debug)]
pub enum ParseError {
    Help(String),
    Status(String),
}

impl From<String> for ParseError {
    fn from(value: String) -> Self {
        Self::Status(value)
    }
}

struct Parser<'a> {
    ctx: &'a CommandLine,
    result: ParseResult,
    args: &'a mut dyn Iterator<Item = String>,
}

impl<'a> Parser<'a> {
    fn with_command(
        ctx: &'a CommandLine,
        args: &'a mut dyn Iterator<Item = String>,
    ) -> Result<ParseResult, ParseError> {
        let result = ParseResult::from(ctx);
        Self { ctx, result, args }.parse()
    }

    fn parse(mut self) -> Result<ParseResult, ParseError> {
        let mut had_option = false;
        while let Some(arg) = self.args.next() {
            let is_end = arg == "--";

            // end of options and/or an external command
            if is_end || self.ctx.external {
                self.collect_remaining(arg, !is_end);
            }
            // long options
            else if let Some(arg) = arg.strip_prefix("--") {
                self.long_opt(arg)?;
            }
            // short options
            else if let Some(arg) = arg.strip_prefix("-") {
                self.short_opt(arg)?;
            }
            // subcommand
            else if !self.ctx.subcommands.is_empty() {
                let sub = self.subcommand(&arg)?;
                self.result.subcommand = Some(Box::new(sub));
                break;
            }
            // positional
            else if self.ctx.positionals {
                self.result.positionals.push(arg);
            } else {
                return Err(format!("unexpected value: '{}'", arg.yellow()).into());
            }
            had_option = true;
        }

        if (had_option || self.result.subcommand.is_some())
            || (self.ctx.subcommands.is_empty() && !self.ctx.positionals)
        {
            Ok(self.result)
        } else {
            Err(ParseError::Help(self.ctx.help()))
        }
    }

    fn maybe_help(&self, opt: &str) -> Result<(), ParseError> {
        if opt == "h" || opt == "help" {
            Err(ParseError::Help(self.ctx.help()))
        } else {
            Ok(())
        }
    }

    fn find_opt(&mut self, opt: &str) -> Result<&mut Opt, ParseError> {
        self.maybe_help(opt)?;
        self.result
            .options
            .get_mut(opt)
            .ok_or_else(|| did_you_mean(opt, self.ctx.options.values()))
    }

    fn short_opt(&mut self, arg: &str) -> Result<(), ParseError> {
        match arg.len() {
            0 => Err(format!("unexpected argument '{}'", '-'.yellow()).into()),
            1 => self.handle_opt(arg),
            _ => {
                let (key, val) = arg.split_at(1);
                let opt = self.find_opt(key)?;
                opt.insert_value(val.to_string());
                Ok(())
            }
        }
    }

    fn long_opt(&mut self, arg: &str) -> Result<(), ParseError> {
        if let Some((key, value)) = arg.split_once('=') {
            let opt = self.find_opt(key)?;
            for val in value.split(',') {
                opt.insert_value(val.to_string());
            }
        } else {
            self.handle_opt(arg)?;
        }
        Ok(())
    }

    fn handle_opt(&mut self, name: &str) -> Result<(), ParseError> {
        let kind = self.find_opt(name)?.kind;
        let value = if kind == Value::Flag {
            true.to_string()
        } else {
            let arg = self
                .args
                .next()
                .ok_or_else(|| format!("argument to '{}' is missing", prefixed(name)))?;

            if arg.starts_with('-') {
                return Err(ParseError::Status(format!(
                    "expected argument to '{}', found option '{}'",
                    prefixed(name),
                    arg.yellow(),
                )));
            }
            arg.clone()
        };

        self.find_opt(name)?.insert_value(value);
        Ok(())
    }

    fn subcommand(&mut self, cmd: &str) -> Result<ParseResult, ParseError> {
        if let Some(cmd) = self
            .ctx
            .subcommands
            .values()
            .flat_map(|c| c.iter())
            .find(|v| v.name == cmd)
        {
            Parser::with_command(cmd, self.args)
        } else {
            Err(format!("unknown subcommand '{}'", cmd.yellow()).into())
        }
    }

    fn collect_remaining(&mut self, arg: String, include_arg: bool) {
        if include_arg {
            self.result.positionals.push(arg);
        }
        self.result.positionals.extend(&mut *self.args);
    }
}

pub fn from_args<I>(mut iter: I, cmd: &mut CommandLine) -> Result<ParseResult, ParseError>
where
    I: Iterator<Item = String>,
{
    Parser::with_command(cmd, &mut iter)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a = a.as_bytes();
    let b = b.as_bytes();

    let len_a = a.len();
    let len_b = b.len();
    let mut column = vec![0; len_a];

    for i in 0..=len_a {
        column.push(i);
    }

    for x in 1..=len_b {
        column[0] = x;
        let mut last_diag = x.saturating_sub(1);

        for y in 1..=len_a {
            let old_diag = column[y];
            let eq = a[y - 1] != b[x - 1];
            let ins_cost = column[y] + 1;
            let sub_cost = column[y - 1] + 1;
            let del_cost = last_diag + usize::from(eq);

            let min = ins_cost.min(sub_cost);
            column[y] = min.min(del_cost);
            last_diag = old_diag;
        }
    }
    column[len_a]
}

fn closest_match<'a>(input: &str, options: &'a [Opt]) -> Option<&'a str> {
    let mut min = usize::MAX;
    let mut closest = None;

    let iter = options
        .iter()
        .flat_map(|v| v.tokens.iter())
        .map(String::as_str)
        .filter(|v| v.len() > 1);

    for tok in iter {
        let dist = levenshtein(input, tok);
        if dist < min {
            min = dist;

            let len = input.len();
            let range = (2 * tok.len()).saturating_div(3);
            let suggested = tok.len();

            if min <= range && suggested - range <= len && len <= suggested + range {
                closest = Some(tok);
            }
        }
    }
    closest
}

fn prefixed(name: &str) -> String {
    let prefix = if name.len() > 1 { "--" } else { "-" };
    format!("{prefix}{name}").yellow().to_string()
}

fn did_you_mean(input: &str, options: &[Opt]) -> ParseError {
    let err = if let Some(v) = closest_match(input, options) {
        format!(
            "unknown option '{}', did you mean '{}'?",
            prefixed(input),
            prefixed(v),
        )
    } else {
        format!("unknown option '{}'", prefixed(input))
    };
    ParseError::Status(err)
}
