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

use std::collections::HashMap;
use std::fmt;

use serde::Deserialize;

#[derive(Debug)]
pub enum GrammarError {
    /// JSON parsing failed
    ParseError(serde_json::Error),

    /// A rule references an undefined rule
    UndefinedRule { rule: String, referenced: String },

    /// A rule references an undefined terminal
    UndefinedTerminal { rule: String, terminal: String },

    /// The start rule is not defined
    UndefinedStart(String),

    /// A rule has no alternatives
    EmptyChoice(String),

    /// A rule has an empty sequence
    EmptySequence(String),
}

impl fmt::Display for GrammarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GrammarError::ParseError(e) => write!(f, "JSON parse error: {e}"),
            GrammarError::UndefinedRule { rule, referenced } => {
                write!(f, "rule '{rule}' references undefined rule '{referenced}'")
            }
            GrammarError::UndefinedTerminal { rule, terminal } => {
                write!(
                    f,
                    "rule '{rule}' references undefined terminal '{terminal}'"
                )
            }
            GrammarError::UndefinedStart(start) => {
                write!(f, "start rule '{start}' is not defined")
            }
            GrammarError::EmptyChoice(rule) => {
                write!(f, "rule '{rule}' has empty choice")
            }
            GrammarError::EmptySequence(rule) => {
                write!(f, "rule '{rule}' has empty sequence")
            }
        }
    }
}

impl std::error::Error for GrammarError {}

impl From<serde_json::Error> for GrammarError {
    fn from(e: serde_json::Error) -> Self {
        GrammarError::ParseError(e)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TerminalSpec {
    Identifier,
    Integer {
        #[serde(default = "default_bases")]
        bases: Vec<IntegerBase>,
    },
    Float,
    String,
    Char,
}

fn default_bases() -> Vec<IntegerBase> {
    vec![IntegerBase::Decimal]
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IntegerBase {
    Decimal,
    Hex,
    Octal,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RuleElement {
    /// Quoted literal like `'struct'` or rule reference like `foo*`
    Literal(String),
    Inline(Box<Rule>),
}

impl RuleElement {
    pub fn is_literal(&self) -> bool {
        match self {
            RuleElement::Literal(s) => s.starts_with('\'') && s.ends_with('\''),
            RuleElement::Inline(_) => false,
        }
    }

    pub fn literal_value(&self) -> Option<&str> {
        match self {
            RuleElement::Literal(s) if s.starts_with('\'') && s.ends_with('\'') => {
                Some(&s[1..s.len() - 1])
            }
            _ => None,
        }
    }

    pub fn rule_name(&self) -> Option<&str> {
        match self {
            RuleElement::Literal(s) if !s.starts_with('\'') => {
                let s = s
                    .trim_end_matches('*')
                    .trim_end_matches('+')
                    .trim_end_matches('?');
                Some(s)
            }
            _ => None,
        }
    }

    pub fn repetition(&self) -> Repetition {
        match self {
            RuleElement::Literal(s) => {
                if s.ends_with('*') {
                    Repetition::ZeroOrMore
                } else if s.ends_with('+') {
                    Repetition::OneOrMore
                } else if s.ends_with('?') {
                    Repetition::Optional
                } else {
                    Repetition::Once
                }
            }
            RuleElement::Inline(_) => Repetition::Once,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repetition {
    Once,

    // *
    ZeroOrMore,

    // +
    OneOrMore,

    // ?
    Optional,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Rule {
    Sequence {
        seq: Vec<RuleElement>,

        #[serde(default = "default_bias")]
        bias: f64,

        #[serde(default)]
        repetition_bias: Option<f64>,
    },
    Choice {
        choice: Vec<RuleElement>,

        #[serde(default = "default_bias")]
        bias: f64,

        #[serde(default)]
        repetition_bias: Option<f64>,
    },
    Optional {
        opt: Box<RuleElement>,
    },
}

fn default_bias() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnnotationConfig {
    #[serde(default)]
    pub probability: f64,

    #[serde(default)]
    pub names: Vec<String>,

    #[serde(default)]
    pub args_probability: f64,
}

impl Default for AnnotationConfig {
    fn default() -> Self {
        Self {
            probability: 0.0,
            names: vec![
                "key".to_string(),
                "id".to_string(),
                "optional".to_string(),
                "nested".to_string(),
            ],
            args_probability: 0.3,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Grammar {
    #[serde(default)]
    pub terminals: HashMap<String, TerminalSpec>,
    pub rules: HashMap<String, Rule>,

    #[serde(default)]
    pub annotations: AnnotationConfig,
    pub start: String,
}

impl Grammar {
    /// # Errors
    ///
    /// Returns an error if parsing or validation fails.
    pub fn from_json(json: &str) -> Result<Self, GrammarError> {
        let grammar: Grammar = serde_json::from_str(json)?;
        grammar.validate()?;
        Ok(grammar)
    }

    /// # Errors
    ///
    /// Returns an error if the grammar is invalid.
    pub fn validate(&self) -> Result<(), GrammarError> {
        if !self.rules.contains_key(&self.start) {
            return Err(GrammarError::UndefinedStart(self.start.clone()));
        }
        for (name, rule) in &self.rules {
            self.validate_rule(name, rule)?;
        }

        Ok(())
    }

    fn validate_rule(&self, name: &str, rule: &Rule) -> Result<(), GrammarError> {
        match rule {
            Rule::Sequence { seq, .. } => {
                if seq.is_empty() {
                    return Err(GrammarError::EmptySequence(name.to_string()));
                }
                for elem in seq {
                    self.validate_element(name, elem)?;
                }
            }
            Rule::Choice { choice, .. } => {
                if choice.is_empty() {
                    return Err(GrammarError::EmptyChoice(name.to_string()));
                }
                for elem in choice {
                    self.validate_element(name, elem)?;
                }
            }
            Rule::Optional { opt } => {
                self.validate_element(name, opt)?;
            }
        }
        Ok(())
    }

    fn validate_element(&self, rule_name: &str, elem: &RuleElement) -> Result<(), GrammarError> {
        match elem {
            RuleElement::Literal(s) => {
                if !s.starts_with('\'') {
                    let name = s
                        .trim_end_matches('*')
                        .trim_end_matches('+')
                        .trim_end_matches('?');
                    if !self.rules.contains_key(name) && !self.terminals.contains_key(name) {
                        return Err(GrammarError::UndefinedRule {
                            rule: rule_name.to_string(),
                            referenced: name.to_string(),
                        });
                    }
                }
            }
            RuleElement::Inline(inner_rule) => {
                self.validate_rule(rule_name, inner_rule)?;
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn get_rule(&self, name: &str) -> Option<&Rule> {
        self.rules.get(name)
    }

    #[must_use]
    pub fn get_rule_bias(&self, name: &str) -> f64 {
        self.rules.get(name).map_or(1.0, Rule::bias)
    }

    #[must_use]
    pub fn get_rule_repetition_bias(&self, name: &str) -> f64 {
        self.rules.get(name).map_or(1.0, Rule::repetition_bias)
    }

    #[must_use]
    pub fn get_terminal(&self, name: &str) -> Option<&TerminalSpec> {
        self.terminals.get(name)
    }

    #[must_use]
    pub fn is_terminal(&self, name: &str) -> bool {
        self.terminals.contains_key(name)
    }
}

impl Rule {
    pub fn bias(&self) -> f64 {
        match self {
            Rule::Optional { .. } => 1.0,
            Rule::Sequence { bias, .. } | Rule::Choice { bias, .. } => *bias,
        }
    }

    pub fn repetition_bias(&self) -> f64 {
        match self {
            Rule::Sequence {
                repetition_bias,
                bias,
                ..
            }
            | Rule::Choice {
                repetition_bias,
                bias,
                ..
            } => repetition_bias.unwrap_or(*bias),
            Rule::Optional { .. } => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_grammar() {
        let json = r#"{
            "terminals": {
                "ident": { "type": "identifier" },
                "number": { "type": "integer" }
            },
            "rules": {
                "start_rule": {
                    "seq": ["'module'", "ident", "';'"]
                }
            },
            "start": "start_rule"
        }"#;

        let grammar = Grammar::from_json(json).unwrap();
        assert_eq!(grammar.start, "start_rule");
        assert!(grammar.terminals.contains_key("ident"));
        assert!(grammar.rules.contains_key("start_rule"));
    }

    #[test]
    fn test_undefined_rule_error() {
        let json = r#"{
            "terminals": {},
            "rules": {
                "start_rule": {
                    "seq": ["undefined_rule"]
                }
            },
            "start": "start_rule"
        }"#;

        let result = Grammar::from_json(json);
        assert!(matches!(result, Err(GrammarError::UndefinedRule { .. })));
    }

    #[test]
    fn test_undefined_start_error() {
        let json = r#"{
            "terminals": {},
            "rules": {
                "some_rule": {
                    "seq": ["'foo'"]
                }
            },
            "start": "nonexistent"
        }"#;

        let result = Grammar::from_json(json);
        assert!(matches!(result, Err(GrammarError::UndefinedStart(_))));
    }

    #[test]
    fn test_choice_rule() {
        let json = r#"{
            "terminals": {},
            "rules": {
                "start_rule": {
                    "choice": ["'foo'", "'bar'"]
                }
            },
            "start": "start_rule"
        }"#;

        let grammar = Grammar::from_json(json).unwrap();
        match grammar.get_rule("start_rule") {
            Some(Rule::Choice { choice, .. }) => {
                assert_eq!(choice.len(), 2);
            }
            _ => panic!("expected choice rule"),
        }
    }

    #[test]
    fn test_repetition_parsing() {
        let elem = RuleElement::Literal("foo*".to_string());
        assert_eq!(elem.repetition(), Repetition::ZeroOrMore);
        assert_eq!(elem.rule_name(), Some("foo"));

        let elem = RuleElement::Literal("bar+".to_string());
        assert_eq!(elem.repetition(), Repetition::OneOrMore);
        assert_eq!(elem.rule_name(), Some("bar"));

        let elem = RuleElement::Literal("baz?".to_string());
        assert_eq!(elem.repetition(), Repetition::Optional);
        assert_eq!(elem.rule_name(), Some("baz"));

        let elem = RuleElement::Literal("qux".to_string());
        assert_eq!(elem.repetition(), Repetition::Once);
        assert_eq!(elem.rule_name(), Some("qux"));
    }

    #[test]
    fn test_literal_parsing() {
        let elem = RuleElement::Literal("'struct'".to_string());
        assert!(elem.is_literal());
        assert_eq!(elem.literal_value(), Some("struct"));
        assert_eq!(elem.rule_name(), None);
    }
}
