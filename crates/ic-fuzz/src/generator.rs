// Copyright 2025 KONGSBERG
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

//! IDL source code generator from grammar.

use std::fmt::Write;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::grammar::{Grammar, Repetition, Rule, RuleElement};
use crate::terminals::TerminalGenerator;

#[derive(Debug, Clone)]
pub struct Generated {
    pub source: String,
    pub token_count: usize,
}

/// Configuration for the fuzzer.
#[derive(Debug, Clone)]
pub struct FuzzerConfig {
    pub max_depth: usize,
    pub min_repetitions: usize,
    pub max_repetitions: usize,
    pub max_tokens: Option<usize>,
    pub annotation_probability: Option<f64>,
    pub optional_probability: f64,
    pub seed: Option<u64>,
}

impl Default for FuzzerConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            min_repetitions: 0,
            max_repetitions: 5,
            max_tokens: None,
            annotation_probability: None,
            optional_probability: 0.5,
            seed: None,
        }
    }
}

impl FuzzerConfig {
    /// Defaults to `max_depth * max_repetitions^2 * 100` (min 1000).
    #[must_use]
    pub fn effective_max_tokens(&self) -> usize {
        self.max_tokens.unwrap_or_else(|| {
            let base = self.max_depth * self.max_repetitions * self.max_repetitions * 100;
            base.max(1000) // minimum 1000 tokens
        })
    }
}

pub struct Fuzzer<'g> {
    grammar: &'g Grammar,
    config: FuzzerConfig,
    max_tokens: usize,
    token_count: usize,
    at_line_start: bool,
    rng: SmallRng,
}

impl<'g> Fuzzer<'g> {
    #[must_use]
    pub fn new(grammar: &'g Grammar, config: FuzzerConfig) -> Self {
        let rng = match config.seed {
            Some(seed) => SmallRng::seed_from_u64(seed),
            None => SmallRng::from_os_rng(),
        };
        let max_tokens = config.effective_max_tokens();
        Self {
            grammar,
            config,
            max_tokens,
            token_count: 0,
            at_line_start: true,
            rng,
        }
    }

    pub fn generate(&mut self) -> Generated {
        let mut out = String::with_capacity(64 * 1024);
        self.generate_into(&mut out);
        Generated {
            source: out,
            token_count: self.token_count,
        }
    }

    pub fn generate_into(&mut self, out: &mut impl Write) -> usize {
        self.token_count = 0;
        self.at_line_start = true;
        let start = self.grammar.start.as_str();
        self.generate_rule(start, 0, out);
        self.token_count
    }

    pub fn generate_with_seed(&mut self, seed: u64) -> Generated {
        self.rng = SmallRng::seed_from_u64(seed);
        self.generate()
    }

    pub fn generate_with_seed_into(&mut self, seed: u64, out: &mut impl Write) -> usize {
        self.rng = SmallRng::seed_from_u64(seed);
        self.generate_into(out)
    }

    fn emit_token(&mut self, token: &str, out: &mut impl Write) {
        self.token_count += 1;

        // Put closing braces on their own line
        if token == "}" && !self.at_line_start {
            _ = out.write_char('\n');
            self.at_line_start = true;
        }

        if !self.at_line_start && Self::needs_space_before(token) {
            _ = out.write_char(' ');
        }

        _ = out.write_str(token);

        if Self::needs_newline_after(token) {
            _ = out.write_char('\n');
            self.at_line_start = true;
        } else {
            self.at_line_start = false;
        }
    }

    fn needs_space_before(token: &str) -> bool {
        if token.starts_with('@') {
            return true;
        }
        !matches!(
            token,
            ";" | "," | ")" | "}" | "]" | ">" | "(" | "{" | "[" | "<"
        )
    }

    fn needs_newline_after(token: &str) -> bool {
        matches!(token, ";" | "{")
    }

    fn annotation_probability(&self) -> f64 {
        self.config
            .annotation_probability
            .unwrap_or(self.grammar.annotations.probability)
    }

    /// Geometric distribution; bias < 1.0 means fewer reps.
    fn repetition_count_one_or_more(&mut self, bias: f64) -> usize {
        let mut count = 1;
        let prob = (0.5 * bias).clamp(0.0, 0.9);
        while count < self.config.max_repetitions && self.rng.random_bool(prob) {
            count += 1;
        }
        count
    }

    fn repetition_count_zero_or_more(&mut self, bias: f64) -> usize {
        let prob = (0.7 * bias).clamp(0.0, 1.0);

        if !self.rng.random_bool(prob) {
            return 0;
        }

        self.rng
            .random_range(1..=self.config.max_repetitions.max(1))
    }

    fn generate_rule(&mut self, name: &str, depth: usize, out: &mut impl Write) {
        if let Some(spec) = self.grammar.get_terminal(name) {
            self.emit_terminal(spec, out);
            return;
        }
        let Some(rule) = self.grammar.get_rule(name) else {
            return;
        };
        self.generate_rule_inner(rule, depth, out);
    }

    fn emit_terminal(&mut self, spec: &crate::grammar::TerminalSpec, out: &mut impl Write) {
        self.token_count += 1;

        if !self.at_line_start {
            _ = out.write_char(' ');
        }

        TerminalGenerator::new(&mut self.rng).write(spec, out);
        self.at_line_start = false;
    }

    fn generate_rule_inner(&mut self, rule: &Rule, depth: usize, out: &mut impl Write) {
        match rule {
            Rule::Sequence { seq, .. } => {
                self.generate_sequence(seq, rule.repetition_bias(), depth, out);
            }
            Rule::Choice { choice, .. } => self.generate_choice(choice, depth, out),
            Rule::Optional { opt } => {
                if self.rng.random_bool(self.config.optional_probability) {
                    self.generate_element(opt, 1.0, depth, out);
                }
            }
        }
    }

    fn generate_sequence(
        &mut self,
        seq: &[RuleElement],
        repetition_bias: f64,
        depth: usize,
        out: &mut impl Write,
    ) {
        for (i, elem) in seq.iter().enumerate() {
            if i > 0 {
                let next_is_lparen = elem.literal_value() == Some("(");
                if !next_is_lparen {
                    self.maybe_inject_annotation(depth, out);
                }
            }
            self.generate_element(elem, repetition_bias, depth, out);
        }
    }

    fn generate_choice(&mut self, choice: &[RuleElement], depth: usize, out: &mut impl Write) {
        if choice.is_empty() {
            return;
        }

        // At max depth, pick non-recursive alternatives to terminate
        let elem = if depth >= self.config.max_depth {
            let non_recursive: Vec<_> = choice
                .iter()
                .filter(|e| e.is_literal() || self.is_non_recursive(e))
                .collect();
            if non_recursive.is_empty() {
                self.weighted_choice(choice)
            } else {
                // TODO: respect weights in non-recursive selection too
                non_recursive[self.rng.random_range(0..non_recursive.len())]
            }
        } else {
            self.weighted_choice(choice)
        };
        self.generate_element(elem, 1.0, depth, out);
    }

    fn element_weight(&self, elem: &RuleElement) -> f64 {
        match elem.rule_name() {
            Some(name) => self.grammar.get_rule_bias(name),
            None => 1.0,
        }
    }

    fn weighted_choice<'a>(&mut self, choice: &'a [RuleElement]) -> &'a RuleElement {
        let total: f64 = choice.iter().map(|e| self.element_weight(e)).sum();
        if total <= 0.0 {
            return &choice[self.rng.random_range(0..choice.len())];
        }
        let mut r = self.rng.random::<f64>() * total;
        for elem in choice {
            r -= self.element_weight(elem);
            if r <= 0.0 {
                return elem;
            }
        }
        choice.last().unwrap()
    }

    fn is_non_recursive(&self, elem: &RuleElement) -> bool {
        let RuleElement::Literal(s) = elem else {
            return false;
        };
        if s.starts_with('\'') {
            return false;
        }
        let name = s
            .trim_end_matches('*')
            .trim_end_matches('+')
            .trim_end_matches('?');
        self.grammar.is_terminal(name) || self.is_leaf_rule(name)
    }

    fn is_leaf_rule(&self, name: &str) -> bool {
        let Some(rule) = self.grammar.get_rule(name) else {
            return false;
        };
        match rule {
            Rule::Choice { choice, .. } => choice.iter().any(|elem| match elem {
                RuleElement::Literal(s) if !s.starts_with('\'') => self.grammar.is_terminal(s),
                RuleElement::Literal(_) | RuleElement::Inline(_) => false,
            }),
            Rule::Sequence { seq, .. } => seq.iter().all(|elem| match elem {
                RuleElement::Literal(s) => s.starts_with('\'') || self.grammar.is_terminal(s),
                RuleElement::Inline(_) => false,
            }),
            Rule::Optional { .. } => false,
        }
    }

    fn generate_element(
        &mut self,
        elem: &RuleElement,
        bias: f64,
        depth: usize,
        out: &mut impl Write,
    ) {
        match elem {
            RuleElement::Literal(s) => {
                if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
                    let token = &s[1..s.len() - 1];
                    self.emit_token(token, out);
                } else {
                    self.generate_rule_ref(s, elem.repetition(), bias, depth, out);
                }
            }
            RuleElement::Inline(rule) => self.generate_rule_inner(rule, depth, out),
        }
    }

    fn generate_rule_ref(
        &mut self,
        s: &str,
        rep: Repetition,
        parent_repetition_bias: f64,
        depth: usize,
        out: &mut impl Write,
    ) {
        let name = s
            .trim_end_matches('*')
            .trim_end_matches('+')
            .trim_end_matches('?');

        // Use rule's repetition_bias if set, otherwise inherit from parent
        let repetition_bias = {
            let rule_bias = self.grammar.get_rule_repetition_bias(name);
            if (rule_bias - 1.0).abs() < f64::EPSILON {
                parent_repetition_bias
            } else {
                rule_bias
            }
        };

        match rep {
            Repetition::Once => self.generate_rule(name, depth + 1, out),
            Repetition::Optional => {
                if self.token_count < self.max_tokens
                    && self.rng.random_bool(self.config.optional_probability)
                {
                    self.generate_rule(name, depth + 1, out);
                }
            }
            Repetition::ZeroOrMore => {
                let count = self.repetition_count_zero_or_more(repetition_bias);
                for i in 0..count {
                    if self.token_count >= self.max_tokens {
                        break;
                    }
                    if i > 0 {
                        self.maybe_inject_annotation(depth, out);
                    }
                    self.generate_rule(name, depth + 1, out);
                }
            }
            Repetition::OneOrMore => {
                let count = self.repetition_count_one_or_more(repetition_bias);
                for i in 0..count {
                    if i > 0 && self.token_count >= self.max_tokens {
                        break;
                    }
                    if i > 0 {
                        self.maybe_inject_annotation(depth, out);
                    }
                    self.generate_rule(name, depth + 1, out);
                }
            }
        }
    }

    fn maybe_inject_annotation(&mut self, depth: usize, out: &mut impl Write) {
        let prob = self.annotation_probability();
        if prob <= 0.0 || depth >= self.config.max_depth || !self.rng.random_bool(prob) {
            return;
        }
        if self.grammar.annotations.names.is_empty() {
            return;
        }
        let idx = self
            .rng
            .random_range(0..self.grammar.annotations.names.len());
        let name = &self.grammar.annotations.names[idx];
        let args_prob = self.grammar.annotations.args_probability;

        self.token_count += 1;
        _ = out.write_char('@');
        _ = out.write_str(name);
        _ = out.write_char('(');
        if args_prob > 0.0 && self.rng.random_bool(args_prob) {
            self.write_annotation_arg(out);
        }
        _ = out.write_char(')');
        _ = out.write_char(' ');
        self.at_line_start = false;
    }

    fn write_annotation_arg(&mut self, out: &mut impl Write) {
        match self.rng.random_range(0..4) {
            0 => {
                _ = out.write_str("TRUE");
            }
            1 => {
                _ = out.write_str("FALSE");
            }
            2 => {
                _ = write!(out, "{}", self.rng.random_range(0..100));
            }
            _ => {
                let s = ["key", "value", "name", "id"][self.rng.random_range(0..4)];
                _ = out.write_char('"');
                _ = out.write_str(s);
                _ = out.write_char('"');
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_grammar() -> &'static str {
        r#"{
            "terminals": {
                "ident": { "type": "identifier" },
                "number": { "type": "integer" }
            },
            "rules": {
                "specification": {
                    "seq": ["definition*"]
                },
                "definition": {
                    "choice": ["module_dcl", "const_dcl"]
                },
                "module_dcl": {
                    "seq": ["'module'", "ident", "'{'", "definition*", "'}'", "';'"]
                },
                "const_dcl": {
                    "seq": ["'const'", "'long'", "ident", "'='", "number", "';'"]
                }
            },
            "annotations": {
                "probability": 0.0,
                "names": ["key", "id"],
                "args_probability": 0.3
            },
            "start": "specification"
        }"#
    }

    #[test]
    fn test_reproducible_with_seed() {
        let grammar = Grammar::from_json(simple_grammar()).unwrap();
        let config = FuzzerConfig {
            max_depth: 5,
            max_repetitions: 2,
            seed: Some(12345),
            ..Default::default()
        };

        let mut fuzzer1 = Fuzzer::new(&grammar, config.clone());
        let output1 = fuzzer1.generate();

        let mut fuzzer2 = Fuzzer::new(&grammar, config);
        let output2 = fuzzer2.generate();

        assert_eq!(
            output1.source, output2.source,
            "same seed should produce same output"
        );
        assert_eq!(
            output1.token_count, output2.token_count,
            "same seed should produce same token count"
        );
    }

    #[test]
    fn test_with_annotations() {
        let grammar = Grammar::from_json(
            r#"{
            "terminals": {
                "ident": { "type": "identifier" }
            },
            "rules": {
                "start": {
                    "seq": ["'struct'", "ident", "'{'", "'}'", "';'"]
                }
            },
            "annotations": {
                "probability": 1.0,
                "names": ["foo", "bar"],
                "args_probability": 0.5
            },
            "start": "start"
        }"#,
        )
        .unwrap();

        let config = FuzzerConfig {
            max_depth: 5,
            seed: Some(42),
            annotation_probability: Some(1.0),
            ..Default::default()
        };
        let mut fuzzer = Fuzzer::new(&grammar, config);
        let output = fuzzer.generate();
        assert!(
            output.source.contains('@'),
            "expected annotations in output"
        );
        assert!(output.token_count > 0, "expected non-zero token count");
    }
}
