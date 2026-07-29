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

use std::env;
use std::path::{Path, PathBuf};

const BINARY_NAME: &str = "ic-idl";

/// Returns a list of all directories found in `$PATH`.
pub fn user_path() -> Vec<PathBuf> {
    env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .map(Into::into)
        .collect()
}

/// Returns a list of all paths that should be searched for executables.
///
/// Note: The `IC_IDL_EXE` environment variable is checked separately in [`find_exe`]
/// before searching these directories. Set `IC_IDL_EXE` to the full path of the
/// `ic-idl` binary to bypass directory searching.
pub fn search_dirs() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Check the DEP_INTERCOM_DIR env variable set by intercom crate
    if let Ok(ref p) = env::var("DEP_INTERCOM_DIR") {
        let p = Path::new(&p).join("bin");
        paths.extend(configs(&p));
    }

    // Check the INTERCOM_DIR env variable
    if let Ok(ref p) = env::var("INTERCOM_DIR") {
        paths.push(p.into());
        let binary = Path::new(&p).join(BINARY_NAME);
        paths.extend(configs(&binary));

        if let Some(parent) = Path::new(&p).parent() {
            let binary = parent.join(BINARY_NAME);
            paths.extend(configs(&binary));
        }

        // Install archive
        let p = Path::new(&p).join("bin");
        paths.extend(configs(&p));
    }

    // Search PATH variable after special locations above have been failed
    paths.extend(user_path());

    // Search the current working directory
    if let Ok(p) = env::current_dir() {
        paths.push(p);
    }

    paths
}

/// Attempts to locate an executable with the specified name.
pub fn find_exe(name: &str) -> Option<PathBuf> {
    // Check IC_IDL_EXE environment variable first (direct path to binary)
    if name == "ic-idl" {
        if let Ok(path) = env::var("IC_IDL_EXE") {
            let path = PathBuf::from(path);
            if path.is_file() {
                return Some(path);
            }
        }
    }

    let dirs = search_dirs();
    let name_exe = format!("{name}{}", env::consts::EXE_SUFFIX);

    for dir in dirs {
        if let Ok(files) = dir.read_dir() {
            let file = files
                .filter_map(Result::ok)
                .find(|f| f.path().is_file() && f.file_name() == name_exe.as_str())
                .map(|f| f.path());

            if file.is_some() {
                return file;
            }
        }
    }
    None
}

// Returns a list of directories to search, adapted for single-config and
// multi-config generators.
fn configs(path: &Path) -> Vec<PathBuf> {
    vec![
        path.join("Release"),
        path.join("RelWithDebInfo"),
        path.join("Debug"),
        path.to_owned(),
    ]
}
