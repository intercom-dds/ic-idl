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

#![allow(unused, clippy::doc_markdown)]

use std::collections::HashMap;
use std::path::PathBuf;

use ic_cli::Command;
use ic_cli::color::Colorize as _;
use ic_cli::convert::{self, ConvertError};
use ic_lint::{Category, Level, LintConfig};

#[derive(Command, Debug, Default)]
pub struct Warnings {
    /// Enable all warnings
    #[option(long)]
    all: bool,

    /// Suspicious annotation usage such as conflicting annotations
    #[option(long)]
    annotation: bool,

    /// Language extensions or implementation-defined behavior
    #[option(long)]
    pedantic: bool,

    /// Preprocessor-related warnings
    #[option(long)]
    preprocessor: bool,

    /// Upgrade warnings to errors
    #[option(long)]
    error: bool,

    /// Show help for warning options
    pub help: bool,

    specific_lints: HashMap<String, bool>,

    error_lints: HashMap<String, bool>,

    /// Unknown warnings that were encountered during parsing
    pub unknown_warnings: Vec<String>,
}

impl Warnings {
    /// Check if preprocessor warnings should be emitted.
    /// They are enabled by default unless explicitly disabled.
    #[must_use]
    pub fn preprocessor_enabled(&self) -> bool {
        self.preprocessor
    }

    /// Build a LintConfig from the warning flags.
    pub fn to_lint_config(&self) -> LintConfig {
        let mut config = LintConfig::new();

        // Start with all warnings disabled by default (unless -Wall is set)
        if !self.all {
            config.set_category_level(Category::Annotation, Level::Disabled);
            config.set_category_level(Category::Pedantic, Level::Disabled);
            config.set_category_level(Category::Unsupported, Level::Disabled);
            config.set_category_level(Category::Deprecated, Level::Disabled);
        }

        // Enable specific categories if requested
        if self.all || self.annotation {
            config.set_category_level(Category::Annotation, Level::Warning);
        }

        if self.all || self.pedantic {
            config.set_category_level(Category::Pedantic, Level::Warning);
        }

        // Apply specific lint settings
        for (lint_name, enabled) in &self.specific_lints {
            let level = if *enabled {
                Level::Warning
            } else {
                Level::Disabled
            };
            config.set_lint_level(lint_name.as_str(), level);
        }

        // Apply lint-specific error settings
        for lint_name in self.error_lints.keys() {
            config.set_lint_level(lint_name.as_str(), Level::Error);
        }

        // If -Werror is set, upgrade all enabled warnings to errors
        if self.error {
            // Upgrade category levels
            if config.category_levels.get(&Category::Annotation) == Some(&Level::Warning) {
                config.set_category_level(Category::Annotation, Level::Error);
            }
            if config.category_levels.get(&Category::Pedantic) == Some(&Level::Warning) {
                config.set_category_level(Category::Pedantic, Level::Error);
            }
            if config.category_levels.get(&Category::Unsupported) == Some(&Level::Warning) {
                config.set_category_level(Category::Unsupported, Level::Error);
            }
            if config.category_levels.get(&Category::Deprecated) == Some(&Level::Warning) {
                config.set_category_level(Category::Deprecated, Level::Error);
            }

            // Upgrade individual lint levels
            let lint_names: Vec<_> = config.lint_levels.keys().copied().collect();
            for lint_name in lint_names {
                if config.lint_levels.get(lint_name) == Some(&Level::Warning) {
                    config.set_lint_level(lint_name, Level::Error);
                }
            }
        }

        config
    }
}

/// Generic IDL code generator
#[derive(Command, Debug, Default)]
#[must_use]
pub struct Options {
    /// Only run the preprocessor
    #[option(short = 'E', long)]
    pub preprocessor_only: bool,

    /// Do not generate code for included files
    #[option(short = 'H', long)]
    pub no_header_follow: bool,

    /// Add directory to include search paths
    #[option(short = 'I', long, arg = "dir")]
    pub include: Vec<String>,

    /// Define preprocessor directive <def> to <val>
    #[option(short = 'D', long, arg = "def>=<val")]
    pub define: Vec<String>,

    /// Output list of files to be generated
    #[option(short, long)]
    pub list: bool,

    /// Erase output directories before emitting code
    #[option(long)]
    pub purge_dirs: bool,

    /// Do not parse Doxy-like comments
    #[option(long)]
    pub ignore_comments: bool,

    /// Enable a warning, see `-W help` for details
    #[option(short = 'W', arg = "lint")]
    pub warn: Warnings,

    /// Unstable flags, see `-Z help` for details
    #[option(short = 'Z', arg = "flag")]
    pub unstable: Unstable,

    /// Display version information
    #[option(short = 'V', long)]
    pub version: bool,

    #[option(positional)]
    pub files: Vec<PathBuf>,

    #[option(section = "c++ options")]
    pub cpp: ic_codegen_cxx::CppOptions,

    #[option(section = "rust options")]
    pub rust: ic_codegen_rust::RustOptions,

    #[option(section = "python options")]
    pub python: ic_codegen_python::PythonOptions,

    #[option(section = "idl options")]
    pub idl: ic_codegen_idl::IdlOptions,

    #[option(section = "backends")]
    pub codegen: CodegenOptions,
}

#[derive(Command, Debug, Default)]
#[allow(clippy::struct_field_names)]
pub struct CodegenOptions {
    /// Generate C++ files in <dir>
    #[option(long, arg = "dir")]
    pub cpp_out: Option<PathBuf>,

    /// Generate Rust files in <dir>
    #[option(long, arg = "dir")]
    pub rust_out: Option<PathBuf>,

    /// Generate Python files in <dir>
    #[option(long, arg = "dir")]
    pub python_out: Option<PathBuf>,

    /// Generate IDL files in <dir>
    #[option(long, arg = "dir")]
    pub idl_out: Option<PathBuf>,

    /// Generate Protobuf files in <dir>
    #[option(long, arg = "dir")]
    pub proto_out: Option<PathBuf>,

    /// Generate JSON files in <dir>
    #[option(long, arg = "dir")]
    pub json_out: Option<PathBuf>,

    /// Generate XML files in <dir>
    #[option(long, arg = "dir")]
    pub xml_out: Option<PathBuf>,
}

#[derive(Command, Debug, Default)]
pub struct Unstable {
    /// Dump out the AST exactly as it was parsed
    #[option(long)]
    pub ast_dump: bool,

    /// Dump out the type-resolved IR
    #[option(long)]
    pub hir_dump: bool,

    /// Print the ptree in a tree-like format
    #[option(long)]
    pub ptree_dump: bool,

    /// Show help for unstable options
    pub help: bool,
}

impl convert::Convert for Unstable {
    fn from_result(input: &[String]) -> convert::Result<Self> {
        let mut this = Self::default();
        for arg in input {
            match arg.as_str() {
                "ast-dump" => this.ast_dump = true,
                "hir-dump" => this.hir_dump = true,
                "ptree-dump" => this.ptree_dump = true,
                "help" => {
                    this.help = true;
                    return Ok(this);
                }
                _ => {
                    return Err(ConvertError::InvalidValue(format!(
                        "unknown unstable option '{}'",
                        format!("-Z{arg}").yellow(),
                    )));
                }
            }
        }
        Ok(this)
    }
}

impl convert::Convert for Warnings {
    fn from_result(input: &[String]) -> convert::Result<Self> {
        let mut warnings = Self {
            preprocessor: true,
            ..Self::default()
        };
        let known_lints = ic_lint::all_lint_names();

        for arg in input {
            // Handle error=lint_name syntax
            if let Some(lint_name) = arg.strip_prefix("error=") {
                if known_lints.contains(&lint_name) {
                    warnings.error_lints.insert(lint_name.to_string(), true);
                } else {
                    warnings
                        .unknown_warnings
                        .push(format!("-Werror={lint_name}"));
                }
                continue;
            }

            let (arg, enabled) = if let Some(arg) = arg.strip_prefix("no-") {
                (arg, false)
            } else {
                (arg.as_str(), true)
            };

            match arg {
                "all" => warnings.all = enabled,
                "annotation" => warnings.annotation = enabled,
                "pedantic" => warnings.pedantic = enabled,
                "preprocessor" => warnings.preprocessor = enabled,
                "error" => warnings.error = enabled,
                "help" => {
                    warnings.help = true;
                    return Ok(warnings);
                }
                _ => {
                    // Check if it's a specific lint name
                    if known_lints.contains(&arg) {
                        warnings.specific_lints.insert(arg.to_string(), enabled);
                    } else {
                        warnings.unknown_warnings.push(format!("-W{arg}"));
                    }
                }
            }
        }

        Ok(warnings)
    }
}
