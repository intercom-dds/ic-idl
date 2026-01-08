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
//! - **Extensions**: Warnings for non-standard language extensions
//! - **Pedantic**: Nitpicky style and quality warnings
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
//! 4. Add the lint to the appropriate section in the `define_lints!` macro
//!
//! Example:
//!
//! ```ignore
//! use ic_syntax::visit::{Visitor, walk_tree};
//! use crate::{Lint, LintCtx, Category};
//!
//! pub struct MyLint<'a> {
//!     ctx: &'a LintCtx<'a>,
//! }
//!
//! impl<'a> Lint<'a> for MyLint<'a> {
//!     fn name() -> &'static str {
//!         "my_lint"
//!     }
//!
//!     fn category() -> Category {
//!         Category::Pedantic
//!     }
//!
//!     fn check(ctx: &'a LintCtx<'_>, ast: &[Item]) {
//!         let visitor = MyLint { ctx };
//!         walk_tree(&visitor, ast);
//!     }
//! }
//!
//! impl<'a> Visitor<'a> for MyLint<'a> {
//!     // Implement visitor methods
//! }
//! ```

use std::cell::RefCell;

// Re-export Level for external use
pub use ic_diagnostic::Level;
use ic_diagnostic::{Diag, Label};
use ic_syntax::{Item, Span};
use ic_vfs::SourceMap;
use tracing::{debug, debug_span, trace};

mod annotation;
mod any_type;
mod deprecated;
mod extensions;
mod pedantic;
mod semantic;
mod syntax;
mod unsupported;

mod iter;

use std::collections::HashMap;

/// Create a diagnostic with a lint code.
fn lint_diag<S: Into<String>>(level: Level, code: &str, msg: S, label: Label) -> Option<Diag> {
    Diag::with_level(level, msg).map(|d| d.code(code).label(label))
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

    /// Lints for non-standard language extensions
    Extensions,

    /// Nitpicky style and quality warnings
    Pedantic,

    // Syntax errors or other semantic issues that should always be hard errors
    Syntax,

    // Semantic validation that should always be hard errors
    Semantic,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Annotation => "annotation",
            Self::Deprecated => "deprecated",
            Self::Extensions => "extensions",
            Self::Pedantic => "pedantic",
            Self::Semantic => "semantic",
            Self::Syntax => "syntax",
            Self::Unsupported => "unsupported",
        };
        write!(f, "{str}")
    }
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
            .insert(Category::Extensions, Level::Warning);
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
    /// Report a diagnostic with the appropriate level based on lint configuration.
    pub fn report(&self, lint_name: &'static str, category: Category, mut diag: Diag) {
        // Add the lint code to the diagnostic
        diag = diag.code(lint_name);

        // Semantic and Syntax lints are always errors
        let level = match category {
            Category::Semantic | Category::Syntax => Level::Error,
            _ => self.config.get_level(lint_name, category),
        };
        match level {
            Level::Error => {
                trace!(lint = lint_name, level = "error", "lint triggered");
                self.errors.borrow_mut().push(diag);
            }
            Level::Warning => {
                trace!(lint = lint_name, level = "warning", "lint triggered");
                self.warnings.borrow_mut().push(diag);
            }
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
        // Semantic and Syntax lints are always errors
        let level = match category {
            Category::Semantic | Category::Syntax => Level::Error,
            _ => self.config.get_level(lint_name, category),
        };

        // Apply color based on level
        let color = match level {
            Level::Error => ic_diagnostic::Color::Red,
            Level::Warning => ic_diagnostic::Color::Purple,
            Level::Disabled => return None,
        };

        lint_diag(level, lint_name, msg, label.color(color))
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

    /// Returns all recorded `#include` directives.
    pub fn includes(&self) -> &[ic_vfs::IncludeInfo] {
        self.vfs.includes()
    }
}

pub trait Lint<'a>: Sized {
    /// The unique name of this lint.
    fn name() -> &'static str;

    /// Category of the lint.
    fn category() -> Category;

    /// A short description of what this lint checks for (40-60 chars).
    fn description() -> &'static str;

    /// Check if this lint should run based on configuration.
    /// Semantic and Syntax category lints always run as they represent validation errors.
    #[must_use]
    fn should_run(config: &LintConfig) -> bool {
        match Self::category() {
            Category::Semantic | Category::Syntax => true,
            _ => config.get_level(Self::name(), Self::category()) != Level::Disabled,
        }
    }

    /// Runs the lint on the given AST.
    ///
    /// A lint should never fail in a way that prevents further traversal. Any
    /// potential errors should be gracefully ignored.
    fn check(_ctx: &'a LintCtx<'_>, _ast: &[Item]) {}

    /// Runs the lint on the given HIR.
    ///
    /// A lint should never fail in a way that prevents further traversal. Any
    /// potential errors should be gracefully ignored.
    fn check_hir(_ctx: &'a LintCtx<'a>, _hir: &'a ic_hir::ResolvedGraph) {}

    /// Helper method to report a diagnostic using this lint's name and category.
    fn report(ctx: &LintCtx<'_>, diag: Diag) {
        ctx.report(Self::name(), Self::category(), diag);
    }
}

#[must_use]
#[derive(Debug)]
pub struct Report {
    pub errors: Vec<Diag>,
    pub warnings: Vec<Diag>,
}

/// Macro to generate the lint list and functions.
macro_rules! define_lints {
    (
        syntax_lints: [
            $($syntax_lint:path,)*
        ],
        hir_lints: [
            $($hir_lint:path,)*
        ],
    ) => {
        /// Returns all known lint names for validation.
        #[must_use]
        pub fn all_lint_names() -> Vec<&'static str> {
            let mut names = vec![
                // Pseudo-lints handled outside the lint framework
                "ann-placement",
                $(<$syntax_lint>::name(),)*
                $(<$hir_lint>::name(),)*
            ];
            names.sort_unstable();
            names.dedup();
            names
        }

        /// Information about a lint.
        #[derive(Debug, Clone)]
        pub struct LintInfo {
            pub name: &'static str,
            pub category: Category,
            pub description: &'static str,
        }

        /// Returns information about all available lints.
        #[must_use]
        pub fn all_lints() -> Vec<LintInfo> {
            let mut lints = vec![
                // Pseudo-lints handled outside the lint framework
                LintInfo {
                    name: "ann-placement",
                    category: Category::Annotation,
                    description: "Annotations not attached to any declaration",
                },
                $(LintInfo {
                    name: <$syntax_lint>::name(),
                    category: <$syntax_lint>::category(),
                    description: <$syntax_lint>::description(),
                },)*
                $(LintInfo {
                    name: <$hir_lint>::name(),
                    category: <$hir_lint>::category(),
                    description: <$hir_lint>::description(),
                },)*
            ];
            lints.sort_by(|a, b| a.name.cmp(b.name));
            lints
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
            let _span = debug_span!("syntax_lints").entered();
            debug!("running syntax lints");
            let ctx = LintCtx {
                vfs,
                warnings: RefCell::default(),
                errors: RefCell::default(),
                config,
            };

            $(
                if <$syntax_lint>::should_run(config) {
                    trace!(lint = <$syntax_lint>::name(), "running lint");
                    <$syntax_lint>::check(&ctx, tree);
                }
            )*

            let errors = ctx.errors.into_inner();
            let warnings = ctx.warnings.into_inner();
            debug!(errors = errors.len(), warnings = warnings.len(), "complete");
            Report { errors, warnings }
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
            let _span = debug_span!("hir_lints").entered();
            debug!("running HIR lints");
            let ctx = LintCtx {
                vfs,
                warnings: RefCell::default(),
                errors: RefCell::default(),
                config,
            };

            $(
                if <$hir_lint>::should_run(config) {
                    trace!(lint = <$hir_lint>::name(), "running lint");
                    <$hir_lint>::check_hir(&ctx, hir);
                }
            )*

            let errors = ctx.errors.into_inner();
            let warnings = ctx.warnings.into_inner();
            debug!(errors = errors.len(), warnings = warnings.len(), "HIR lints complete");
            Report { errors, warnings }
        }
    };
}

// Define all lints in a single place
define_lints! {
    syntax_lints: [
        annotation::decl::AnnotatedDecl,
        extensions::array_param::ArrayParam,
        extensions::assign_expr::AssignExpr,
        extensions::bitmask_ann::BitmaskAnn,
        extensions::char_arithmetic::CharArithmetic,
        extensions::char_discriminator::CharDiscriminator,
        extensions::complex_lit::ComplexLit,
        extensions::empty_mod::EmptyMod,
        extensions::lowercase_bool::LowercaseBool,
        extensions::null::NullVariant,
        extensions::omitted_in::OmittedIn,
        extensions::scoped_lit::ScopedLit,
        extensions::shift_bound::ShiftBound,
        pedantic::ambiguous_precedence::AmbiguousPrecedence,
        semantic::ann_template::AnnTemplate,
        semantic::keywords::KwIdent,
        semantic::oneway::NonVoidOneway,
        semantic::redundant_inheritance::RedundantInheritance,
        syntax::ann_members::AnnMembers,
        syntax::ascii::AsciiIdent,
        syntax::empty::EmptyTypes,
        syntax::sanity::Sanity,
        unsupported::items::Unsupported,
    ],
    hir_lints: [
        annotation::deprecated_annotations::DeprecatedAnnotations,
        annotation::range_bound::RangeBound,
        annotation::unknown::UnknownAnnotation,
        any_type::AnyType,
        extensions::char_discriminator::CharDiscriminator,
        extensions::complex_key::ComplexMapKey,
        pedantic::invalid_array_size::InvalidArraySize,
        pedantic::large_union_variant::LargeUnionVariant,
        pedantic::prefer_enum_name::PreferEnumName,
        pedantic::unused_include::UnusedInclude,
        semantic::bit_bound::BitBound,
        semantic::conflicting_annotations::ConflictingAnnotations,
        semantic::derived_struct_key::DerivedStructKey,
        semantic::duplicate_annotations_hir::DuplicateAnnotations,
        semantic::duplicate_bounds::DuplicateBounds,
        semantic::duplicate_case_labels::DuplicateCaseLabels,
        semantic::duplicate_enum_values::DuplicateEnumValues,
        semantic::duplicate_name::DuplicateName,
        semantic::exhaustive_union_default::ExhaustiveUnionDefaultLint,
        semantic::initializer_list_size::InitializerListSize,
        semantic::invalid_annotation_target::InvalidAnnotationTarget,
        semantic::invalid_enum_literal::InvalidEnumLiteral,
        semantic::invalid_enum_value::InvalidEnumValue,
        semantic::invalid_inheritance::InvalidInheritance,
        semantic::multiple_default_cases::MultipleDefaultCases,
        semantic::recursive_type::RecursiveType,
        semantic::union_case_label_range::UnionCaseLabelRange,
        semantic::union_case_type_mismatch::UnionCaseTypeMismatch,
        semantic::union_key::UnionKey,
        semantic::unreachable_union_cases::UnreachableUnionCases,
        semantic::void_ty::VoidTy,
        semantic::zero_bound::ZeroBound,
        // unsupported::proto::Proto,
    ],
}
