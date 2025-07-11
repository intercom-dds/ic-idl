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

#![allow(unused, clippy::doc_markdown)]

use std::path::PathBuf;
use std::collections::HashMap;

use ic_cli::Command;
use ic_cli::color::Colorize;
use ic_cli::convert::{self, ConvertError};
use ic_lint::{LintConfig, Category, Level};

use crate::warn;

#[derive(Command, Debug, Default)]
pub struct Warnings {
    /// Enable all warnings
    #[option(long)]
    all: bool,

    /// Suspicious annotation usage such as conflicting annotations
    #[option(long)]
    annotation: bool,

    /// Use of unknown annotations
    #[option(long)]
    unknown_annotation: bool,

    /// Language extensions or implementation-defined behavior
    #[option(long)]
    pedantic: bool,

    /// Upgrade warnings to errors
    #[option(long)]
    error: bool,
}

impl Warnings {
    /// Build a LintConfig from the warning flags, with specific lint overrides.
    pub fn to_lint_config(&self, specific_lints: &HashMap<String, bool>, error_lints: &HashMap<String, bool>) -> LintConfig {
        let mut config = LintConfig::new();

        // Start with all warnings disabled by default (unless -Wall is set)
        if !self.all {
            config.set_category_level(Category::Annotation, Level::Disabled);
            config.set_category_level(Category::Pedantic, Level::Disabled);
            config.set_category_level(Category::Unsupported, Level::Disabled);
            config.set_category_level(Category::Deprecated, Level::Disabled);
        }

        // Enable specific categories if requested
        if self.all || self.annotation || self.unknown_annotation {
            config.set_category_level(Category::Annotation, Level::Warning);
        }

        if self.all || self.pedantic {
            config.set_category_level(Category::Pedantic, Level::Warning);
        }

        // Apply specific lint settings
        for (lint_name, enabled) in specific_lints {
            let level = if *enabled {
                Level::Warning
            } else {
                Level::Disabled
            };
            config.set_lint_level(lint_name.as_str(), level);
        }

        // Apply lint-specific error settings
        for (lint_name, _) in error_lints {
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
            let lint_names: Vec<_> = config.lint_levels.keys().cloned().collect();
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
    pub cpp: CppOptions,

    #[option(section = "rust options")]
    pub rust: RustOptions,

    #[option(section = "python options")]
    pub python: PythonOptions,

    #[option(section = "idl options")]
    pub idl: IdlOptions,

    #[option(section = "backends")]
    pub codegen: CodegenOptions,
}

#[derive(Command, Debug, Default)]
pub struct CppOptions {
    /// Generate scoped enums
    #[option(long)]
    pub scoped_enums: bool,

    /// Use access functions instead of direct member access
    #[option(long)]
    pub access_functions: bool,

    /// Do not generate ostream operators for serialization
    #[option(long)]
    pub no_stream_op: bool,

    /// Generate formatting specializations for fmtlib
    #[option(long)]
    pub use_fmt: bool,

    /// Use <sym> as dllexport symbol
    #[option(long, arg = "sym")]
    pub dll_export: Option<String>,

    /// Use <ext> as file extension for C++ headers
    #[option(long, arg = "ext")]
    pub header_ext: Option<String>,

    /// Store header files inside a subfolder
    #[option(long, arg = "dir")]
    pub header_subfolder: Option<String>,
}

#[derive(Command, Debug, Default)]
pub struct RustOptions {
    /// Do not rename generated types
    #[option(long)]
    pub no_rename: bool,

    /// Annotate all types with `#[must_use]`
    #[option(long)]
    pub must_use: bool,
}

#[derive(Command, Debug, Default)]
pub struct PythonOptions {
    /// Rename all types to conform to PEP-8
    #[option(long)]
    pub use_pep8: bool,

    /// Postfix to use for global modules
    #[option(long)]
    pub global_postfix: Option<String>,
}

#[derive(Command, Debug, Default)]
pub struct IdlOptions {
    /// Output Doxygen-compatible IDL files
    #[option(long)]
    pub idl_doxygen: bool,

    /// Expand @DDSService interfaces
    #[option(long)]
    pub idl_expand: bool,
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
}

impl convert::Convert for Unstable {
    fn from_result(input: &[String]) -> convert::Result<Self> {
        let mut this = Self::default();
        for arg in input {
            match arg.as_str() {
                "ast-dump" => this.ast_dump = true,
                "hir-dump" => this.hir_dump = true,
                "ptree-dump" => this.ptree_dump = true,
                "help" => crate::unstable::unstable_help(),
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

/// Parsed warning configuration including specific lint settings
pub struct ParsedWarnings {
    pub warnings: Warnings,
    pub specific_lints: HashMap<String, bool>,
    pub error_lints: HashMap<String, bool>,
}

impl convert::Convert for Warnings {
    fn from_result(input: &[String]) -> convert::Result<Self> {
        let mut warnings = Self::default();
        let mut specific_lints = HashMap::new();
        let mut error_lints = HashMap::new();
        let known_lints = ic_lint::all_lint_names();

        for arg in input {
            // Handle error=lint_name syntax
            if let Some(lint_name) = arg.strip_prefix("error=") {
                let normalized = ic_lint::normalize_lint_name(lint_name);
                if known_lints.contains(&normalized.as_str()) {
                    error_lints.insert(normalized, true);
                } else {
                    warn!("unknown lint '{}'", format!("-Werror={}", lint_name).yellow());
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
                "unknown-annotation" => warnings.unknown_annotation = enabled,
                "pedantic" => warnings.pedantic = enabled,
                "error" => warnings.error = enabled,
                "help" => crate::unstable::warning_help(),
                _ => {
                    // Check if it's a specific lint name
                    let normalized = ic_lint::normalize_lint_name(arg);
                    if known_lints.contains(&normalized.as_str()) {
                        specific_lints.insert(normalized, enabled);
                    } else {
                        warn!("unknown warning '{}' (use -W help to see available warnings)", 
                              format!("-W{arg}").yellow());
                    }
                }
            }
        }
        
        // Store the specific lint settings in a global for later retrieval
        PARSED_WARNINGS.with(|p| {
            *p.borrow_mut() = Some((specific_lints, error_lints));
        });
        
        Ok(warnings)
    }
}

use std::cell::RefCell;

thread_local! {
    static PARSED_WARNINGS: RefCell<Option<(HashMap<String, bool>, HashMap<String, bool>)>> = RefCell::new(None);
}

/// Get the parsed warning settings
pub fn take_parsed_warnings() -> (HashMap<String, bool>, HashMap<String, bool>) {
    PARSED_WARNINGS.with(|p| {
        p.borrow_mut().take().unwrap_or_default()
    })
}
