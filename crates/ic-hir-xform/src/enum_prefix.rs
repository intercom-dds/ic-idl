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

//! Enum prefix stripping transformation
//!
//! If all enumerators have a prefix that is also found in the name of the enum
//! itself, this transformation will strip that prefix from the names of the
//! enumerators.
//!
//! For example:
//! ```idl
//! enum Color { COLOR_RED, COLOR_GREEN };
//! ```
//! will be converted to:
//! ```idl
//! enum Color { RED, GREEN };
//! ```

use ic_emit::case;
use ic_hir::{Context, ResolvedGraph, hir};

/// Check if a string starts with an alphabetic character (allowing leading underscores)
fn starts_with_alpha(s: &str) -> bool {
    for c in s.chars() {
        if c.is_alphabetic() {
            return true;
        }
        if c != '_' {
            return false;
        }
    }
    false
}

/// Find the last delimiter (underscore or case change) in a string
/// Returns the position where the prefix ends (exclusive)
fn rfind_delimiter(s: &str) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    let mut was_upper = false;

    for i in (1..chars.len()).rev() {
        let c = chars[i];
        if i >= 1 {
            let peek = chars[i - 1];

            if peek == '_' {
                // For underscores, include the underscore in the prefix
                return Some(i);
            } else if (c.is_lowercase() && peek.is_uppercase())
                || (was_upper && c.is_uppercase() && peek.is_lowercase())
            {
                return Some(i - 1);
            }
        }
        was_upper = c.is_uppercase();
    }

    None
}

/// Strip common prefix from enum constants
fn strip_prefix_from_enum(enum_def: &hir::Def, context: &Context) -> Vec<(hir::DefId, String)> {
    let enum_name = &enum_def.ident.name;
    let mut renames = Vec::new();

    // Get all enum constants
    let constants: Vec<_> = match &enum_def.kind {
        hir::DefKind::Enum(e) => e
            .fields
            .iter()
            .filter_map(|&id| {
                let def = context.type_of(id);
                if matches!(def.kind, hir::DefKind::Const(_)) {
                    Some((id, def.ident.name.clone()))
                } else {
                    None
                }
            })
            .collect(),
        _ => return renames,
    };

    if constants.is_empty() {
        return renames;
    }

    // Get the first constant's name to find potential prefixes
    let first_name = &constants[0].1;
    let mut prefix = if let Some(pos) = rfind_delimiter(&first_name) {
        first_name[..pos].to_string()
    } else {
        // No delimiter found, use empty prefix
        return renames;
    };

    loop {
        // Check if all constants have this prefix
        let all_have_prefix = constants.iter().all(|(_, name)| {
            if name.len() > prefix.len() {
                let remainder = &name[prefix.len()..];
                let view = &name[..prefix.len()];
                starts_with_alpha(remainder) && view == prefix
            } else {
                false
            }
        });

        if all_have_prefix {
            // Convert both enum name and prefix to snake_case for comparison
            let found_prefix = case::convert(&prefix, case::Case::Snake);
            let type_name = case::convert(enum_name, case::Case::Snake);

            // Check if the type name contains the same prefix
            if type_name.len() >= found_prefix.len() && type_name.starts_with(&found_prefix) {
                // Strip the prefix from all constants
                for (id, name) in &constants {
                    let new_name = name[prefix.len()..].to_string();
                    renames.push((*id, new_name));
                }
                break;
            }
        }

        // Find the next delimiter and try again
        if let Some(pos) = rfind_delimiter(&prefix) {
            prefix = prefix[..pos].to_string();
        } else {
            break;
        }
    }

    renames
}

/// Strip common prefix from bitmask flags  
fn strip_prefix_from_bitmask(
    context: &Context,
    bitmask_def: &hir::Def,
) -> Vec<(hir::DefId, String)> {
    let bitmask_name = &bitmask_def.ident.name;
    let mut renames = Vec::new();

    // Get all flag names and IDs
    let flags: Vec<(hir::DefId, String)> = match &bitmask_def.kind {
        hir::DefKind::Bitmask(b) => b
            .flags
            .iter()
            .filter_map(|&flag_id| {
                let flag_def = context.type_of(flag_id);
                if matches!(flag_def.kind, hir::DefKind::Const(_)) {
                    Some((flag_id, flag_def.ident.name.clone()))
                } else {
                    None
                }
            })
            .collect(),
        _ => return renames,
    };

    if flags.is_empty() {
        return renames;
    }

    // Similar logic as enums
    let first_name = &flags[0].1;
    let mut prefix = if let Some(pos) = rfind_delimiter(&first_name) {
        first_name[..pos].to_string()
    } else {
        return renames;
    };

    loop {
        let all_have_prefix = flags.iter().all(|(_, name)| {
            if name.len() > prefix.len() {
                let remainder = &name[prefix.len()..];
                let view = &name[..prefix.len()];
                starts_with_alpha(remainder) && view == prefix
            } else {
                false
            }
        });

        if all_have_prefix {
            let found_prefix = case::convert(&prefix, case::Case::Snake);
            let type_name = case::convert(bitmask_name, case::Case::Snake);

            if type_name.len() >= found_prefix.len() && type_name.starts_with(&found_prefix) {
                for (id, name) in &flags {
                    let new_name = name[prefix.len()..].to_string();
                    renames.push((*id, new_name));
                }
                break;
            }
        }

        if let Some(pos) = rfind_delimiter(&prefix) {
            prefix = prefix[..pos].to_string();
        } else {
            break;
        }
    }

    renames
}

/// Transform HIR to strip common prefixes from enum and bitmask members
#[must_use]
pub fn transform(mut hir: ResolvedGraph) -> ResolvedGraph {
    // Process all enums
    let enum_renames: Vec<_> = hir
        .context
        .definitions
        .iter()
        .filter(|(_, def)| matches!(def.kind, hir::DefKind::Enum(_)))
        .flat_map(|(_, def)| strip_prefix_from_enum(def, &hir.context))
        .collect();

    // Apply enum constant renames
    for (const_id, new_name) in enum_renames {
        hir.context.definitions.get_mut(const_id).ident.name = new_name;
    }

    // Process all bitmasks
    let bitmask_renames: Vec<_> = hir
        .context
        .definitions
        .iter()
        .filter(|(_, def)| matches!(def.kind, hir::DefKind::Bitmask(_)))
        .flat_map(|(_, def)| strip_prefix_from_bitmask(&hir.context, def))
        .collect();

    // Apply bitmask flag renames
    for (flag_id, new_name) in bitmask_renames {
        hir.context.definitions.get_mut(flag_id).ident.name = new_name;
    }

    hir
}
