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

use crate::color::Colorize;
use crate::index::IndexMap;
use crate::{CommandLine, Opt, Value};

#[must_use]
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

struct Parser<I: Iterator<Item = String>> {
    iter: I,
    result: ParseResult,
}

impl<I> Parser<I>
where
    I: Iterator<Item = String>,
{
    fn parse(&mut self, context: &mut CommandLine) -> Result<(), ParseError> {
        let mut had_option = false;
        while let Some(arg) = self.iter.next() {
            // end of options
            let is_end = arg == "--";
            if context.external || is_end {
                self.collect_remaining(arg, !is_end);
            }
            // long option
            else if let Some(option) = arg.strip_prefix("--") {
                let (option, value) = Self::split(option);
                self.parse_arg(context, option, value)?;
            }
            // short, possibly chained option(s)
            else if let Some(option) = arg.strip_prefix('-') {
                self.parse_arg(context, option, None)?;
            }
            // subcommand or positional argument
            else {
                self.unnamed(context, &arg)?;
            }
            had_option = true;
        }

        if had_option || (context.subcommands.is_empty() && !context.positionals) {
            Ok(())
        } else {
            Err(ParseError::Help(context.help()))
        }
    }

    fn split(arg: &str) -> (&str, Option<String>) {
        match arg.split_once('=') {
            Some((a, v)) => (a, Some(v.to_string())),
            None => (arg, None),
        }
    }

    fn unnamed(&mut self, context: &mut CommandLine, arg: &str) -> Result<(), ParseError> {
        if context.subcommands.is_empty() {
            if context.positionals {
                self.result().positionals.push(arg.to_string());
            } else {
                return Err(format!("unexpected value: '{}'", arg.yellow()).into());
            }
        } else if let Some(cmd) = context
            .subcommands
            .values_mut()
            .flat_map(|c| c.iter_mut())
            .find(|v| v.name == arg)
        {
            let result = ParseResult::from(cmd);
            self.result().subcommand = Some(Box::new(result));
            self.parse(cmd)?;
        } else {
            return Err(format!("unknown subcommand '{}'", arg.yellow()).into());
        }
        Ok(())
    }

    fn consume_arg(&mut self, name: &str) -> Result<String, ParseError> {
        let arg = self
            .iter
            .next()
            .ok_or_else(|| format!("argument to '{}' is missing", name.yellow()))?;

        if arg.starts_with('-') {
            Err(format!("expected value, found '{}'", arg.yellow()).into())
        } else {
            Ok(arg)
        }
    }

    fn parse_arg(
        &mut self,
        context: &mut CommandLine,
        name: &str,
        value: Option<String>,
    ) -> Result<(), ParseError> {
        let Some(opt) = context.options.get_mut(name) else {
            return Err(did_you_mean(name, context.options.values()));
        };

        if name == "h" || name == "help" {
            return Err(ParseError::Help(context.help()));
        }

        if opt.kind == Value::Flag {
            // temporary hack to make flags work properly with the derive macro
            let parsed = self.result().options.get_mut(name).unwrap();
            parsed.values.push(true.to_string());
        } else {
            let value = match value {
                Some(v) => v,
                None => self.consume_arg(name)?,
            };
            let value: Vec<_> = value.split(',').collect();
            let parsed = self.result().options.get_mut(name).unwrap();

            for v in value {
                if opt.kind == Value::Single && !opt.values.is_empty() {
                    parsed.values[0] = v.to_string();
                } else {
                    parsed.values.push(v.to_string());
                }
            }
        }
        opt.count += 1;
        Ok(())
    }

    fn result(&mut self) -> &mut ParseResult {
        fn current(result: &mut ParseResult) -> &mut ParseResult {
            if let Some(ref mut inner) = result.subcommand {
                current(inner.as_mut())
            } else {
                result
            }
        }
        current(&mut self.result)
    }

    fn collect_remaining(&mut self, arg: String, include_arg: bool) {
        let mut positionals: Vec<String> = vec![];
        if include_arg {
            positionals.push(arg);
        }
        positionals.extend(self.iter.by_ref());
        self.result().positionals.extend(positionals);
    }
}

pub fn from_args<I>(iter: I, cmd: &mut CommandLine) -> Result<ParseResult, ParseError>
where
    I: Iterator<Item = String>,
{
    let mut parser = Parser {
        iter,
        result: ParseResult::from(cmd),
    };
    parser.parse(cmd)?;
    Ok(parser.result)
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

fn did_you_mean(input: &str, options: &[Opt]) -> ParseError {
    let err = if let Some(v) = closest_match(input, options) {
        format!(
            "unknown option '{}', did you mean '{}'?",
            input.yellow(),
            v.yellow(),
        )
    } else {
        format!("unknown option '{}'", input.yellow())
    };
    ParseError::Status(err)
}
