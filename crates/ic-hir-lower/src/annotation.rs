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

use ic_hir::hir::{Ann, AnnArg, AnnParam, DefFlags, DefId, DefKind, Ident, Ty, TyKind};
use ic_hir::scope::ScopeId;
use ic_syntax::util::{path_name, path_span};
use ic_syntax::{Annotation, AnnotationArg};

use crate::LoweringContext;
use crate::eval::ConstEvaluator;

fn evaluator_for_scope(
    ctx: &mut LoweringContext,
    scope: ScopeId,
    def_id: Option<DefId>,
) -> ConstEvaluator<'_> {
    if let Some(ann_def_id) = def_id
        && let Some(ann_scope) = ctx.context.scopes.find_scope_for_def(ann_def_id)
    {
        ConstEvaluator::with_annotation_scope(ctx, scope, ann_scope)
    } else {
        ConstEvaluator::new(ctx, scope)
    }
}

/// Convert AST annotations to HIR annotations.
pub fn convert_annotations(
    ctx: &mut LoweringContext,
    ast_annotations: &[Annotation],
    scope: ScopeId,
) -> Vec<Ann> {
    convert_annotations_for(ctx, ast_annotations, scope, None)
}

/// Convert AST annotations to HIR annotations, evaluating untyped arguments
/// against the type of the annotated item.
pub fn convert_annotations_for(
    ctx: &mut LoweringContext,
    ast_annotations: &[Annotation],
    scope: ScopeId,
    target: Option<&Ty>,
) -> Vec<Ann> {
    ast_annotations
        .iter()
        .map(|ann_appl| convert_annotation(ctx, ann_appl, scope, target))
        .collect()
}

/// Convert a single AST annotation to HIR annotation.
fn convert_annotation(
    ctx: &mut LoweringContext,
    ann_appl: &Annotation,
    scope: ScopeId,
    target: Option<&Ty>,
) -> Ann {
    let start = if ann_appl.path.leading_colons.is_some() {
        ctx.context.root_scope()
    } else {
        scope
    };

    // Try to resolve the annotation path
    let def_id = crate::resolve::resolve_annotation(&ctx.context, start, &ann_appl.path);
    let name = path_name(&ann_appl.path);

    // If we found something, verify it's an annotation
    let def_id = if let Some(id) = def_id {
        let def = ctx.context.definitions.get(id);
        if matches!(def.kind, DefKind::Annotation(_)) {
            Some(id)
        } else {
            ctx.diagnostics.error(
                format!("`{}` is not an annotation", path_name(&ann_appl.path)),
                ic_diagnostic::Label::new(ann_appl.span)
                    .message("expected an annotation definition"),
            );
            None
        }
    } else {
        None
    };

    // Convert annotation arguments
    let span = path_span(&ann_appl.path);
    let args = convert_annotation_args(ctx, &ann_appl.arguments, def_id, scope, span, target);

    Ann {
        ident: Ident { name, span },
        def_id,
        args,
    }
}

/// Convert annotation arguments.
fn convert_annotation_args(
    ctx: &mut LoweringContext,
    ast_args: &[AnnotationArg],
    def_id: Option<DefId>,
    scope: ScopeId,
    ann_span: ic_syntax::Span,
    target: Option<&Ty>,
) -> Vec<AnnArg> {
    // Get the annotation definition parameters if available
    let ann_params = get_annotation_params(ctx, def_id);
    let target = target.filter(|_| is_type_dependent(ctx, def_id));

    // Process arguments based on whether we have named or positional args
    let has_named_args = ast_args.iter().any(|arg| arg.name.is_some());

    if has_named_args {
        process_named_arguments(
            ctx,
            ast_args,
            ann_params.as_ref(),
            def_id,
            scope,
            ann_span,
            target,
        )
    } else {
        process_positional_arguments(
            ctx,
            ast_args,
            ann_params.as_ref(),
            def_id,
            scope,
            ann_span,
            target,
        )
    }
}

/// Whether the value of an annotation is a value of the annotated item's type.
fn is_type_dependent(ctx: &LoweringContext, def_id: Option<DefId>) -> bool {
    let Some(def_id) = def_id else {
        return false;
    };

    ctx.context
        .type_of(def_id)
        .flags
        .contains(DefFlags::IS_BUILTIN)
}

/// Get annotation parameters from definition if available.
fn get_annotation_params(ctx: &LoweringContext, def_id: Option<DefId>) -> Option<Vec<AnnParam>> {
    def_id.and_then(|id| {
        let def = ctx.context.definitions.get(id);
        if let DefKind::Annotation(ann_ty) = &def.kind {
            Some(ann_ty.params.clone())
        } else {
            None
        }
    })
}

/// Process named arguments for annotations.
fn process_named_arguments(
    ctx: &mut LoweringContext,
    ast_args: &[AnnotationArg],
    ann_params: Option<&Vec<AnnParam>>,
    def_id: Option<DefId>,
    scope: ScopeId,
    ann_span: ic_syntax::Span,
    target: Option<&Ty>,
) -> Vec<AnnArg> {
    let mut args = Vec::new();

    if let Some(params) = ann_params {
        // Collect all provided named arguments
        let named_args = collect_named_arguments(ctx, ast_args);

        // Process each parameter and match with provided arguments
        for param in params {
            if let Some(arg) =
                process_named_parameter(ctx, param, &named_args, def_id, scope, target)
            {
                args.push(arg);
            } else if let Some(arg) = create_default_argument(param) {
                args.push(arg);
            } else {
                report_missing_parameter(ctx, param, ann_span);
            }
        }

        // Check for unknown parameters
        validate_unknown_parameters(ctx, &named_args, params);
    } else {
        // No annotation definition available, just process named args as-is
        args = process_unvalidated_named_arguments(ctx, ast_args, def_id, scope);
    }

    args
}

/// Collect named arguments into a map.
fn collect_named_arguments<'a>(
    ctx: &mut LoweringContext,
    ast_args: &'a [AnnotationArg],
) -> std::collections::HashMap<String, &'a AnnotationArg> {
    let mut named_args = std::collections::HashMap::new();
    for arg in ast_args {
        if let Some(ref name) = arg.name {
            named_args.insert(name.name.clone(), arg);
        } else {
            ctx.diagnostics.error(
                "cannot mix named and positional annotation arguments".to_string(),
                ic_diagnostic::Label::new(arg.span).message("this argument should be named"),
            );
        }
    }
    named_args
}

/// Process a single named parameter.
fn process_named_parameter(
    ctx: &mut LoweringContext,
    param: &AnnParam,
    named_args: &std::collections::HashMap<String, &AnnotationArg>,
    def_id: Option<DefId>,
    scope: ScopeId,
    target: Option<&Ty>,
) -> Option<AnnArg> {
    if let Some(arg) = named_args.get(&param.ident.name) {
        let ty = argument_ty(ctx, param, arg, target).clone();
        let mut evaluator = evaluator_for_scope(ctx, scope, def_id);

        evaluator
            .eval_for_type(&arg.value, &ty)
            .map(|value| AnnArg {
                ident: Ident {
                    name: param.ident.name.clone(),
                    span: arg.span,
                },
                value,
                ty: Some(ty),
            })
    } else {
        None
    }
}

/// Pick the type an argument is evaluated against.
///
/// An `any` parameter carries no type information, so an initializer list can
/// only be shaped against the type of the annotated item. Scalar arguments stay
/// untyped, leaving compatibility checks to the `default-type-mismatch` lint.
fn argument_ty<'t>(
    ctx: &LoweringContext,
    param: &'t AnnParam,
    arg: &AnnotationArg,
    target: Option<&'t Ty>,
) -> &'t Ty {
    let Some(target) = target else {
        return &param.ty;
    };

    if !matches!(ctx.context.resolve_ty(&param.ty).kind, TyKind::Any) {
        return &param.ty;
    }

    match &arg.value.value {
        ic_syntax::ExprKind::InitList(_) => target,
        _ => &param.ty,
    }
}

/// Create default argument for a parameter with default value.
fn create_default_argument(param: &AnnParam) -> Option<AnnArg> {
    param.default.as_ref().map(|default_value| AnnArg {
        ident: param.ident.clone(),
        value: default_value.clone(),
        ty: Some(param.ty.clone()),
    })
}

/// Report missing required parameter.
fn report_missing_parameter(
    ctx: &mut LoweringContext,
    param: &AnnParam,
    ann_span: ic_syntax::Span,
) {
    ctx.diagnostics.error(
        format!(
            "missing required annotation parameter `{}`",
            param.ident.name
        ),
        ic_diagnostic::Label::new(ann_span).message("annotation parameter missing"),
    );
}

/// Validate that there are no unknown parameters.
fn validate_unknown_parameters(
    ctx: &mut LoweringContext,
    named_args: &std::collections::HashMap<String, &AnnotationArg>,
    params: &[AnnParam],
) {
    for (name, arg) in named_args {
        if !params.iter().any(|p| &p.ident.name == name) {
            ctx.diagnostics.error(
                format!("unknown annotation parameter `{name}`"),
                ic_diagnostic::Label::new(arg.span)
                    .message("no such parameter in annotation definition"),
            );
        }
    }
}

/// Process named arguments without validation.
fn process_unvalidated_named_arguments(
    ctx: &mut LoweringContext,
    ast_args: &[AnnotationArg],
    def_id: Option<DefId>,
    scope: ScopeId,
) -> Vec<AnnArg> {
    let mut args = Vec::new();
    for arg in ast_args {
        let mut evaluator = evaluator_for_scope(ctx, scope, def_id);

        if let Some(value) = evaluator.eval_annotation_arg(&arg.value) {
            let ident = arg.name.clone().unwrap_or_else(|| Ident {
                name: String::new(),
                span: arg.span,
            });
            args.push(AnnArg {
                ident,
                value,
                ty: None,
            });
        }
    }
    args
}

/// Process positional arguments for annotations.
fn process_positional_arguments(
    ctx: &mut LoweringContext,
    ast_args: &[AnnotationArg],
    ann_params: Option<&Vec<AnnParam>>,
    def_id: Option<DefId>,
    scope: ScopeId,
    ann_span: ic_syntax::Span,
    target: Option<&Ty>,
) -> Vec<AnnArg> {
    let mut args = Vec::new();

    if let Some(params) = ann_params {
        // Validate positional argument rules
        validate_positional_rules(ctx, ast_args, params, ann_span);

        // Process each positional argument
        for (i, arg) in ast_args.iter().enumerate() {
            if let Some(param) = params.get(i)
                && let Some(processed_arg) =
                    evaluate_argument(ctx, arg, param, def_id, scope, target)
            {
                args.push(processed_arg);
            }
        }

        // Add default values for missing parameters
        add_missing_defaults(ctx, &mut args, params, ast_args.len(), ann_span);
    } else {
        // No annotation definition available, just process as-is
        args = process_unvalidated_positional_arguments(ctx, ast_args, def_id, scope);
    }

    args
}

/// Validate rules for positional arguments.
fn validate_positional_rules(
    ctx: &mut LoweringContext,
    ast_args: &[AnnotationArg],
    params: &[AnnParam],
    ann_span: ic_syntax::Span,
) {
    // Rule 1: If multiple arguments are provided, all must be named
    // Exception: If the annotation has only one member without a default value
    let params_without_default: Vec<_> = params.iter().filter(|p| p.default.is_none()).collect();

    if ast_args.len() > 1 && params_without_default.len() > 1 {
        ctx.diagnostics.error(
            "positional arguments not allowed for multi-parameter annotations".to_string(),
            ic_diagnostic::Label::new(ann_span).message("use named arguments instead"),
        );
    }

    // Validate we don't have too many arguments
    if ast_args.len() > params.len() {
        ctx.diagnostics.error(
            format!(
                "too many arguments for annotation: expected at most {}, found {}",
                params.len(),
                ast_args.len()
            ),
            ic_diagnostic::Label::new(ast_args[params.len()].span).message("extra argument"),
        );
    }
}

/// Evaluate a single argument with type checking.
fn evaluate_argument(
    ctx: &mut LoweringContext,
    arg: &AnnotationArg,
    param: &AnnParam,
    def_id: Option<DefId>,
    scope: ScopeId,
    target: Option<&Ty>,
) -> Option<AnnArg> {
    let ty = argument_ty(ctx, param, arg, target).clone();
    let mut evaluator = evaluator_for_scope(ctx, scope, def_id);

    evaluator
        .eval_for_type(&arg.value, &ty)
        .map(|value| AnnArg {
            ident: Ident {
                name: String::new(), // Positional arguments have empty names
                span: arg.span,
            },
            value,
            ty: Some(ty),
        })
}

/// Add default values for missing parameters.
fn add_missing_defaults(
    ctx: &mut LoweringContext,
    args: &mut Vec<AnnArg>,
    params: &[AnnParam],
    provided_count: usize,
    ann_span: ic_syntax::Span,
) {
    for param in params.iter().skip(provided_count) {
        if let Some(ref default_value) = param.default {
            args.push(AnnArg {
                ident: param.ident.clone(),
                value: default_value.clone(),
                ty: Some(param.ty.clone()),
            });
        } else if provided_count == 0 {
            // Special case: if no arguments provided and param has no default
            ctx.diagnostics.error(
                format!(
                    "missing required annotation parameter `{}`",
                    param.ident.name
                ),
                ic_diagnostic::Label::new(ann_span).message("annotation requires arguments"),
            );
        }
    }
}

/// Process positional arguments without validation.
fn process_unvalidated_positional_arguments(
    ctx: &mut LoweringContext,
    ast_args: &[AnnotationArg],
    def_id: Option<DefId>,
    scope: ScopeId,
) -> Vec<AnnArg> {
    let mut args = Vec::new();
    for arg in ast_args {
        let mut evaluator = evaluator_for_scope(ctx, scope, def_id);

        if let Some(value) = evaluator.eval_annotation_arg(&arg.value) {
            args.push(AnnArg {
                ident: Ident {
                    name: String::new(),
                    span: arg.span,
                },
                value,
                ty: None,
            });
        }
    }
    args
}
