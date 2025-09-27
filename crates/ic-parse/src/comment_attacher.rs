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

use ic_syntax::util::{decl_span, item_span};
use ic_syntax::{
    AnnotationAppl, AnnotationArg, AnnotationDef, AnnotationField, BitmaskDef, BitsetDef, EnumDef,
    ExceptDef, Expr, Ident, InterfaceDef, InterfaceMember, Item, Literal, LiteralValue, ModuleDef,
    Path, Span, StructDef, UnionDef, ValueElement, ValuetypeDef,
};

/// A comment with its location and content
#[derive(Clone, Debug)]
pub struct Comment {
    pub span: Span,
    pub text: String,
    pub is_trailing: bool,
}

/// Attaches comments to AST nodes by converting them to @doc annotations
pub struct CommentAttacher {
    /// All comments from the file
    comments: Vec<Comment>,
}

impl CommentAttacher {
    pub fn new(comments: Vec<Comment>) -> Self {
        Self { comments }
    }

    /// Attach comments to the AST
    pub fn attach(&mut self, mut tree: Vec<Item>) -> Vec<Item> {
        // Sort comments by position for efficient processing
        self.comments.sort_by_key(|c| c.span.start.offset);

        // Process the tree
        let mut processor = Processor {
            comments: &self.comments,
            comment_idx: 0,
        };

        let tree_len = tree.len();
        for i in 0..tree_len {
            processor.process_item(&mut tree[i]);

            // After processing each item, attach trailing comments up to the next item
            let next_pos = if i + 1 < tree_len {
                item_span(&tree[i + 1]).start.offset
            } else {
                u32::MAX
            };

            if let Some(annotations) = item_annotations_mut(&mut tree[i]) {
                processor.attach_trailing_comments_until(next_pos, annotations);
            }
        }

        tree
    }
}

struct Processor<'a> {
    comments: &'a [Comment],
    comment_idx: usize,
}

impl Processor<'_> {
    /// Process container items that have nested items (module, interface, annotation, valuetype)
    fn process_container<T>(
        &mut self,
        span: Span,
        annotations: &mut Vec<AnnotationAppl>,
        members: &mut [T],
        first_member_offset: Option<u32>,
        mut process_member: impl FnMut(&mut Self, &mut T),
    ) {
        self.attach_comments_before(span.start.offset, annotations);

        let first_offset = first_member_offset.unwrap_or(span.end.offset.saturating_sub(1));
        self.handle_inline_and_body_comments(first_offset, span.end.offset, annotations);

        for member in members {
            process_member(self, member);
        }

        self.attach_item_trailing_comment(span.end.offset, annotations);
    }
    /// Common pattern for processing items with members
    fn process_item_with_members<T>(
        &mut self,
        span: Span,
        annotations: &mut Vec<AnnotationAppl>,
        members: &mut [T],
        member_span: impl Fn(&T) -> Span,
        member_annotations: impl Fn(&mut T) -> &mut Vec<AnnotationAppl>,
        use_bounded_trailing: bool,
    ) {
        self.attach_comments_before(span.start.offset, annotations);

        let first_member_offset = members
            .first()
            .map_or(span.end.offset.saturating_sub(1), |m| {
                member_span(m).start.offset
            });

        self.handle_inline_and_body_comments(first_member_offset, span.end.offset, annotations);

        let container_end = span.end.offset;
        if use_bounded_trailing {
            let member_count = members.len();
            for i in 0..member_count {
                let member_sp = member_span(&members[i]);
                self.attach_comments_before(
                    member_sp.start.offset,
                    member_annotations(&mut members[i]),
                );

                // For bounded trailing, look up to the next member or container end
                let next_pos = if i + 1 < member_count {
                    member_span(&members[i + 1]).start.offset
                } else {
                    container_end
                };
                self.attach_trailing_comments_until(next_pos, member_annotations(&mut members[i]));
            }
        } else {
            let member_count = members.len();
            for i in 0..member_count {
                let span = member_span(&members[i]);
                self.attach_comments_before(span.start.offset, member_annotations(&mut members[i]));

                let next_pos = if i + 1 < member_count {
                    member_span(&members[i + 1]).start.offset
                } else {
                    span.end.offset
                };
                self.attach_trailing_comments_until(next_pos, member_annotations(&mut members[i]));
            }
        }

        self.attach_item_trailing_comment(span.end.offset, annotations);
    }
    fn process_item(&mut self, item: &mut Item) {
        match item {
            Item::ModuleValue(m) => self.process_module(m),
            Item::StructValue(s) => self.process_struct(s),
            Item::UnionValue(u) => self.process_union(u),
            Item::EnumValue(e) => self.process_enum(e),
            Item::InterfaceValue(i) => self.process_interface(i),
            Item::ExceptionValue(e) => self.process_exception(e),
            Item::BitmaskValue(b) => self.process_bitmask(b),
            Item::BitsetValue(b) => self.process_bitset(b),
            Item::AnnotationValue(a) => self.process_annotation(a),
            Item::ValuetypeValue(v) => self.process_valuetype(v),
            Item::ConstValue(c) => self.process_simple_item(c.span, &mut c.annotations),
            Item::AliasValue(a) => self.process_simple_item(a.span, &mut a.annotations),
            Item::DeclValue(_) => {}
        }
    }

    fn process_module(&mut self, module: &mut ModuleDef) {
        let first_offset = module
            .definitions
            .first()
            .map(|d| item_span(d).start.offset);

        self.process_container(
            module.span,
            &mut module.annotations,
            &mut module.definitions,
            first_offset,
            |processor, item| {
                processor.process_item(item);
            },
        );

        // Handle trailing comments between definitions
        let def_count = module.definitions.len();
        if def_count > 1 {
            for i in 0..def_count - 1 {
                let next_pos = item_span(&module.definitions[i + 1]).start.offset;
                if let Some(annotations) = item_annotations_mut(&mut module.definitions[i]) {
                    self.attach_trailing_comments_until(next_pos, annotations);
                }
            }
        }
    }

    fn process_struct(&mut self, s: &mut StructDef) {
        self.process_item_with_members(
            s.span,
            &mut s.annotations,
            &mut s.members,
            |member| member.span,
            |member| &mut member.annotations,
            true,
        );
    }

    fn process_union(&mut self, u: &mut UnionDef) {
        self.process_item_with_members(
            u.span,
            &mut u.annotations,
            &mut u.fields,
            |field| field.span,
            |field| &mut field.annotations,
            true,
        );
    }

    fn process_enum(&mut self, e: &mut EnumDef) {
        self.process_item_with_members(
            e.span,
            &mut e.annotations,
            &mut e.fields,
            |field| field.ident.span,
            |field| &mut field.annotations,
            false,
        );
    }

    fn process_exception(&mut self, e: &mut ExceptDef) {
        self.process_item_with_members(
            e.span,
            &mut e.annotations,
            &mut e.members,
            |member| member.span,
            |member| &mut member.annotations,
            true,
        );
    }

    fn process_bitmask(&mut self, b: &mut BitmaskDef) {
        self.process_item_with_members(
            b.span,
            &mut b.annotations,
            &mut b.bits,
            |bit| bit.span,
            |bit| &mut bit.annotations,
            false,
        );
    }

    fn process_bitset(&mut self, b: &mut BitsetDef) {
        self.process_item_with_members(
            b.span,
            &mut b.annotations,
            &mut b.fields,
            |field| field.span,
            |field| &mut field.annotations,
            true,
        );
    }

    fn process_interface(&mut self, i: &mut InterfaceDef) {
        let first_offset = first_interface_member_offset(&i.members);
        self.process_container(
            i.span,
            &mut i.annotations,
            &mut i.members,
            first_offset,
            |processor, member| {
                if let InterfaceMember::Item(item) = member {
                    processor.process_item(item);
                }
            },
        );
    }

    fn process_annotation(&mut self, a: &mut AnnotationDef) {
        self.attach_comments_before(a.span.start.offset, &mut a.annotations);

        let first_member_offset =
            first_annotation_param_offset(&a.params).unwrap_or(a.span.end.offset.saturating_sub(1));

        self.handle_inline_and_body_comments(
            first_member_offset,
            a.span.end.offset,
            &mut a.annotations,
        );

        let params_count = a.params.len();
        for i in 0..params_count {
            // Get next position before mutating current param
            let next_pos = if i + 1 < params_count {
                match &a.params[i + 1] {
                    AnnotationField::Item(item) => item_span(item).start.offset,
                    AnnotationField::Member(m) => m.span.start.offset,
                }
            } else {
                a.span.end.offset
            };

            match &mut a.params[i] {
                AnnotationField::Item(item) => self.process_item(item),
                AnnotationField::Member(member) => {
                    self.attach_comments_before(member.span.start.offset, &mut member.annotations);
                    self.attach_trailing_comments_until(next_pos, &mut member.annotations);
                }
            }
        }

        self.attach_item_trailing_comment(a.span.end.offset, &mut a.annotations);
    }

    fn process_valuetype(&mut self, v: &mut ValuetypeDef) {
        let first_offset = first_value_element_offset(&v.elements);
        self.process_container(
            v.span,
            &mut v.annotations,
            &mut v.elements,
            first_offset,
            |processor, element| {
                if let ValueElement::Item(item) = element {
                    processor.process_item(item);
                }
            },
        );
    }

    fn process_simple_item(&mut self, span: Span, annotations: &mut Vec<AnnotationAppl>) {
        self.attach_comments_before(span.start.offset, annotations);
        self.attach_trailing_comments(annotations);
    }

    fn handle_inline_and_body_comments(
        &mut self,
        first_member_offset: u32,
        end_offset: u32,
        annotations: &mut Vec<AnnotationAppl>,
    ) {
        // Only process trailing comments that are inside the container
        // Leading comments before members should be left for the members to process
        while self.comment_idx < self.comments.len() {
            let comment = &self.comments[self.comment_idx];

            if comment.span.start.offset >= end_offset {
                break;
            }

            // Stop processing when we reach the first member to avoid consuming
            // comments that belong to members
            if comment.span.start.offset >= first_member_offset {
                break;
            }

            // Only process trailing comments in this method
            // Leading comments will be handled by the members themselves
            if comment.is_trailing {
                annotations.push(comment_to_doc_annotation(comment));
                self.comment_idx += 1;
            } else {
                // Don't consume leading comments - they belong to members
                break;
            }
        }
    }

    fn attach_item_trailing_comment(
        &mut self,
        item_end: u32,
        annotations: &mut Vec<AnnotationAppl>,
    ) {
        // Look for trailing comments after the item (e.g., after semicolon)
        let mut temp_idx = self.comment_idx;
        while temp_idx < self.comments.len() {
            let comment = &self.comments[temp_idx];

            if comment.is_trailing && comment.span.start.offset > item_end {
                annotations.push(comment_to_doc_annotation(comment));
                self.comment_idx = temp_idx + 1;
                temp_idx = self.comment_idx;
                continue;
            } else if !comment.is_trailing && comment.span.start.offset > item_end {
                // Non-trailing comment after item - belongs to next item
                break;
            }

            temp_idx += 1;
        }
    }

    fn attach_comments_before(&mut self, pos: u32, annotations: &mut Vec<AnnotationAppl>) {
        while self.comment_idx < self.comments.len() {
            let comment = &self.comments[self.comment_idx];

            if comment.span.start.offset >= pos {
                break;
            }

            if comment.is_trailing {
                break;
            }
            annotations.push(comment_to_doc_annotation(comment));
            self.comment_idx += 1;
        }
    }

    fn attach_trailing_comments_impl(
        &mut self,
        max_offset: Option<u32>,
        annotations: &mut Vec<AnnotationAppl>,
    ) -> bool {
        let start_idx = self.comment_idx;

        while self.comment_idx < self.comments.len() {
            let comment = &self.comments[self.comment_idx];

            if let Some(max) = max_offset {
                if comment.span.start.offset >= max {
                    break;
                }
            }

            if comment.is_trailing {
                annotations.push(comment_to_doc_annotation(comment));
                self.comment_idx += 1;
            } else {
                break;
            }
        }

        self.comment_idx > start_idx
    }

    fn attach_trailing_comments_until(&mut self, pos: u32, annotations: &mut Vec<AnnotationAppl>) {
        self.attach_trailing_comments_impl(Some(pos), annotations);
    }

    fn attach_trailing_comments(&mut self, annotations: &mut Vec<AnnotationAppl>) {
        self.attach_trailing_comments_impl(None, annotations);
    }
}

/// Get mutable reference to item's annotations if it has any
fn item_annotations_mut(item: &mut Item) -> Option<&mut Vec<AnnotationAppl>> {
    use Item::{
        AliasValue, AnnotationValue, BitmaskValue, BitsetValue, ConstValue, DeclValue, EnumValue,
        ExceptionValue, InterfaceValue, ModuleValue, StructValue, UnionValue, ValuetypeValue,
    };
    match item {
        ModuleValue(x) => Some(&mut x.annotations),
        StructValue(x) => Some(&mut x.annotations),
        UnionValue(x) => Some(&mut x.annotations),
        EnumValue(x) => Some(&mut x.annotations),
        InterfaceValue(x) => Some(&mut x.annotations),
        ExceptionValue(x) => Some(&mut x.annotations),
        BitmaskValue(x) => Some(&mut x.annotations),
        BitsetValue(x) => Some(&mut x.annotations),
        AnnotationValue(x) => Some(&mut x.annotations),
        ValuetypeValue(x) => Some(&mut x.annotations),
        ConstValue(x) => Some(&mut x.annotations),
        AliasValue(x) => Some(&mut x.annotations),
        DeclValue(_) => None,
    }
}

fn first_interface_member_offset(members: &[InterfaceMember]) -> Option<u32> {
    members.first().map(|m| match m {
        InterfaceMember::Attr(a) => a.decl.first().map_or(0, |d| decl_span(d).start.offset),
        InterfaceMember::Proto(p) => p.ident.span.start.offset,
        InterfaceMember::Item(item) => item_span(item).start.offset,
    })
}

fn first_annotation_param_offset(params: &[AnnotationField]) -> Option<u32> {
    params.first().map(|p| match p {
        AnnotationField::Item(item) => item_span(item).start.offset,
        AnnotationField::Member(m) => m.span.start.offset,
    })
}

fn first_value_element_offset(elements: &[ValueElement]) -> Option<u32> {
    elements.first().map(|e| match e {
        ValueElement::State(m) => m.decl.first().map_or(0, |d| decl_span(d).start.offset),
        ValueElement::Attr(a) => a.decl.first().map_or(0, |d| decl_span(d).start.offset),
        ValueElement::Proto(p) => p.ident.span.start.offset,
        ValueElement::Item(item) => item_span(item).start.offset,
    })
}

/// Convert a comment to a @doc annotation
fn comment_to_doc_annotation(comment: &Comment) -> AnnotationAppl {
    let text = clean_comment_text(&comment.text);

    AnnotationAppl {
        ident: Path {
            leading_colons: None,
            segments: vec![Ident {
                name: "doc".to_string(),
                span: comment.span,
            }],
        },
        span: comment.span,
        args: vec![AnnotationArg {
            ident: None,
            span: comment.span,
            value: Expr::Literal(Literal {
                span: comment.span,
                value: LiteralValue::String(text),
            }),
        }],
    }
}

/// Clean comment text by removing comment markers
fn clean_comment_text(text: &str) -> String {
    let trimmed = if text.starts_with("///<") || text.starts_with("//!<") {
        &text[4..]
    } else if text.starts_with("///") || text.starts_with("//!") {
        &text[3..]
    } else if let Some(stripped) = text.strip_prefix("//") {
        stripped
    } else if text.starts_with("/**<") || text.starts_with("/*!<") {
        &text[4..text.len() - 2]
    } else if text.starts_with("/**") || text.starts_with("/*!") {
        &text[3..text.len() - 2]
    } else if text.starts_with("/*") {
        &text[2..text.len() - 2]
    } else {
        text
    };
    trimmed.trim().to_string()
}
