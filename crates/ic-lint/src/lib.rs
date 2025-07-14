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

//! # ic-lint
//!
//! A linting framework for IDL files that checks for syntax errors, semantic
//! issues, and style violations.
//!
//! ## Overview
//!
//! The lint system is organized into categories:
//!
//! - **Syntax**: Hard errors for malformed IDL constructs
//! - **Semantic**: Errors for semantically invalid constructs
//! - **Pedantic**: Warnings for non-standard language extensions
//! - **Unsupported**: Warnings for unsupported language features
//! - **Annotation**: Warnings for annotation usage issues
//! - **Deprecated**: Warnings for deprecated language features
//!
//! ## Usage
//!
//! ```no_run
//! use ic_lint::{lint_syntax, lint_hir};
//! use ic_vfs::SourceMap;
//! use ic_syntax::Item;
//!
//! // Assume we have a parsed AST
//! let vfs = SourceMap::default();
//! let ast: Vec<Item> = vec![];
//!
//! // Run AST-based lints
//! let report = lint_syntax(&ast, &vfs);
//! assert!(report.errors.is_empty());
//! assert!(report.warnings.is_empty());
//! ```
//!
//! ## Configuration
//!
//! Lints can be configured using [`LintConfig`]:
//!
//! ```no_run
//! use ic_lint::{LintConfig, Category, Level, lint_syntax_with_config};
//! # use ic_vfs::SourceMap;
//! # use ic_syntax::Item;
//! # let vfs = SourceMap::default();
//! # let ast: Vec<Item> = vec![];
//!
//! let mut config = LintConfig::new();
//! config.set_category_level(Category::Pedantic, Level::Error);
//! config.set_lint_level("null", Level::Warning);
//!
//! let report = lint_syntax_with_config(&ast, &vfs, &config);
//! ```
//!
//! ## Writing New Lints
//!
//! To create a new lint:
//!
//! 1. Create a new module in the appropriate category directory
//! 2. Define a struct that implements the [`Lint`] trait
//! 3. Implement `Visitor` for AST lints or `ic_hir::visit::Visitor` for HIR lints
//! 4. Register the lint in `lint_syntax()` or `lint_hir()`
//! 5. Add the lint name to `all_lint_names()`
//!
//! Example:
//!
//! ```ignore
//! use ic_syntax::visit::{Visitor, walk_tree};
//! use crate::{Lint, LintCtx, Category, lint_impl};
//!
//! pub struct MyLint<'a> {
//!     ctx: &'a LintCtx<'a>,
//! }
//!
//! impl<'a> Visitor<'a> for MyLint<'a> {
//!     // Implement visitor methods
//! }
//!
//! lint_impl!(MyLint, "my_lint", Category::Pedantic);
//! ```

use std::cell::RefCell;

// Re-export Level for external use
pub use ic_diagnostic::Level;
use ic_diagnostic::{Diag, Label, level_span};
use ic_syntax::{Item, Span};
use ic_vfs::SourceMap;

mod annotation;
mod pedantic;
mod semantic;
mod syntax;
mod unsupported;

mod iter;

use std::collections::HashMap;

/// Helper macro to simplify lint implementation.
///
/// Example:
/// ```ignore
/// lint_impl! {
///     name: "lowercase_bool",
///     category: Category::Pedantic,
///     message: "lowercase boolean literals are an InterCOM extension",
/// }
/// ```
#[macro_export]
macro_rules! lint_impl {
    (
        name: $name:expr,
        category: $cat:expr,
        $(,)?
    ) => {
        fn name() -> &'static str {
            $name
        }

        fn category() -> $crate::Category {
            $cat
        }
    };
}

/// The supported lint categories.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    /// Annotation-related lints
    Annotation,

    /// Deprecated language items
    Deprecated,

    /// Unsupported language items
    Unsupported,

    /// Lint for language extensions
    Pedantic,

    // Syntax errors or other semantic issues that should always be hard errors
    Syntax,

    // Semantic validation that should always be hard errors
    Semantic,
}

/// Configuration for lint levels.
#[must_use]
#[derive(Debug, Default)]
pub struct LintConfig {
    /// Maps categories to their configured level.
    pub category_levels: HashMap<Category, Level>,
    /// Maps specific lint names to their configured level.
    pub lint_levels: HashMap<&'static str, Level>,
}

impl LintConfig {
    /// Create a new default lint configuration.
    pub fn new() -> Self {
        let mut config = Self::default();
        // Default levels for each category
        config
            .category_levels
            .insert(Category::Syntax, Level::Error);
        config
            .category_levels
            .insert(Category::Semantic, Level::Error);
        config
            .category_levels
            .insert(Category::Annotation, Level::Warning);
        config
            .category_levels
            .insert(Category::Pedantic, Level::Warning);
        config
            .category_levels
            .insert(Category::Unsupported, Level::Warning);
        config
            .category_levels
            .insert(Category::Deprecated, Level::Warning);
        config
    }

    /// Set the level for an entire category.
    pub fn set_category_level(&mut self, category: Category, level: Level) {
        self.category_levels.insert(category, level);
    }

    /// Set the level for a specific lint.
    pub fn set_lint_level(&mut self, lint_name: &str, level: Level) {
        // We need to find the static string that matches this lint name
        if let Some(&static_name) = all_lint_names().iter().find(|&&n| n == lint_name) {
            self.lint_levels.insert(static_name, level);
        }
    }

    /// Get the configured level for a lint.
    pub fn get_level(&self, lint_name: &'static str, category: Category) -> Level {
        // Specific lint level overrides category level
        self.lint_levels
            .get(lint_name)
            .or_else(|| self.category_levels.get(&category))
            .copied()
            .unwrap_or(Level::Warning)
    }
}

#[derive(Debug)]
pub struct LintCtx<'a> {
    vfs: &'a SourceMap,
    warnings: RefCell<Vec<Diag>>,
    errors: RefCell<Vec<Diag>>,
    config: &'a LintConfig,
}

impl LintCtx<'_> {
    /// Emit a diagnostic.
    ///
    /// Diagnostics will be collected and emitted after all lints have been
    /// ran.
    pub fn report_error(&self, diag: Diag) {
        self.errors.borrow_mut().push(diag);
    }

    pub fn report_warn(&self, diag: Diag) {
        self.warnings.borrow_mut().push(diag);
    }

    /// Report a diagnostic with the appropriate level based on lint configuration.
    pub fn report(&self, lint_name: &'static str, category: Category, diag: Diag) {
        let level = self.config.get_level(lint_name, category);
        match level {
            Level::Error => self.report_error(diag),
            Level::Warning => self.report_warn(diag),
            Level::Disabled => {} // Don't emit disabled diagnostics
        }
    }

    /// Create a diagnostic with the appropriate level for this lint.
    /// Returns None if the lint is disabled.
    pub fn diag_span<S: Into<String>>(
        &self,
        lint_name: &'static str,
        category: Category,
        msg: S,
        label: Label,
    ) -> Option<Diag> {
        let level = self.config.get_level(lint_name, category);
        level_span(level, msg, label)
    }

    /// Returns a slice of the given span.
    ///
    /// # Panics
    ///
    /// Panics if the span crosses file boundaries (i.e., start and end are in
    /// different files). This is enforced by `span.range()` which contains a
    /// debug assertion.
    pub fn slice(&self, span: Span) -> &str {
        // span.range() will panic in debug builds if start.file_id != end.file_id
        &self.vfs.source_str(span.start.file_id)[span.range()]
    }
}

pub trait Lint<'a>: Sized {
    /// The unique name of this lint.
    fn name() -> &'static str;

    /// Category of the lint.
    fn category() -> Category;

    /// Runs the lint on the given AST.
    ///
    /// A lint should never fail in a way that prevents further traversal. Any
    /// potential errors should be gracefully ignored.
    fn check(_ctx: &'a LintCtx<'_>, _ast: &[Item]) {}

    /// Runs the lint on the given HIR.
    ///
    /// A lint should never fail in a way that prevents further traversal. Any
    /// potential errors should be gracefully ignored.
    fn check_hir(_ctx: &'a LintCtx<'_>, _hir: &ic_hir::ResolvedGraph) {}
}

#[must_use]
#[derive(Debug)]
pub struct Report {
    pub errors: Vec<Diag>,
    pub warnings: Vec<Diag>,
}

/// Returns all known lint names for validation.
#[must_use]
pub fn all_lint_names() -> Vec<&'static str> {
    vec![
        // Annotation lints
        "annotated_decl",
        // Pedantic lints
        "ambiguous_precedence",
        "array_param",
        "assign_expr",
        "bitmask_ann",
        "complex_lit",
        "complex_key",
        "empty_mod",
        "invalid_array_size",
        "lowercase_bool",
        "null",
        "omitted_in",
        "scoped_lit",
        // Semantic lints
        "bit_bound",
        "circular_inheritance",
        "deprecated",
        "duplicate_annotations",
        "duplicate_case_labels",
        "invalid_enum_value",
        "keywords",
        "oneway",
        "range_bound",
        "redundant_inheritance",
        "unnamed_args",
        "unreachable_union_cases",
        "zero_bound",
        // Syntax lints
        "ann_members",
        "ascii",
        "empty",
        "sanity",
        // Unsupported lints
        "items",
        // "proto", // Commented out - too restrictive for non-proto3 IDL
    ]
}

/// Normalize a lint name by replacing dashes with underscores.
#[must_use]
pub fn normalize_lint_name(name: &str) -> String {
    name.replace('-', "_")
}

/// Traverses the AST and produces diagnostics for all enabled lints.
///
/// Lints that operate on the AST are mostly syntactic. Other lints that
/// require more in-depth semantic analysis is typically done on the HIR with
/// [`lint_hir`].
pub fn lint_syntax(tree: &[Item], vfs: &SourceMap) -> Report {
    lint_syntax_with_config(tree, vfs, &LintConfig::new())
}

/// Traverses the AST with a custom lint configuration.
pub fn lint_syntax_with_config(tree: &[Item], vfs: &SourceMap, config: &LintConfig) -> Report {
    let ctx = LintCtx {
        vfs,
        warnings: RefCell::default(),
        errors: RefCell::default(),
        config,
    };

    let lints = &[
        annotation::decl::AnnotatedDecl::check,
        pedantic::ambiguous_precedence::AmbiguousPrecedence::check,
        pedantic::array_param::ArrayParam::check,
        pedantic::assign_expr::AssignExpr::check,
        pedantic::bitmask_ann::BitmaskAnn::check,
        pedantic::complex_lit::ComplexLit::check,
        pedantic::empty_mod::EmptyMod::check,
        pedantic::invalid_array_size::InvalidArraySize::check,
        pedantic::lowercase_bool::LowercaseBool::check,
        pedantic::null::NullVariant::check,
        pedantic::omitted_in::OmittedIn::check,
        pedantic::scoped_lit::ScopedLit::check,
        semantic::duplicate_annotations::DuplicateAnnotations::check,
        semantic::keywords::KwIdent::check,
        semantic::oneway::NonVoidOneway::check,
        semantic::redundant_inheritance::RedundantInheritance::check,
        syntax::ann_members::AnnMembers::check,
        syntax::ascii::AsciiIdent::check,
        syntax::empty::EmptyTypes::check,
        syntax::sanity::Sanity::check,
        unsupported::items::Unsupported::check,
    ];

    for check in lints {
        check(&ctx, tree);
    }

    Report {
        errors: ctx.errors.take(),
        warnings: ctx.warnings.take(),
    }
}

/// Set of lints that operates on the HIR.
pub fn lint_hir(hir: &ic_hir::ResolvedGraph, vfs: &SourceMap) -> Report {
    lint_hir_with_config(hir, vfs, &LintConfig::new())
}

/// Set of lints that operates on the HIR with a custom configuration.
pub fn lint_hir_with_config(
    hir: &ic_hir::ResolvedGraph,
    vfs: &SourceMap,
    config: &LintConfig,
) -> Report {
    let ctx = LintCtx {
        vfs,
        warnings: RefCell::default(),
        errors: RefCell::default(),
        config,
    };

    let lints = &[
        pedantic::complex_key::ComplexMapKey::check_hir,
        semantic::bit_bound::BitBound::check_hir,
        semantic::circular_inheritance::CircularInheritance::check_hir,
        semantic::deprecated::Deprecated::check_hir,
        semantic::duplicate_case_labels::DuplicateCaseLabels::check_hir,
        semantic::invalid_enum_value::InvalidEnumValue::check_hir,
        semantic::range_bound::RangeBound::check_hir,
        semantic::unnamed_args::UnnamedArgs::check_hir,
        semantic::unreachable_union_cases::UnreachableUnionCases::check_hir,
        semantic::zero_bound::ZeroBound::check_hir,
        // unsupported::proto::Proto::check_hir, // Commented out - too restrictive for non-proto3 IDL
    ];

    for check in lints {
        check(&ctx, hir);
    }

    Report {
        errors: ctx.errors.take(),
        warnings: ctx.warnings.take(),
    }
}
