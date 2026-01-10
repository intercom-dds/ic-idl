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

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::{env, error, fmt, io};

mod path;

/// Configure and generate Rust types from IDL.
///
/// ```ignore
/// use intercom_build::Codegen;
///
/// // Configure and generate types
/// let files = Codegen::new("my_idl")
///     .set_header_follow(false)
///     .include("my/include/directory")
///     .input([
///         "my_file.idl",
///         "my_other_file.idl",
///     ])
///     .generate()?;
/// ```
///
/// Then, in your crate root, you can use the [`include_idl!`] macro to include
/// the generated code:
///
/// ```ignore
/// intercom::include_idl!("my_idl");
/// ````
///
/// [`include_idl!`]: crate::include_idl
#[must_use]
#[derive(Default)]
pub struct Codegen {
    subdir: String,
    input: Vec<PathBuf>,
    includes: Vec<PathBuf>,
    flags: Vec<String>,
    env: HashMap<String, String>,
    exe: Option<PathBuf>,
}

impl Codegen {
    /// The name specified here will be the same name you must specify when
    /// using the [`include_idl!`] macro. The name has no other implications,
    /// and only affects the output directory in which the generated files are
    /// placed.
    ///
    /// [`include_idl!`]: intercom::include_idl
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            subdir: name.into(),
            input: vec![],
            includes: vec![],
            flags: vec![],
            env: HashMap::new(),
            exe: None,
        }
    }

    /// List of input files that will be compiled by ic-idl.
    pub fn input<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<PathBuf>,
    {
        self.input.extend(iter.into_iter().map(Into::into));
        self
    }

    /// Sets the include directories that ic-idl will search for included IDL files.
    pub fn include<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.includes.push(path.into());
        self
    }

    /// Sets the include directories that ic-idl will search for included IDL files.
    pub fn includes<I>(mut self, dirs: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<PathBuf>,
    {
        self.includes.extend(dirs.into_iter().map(Into::into));
        self
    }

    /// Refer to the core InterCOM documentation for more information about the
    /// available flags.
    pub fn flag<S: Into<String>>(mut self, flag: S) -> Self {
        self.flags.push(flag.into());
        self
    }

    /// Refer to the core InterCOM documentation for more information about the
    /// available flags.
    pub fn flags<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.flags.extend(iter.into_iter().map(Into::into));
        self
    }

    pub fn env<K, V>(mut self, key: K, val: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.env.insert(key.into(), val.into());
        self
    }

    /// Do not rename types to follow Rust's naming convention.
    pub fn no_rename(mut self, no_rename: bool) -> Self {
        self.flags.push(format!("--no-rename={no_rename}"));
        self
    }

    /// Manually specify the path of ic-idl.
    /// In most cases, setting this should not be necessary as the library will
    /// attempt to locate ic-idl by searching the user's path.
    pub fn executable<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.exe = Some(path.into());
        self
    }

    /// Invokes the compiler, outputting generated Rust code.
    pub fn generate(mut self) -> Result<(), CodegenError> {
        let exe = self.find_compiler()?;
        let dir = env::var("OUT_DIR").expect("ic-idl can only be invoked from build scripts");

        for f in &self.input {
            println!("cargo:rerun-if-changed={}", f.to_string_lossy());
        }
        self.flags.push(format!("--rust-out={dir}/{}", self.subdir));
        println!("cargo:rerun-if-changed={}", exe.to_string_lossy());

        for inc in self.includes {
            self.flags
                .push(format!("--include={}", inc.to_string_lossy()));
        }

        let res = Command::new(exe)
            .envs(&self.env)
            .args(&self.flags)
            .args(&self.input)
            .output()
            .map_err(CodegenError::IoError)?;

        if !res.stderr.is_empty() {
            Self::emit_diagnostics(&res);
        }
        Ok(())
    }

    fn find_compiler(&self) -> Result<PathBuf, CodegenError> {
        if let Some(exe) = self.exe.clone().or_else(|| path::find_exe("ic-idl")) {
            Ok(exe)
        } else {
            Err(CodegenError::NotFound(format!(
                "Failed to locate executable in search path {:?}",
                path::search_dirs()
            )))
        }
    }

    fn emit_diagnostics(output: &Output) {
        let msg = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            let mut formatted = String::with_capacity(msg.len());
            for line in msg.lines() {
                if !line.is_empty() {
                    formatted += &format!("cargo:warning=ic-idl: {line}\n");
                }
            }
            println!("{formatted}");
        } else {
            panic!("ic-idl: {msg}");
        }
    }
}

#[derive(Debug)]
pub enum CodegenError {
    /// A suitable ic-idl executable was not found.
    NotFound(String),

    /// Diagnostic produced by ic-idl.
    Diagnostic(String),

    /// Error produced by the underlying filesystem.
    IoError(io::Error),
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(s) => write!(f, "A suitable ic-idl executable was not found: {s}"),
            Self::Diagnostic(s) => write!(f, "ic-idl diagnostic: {s}"),
            Self::IoError(e) => write!(f, "{e}"),
        }
    }
}

impl error::Error for CodegenError {}
