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

use std::collections::BTreeMap;
use std::fmt;

use ic_cli::color::Colorize as _;
use ic_vfs::SourceMap;

use crate::engine::{LineIndex, compute_frame_layout};
use crate::render::{Renderer, render_footer, render_header};
use crate::{Color, Diag};

#[derive(Clone, Debug)]
pub struct Line {
    pub text: &'static str,
    pub color: Color,
    pub symbol: &'static str,
}

pub fn compact(
    f: &mut dyn fmt::Write,
    filename: &str,
    diag: &Diag,
    index: &LineIndex,
) -> fmt::Result {
    let title = if let Some(code) = &diag.code {
        format!("{}[{}]", diag.title.text, code)
    } else {
        diag.title.text.to_string()
    };

    if let Some(label) = diag.labels.first() {
        let (line, col) = index.line_col(label.span.start.offset);
        writeln!(
            f,
            "{}:{}:{}: {}: {}",
            filename,
            line,
            col,
            title.fg(diag.title.color).bold(),
            diag.msg,
        )
    } else {
        writeln!(
            f,
            "{}: {}: {}",
            filename,
            title.fg(diag.title.color).bold(),
            diag.msg,
        )
    }
}

pub fn with_file_cached(
    f: &mut dyn fmt::Write,
    vfs: &SourceMap,
    diag: &Diag,
    cache: &mut std::collections::HashMap<ic_vfs::FileId, LineIndex>,
    max_width: Option<usize>,
) -> fmt::Result {
    render_header(f, diag)?;

    if diag.labels.is_empty() {
        render_footer(f, diag)?;
        return Ok(());
    }

    let mut labels_by_file: BTreeMap<ic_vfs::FileId, Vec<usize>> = BTreeMap::new();
    for (idx, label) in diag.labels.iter().enumerate() {
        labels_by_file
            .entry(label.span.start.file_id)
            .or_default()
            .push(idx);
    }

    for (file_id, label_indices) in labels_by_file {
        let info = vfs.file_info(file_id);
        let name = info.path.to_string_lossy();
        let source = &info.source;

        let index = cache
            .entry(file_id)
            .or_insert_with(|| LineIndex::new(source));

        let file_labels: Vec<crate::Label> = label_indices
            .iter()
            .map(|&idx| diag.labels[idx].clone())
            .collect();

        let layout = compute_frame_layout(
            index,
            source,
            &file_labels,
            max_width,
            diag.get_context_lines(),
        );

        let mut file_renderer = Renderer::new(
            f,
            index,
            source,
            layout.gutter_width,
            Some(&name),
            diag.title.color,
        );
        file_renderer.render_frame(&layout, &file_labels)?;
    }

    render_footer(f, diag)
}

pub fn with_source_indexed(
    f: &mut dyn fmt::Write,
    name: &str,
    source: &str,
    diag: &Diag,
    index: &LineIndex,
    max_width: Option<usize>,
) -> fmt::Result {
    crate::render::render_diagnostic(f, index, source, diag, Some(name), max_width)
}
