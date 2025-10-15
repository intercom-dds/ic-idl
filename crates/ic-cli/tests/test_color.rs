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

use ic_cli::color::{ColorMode, Colorize as _};

#[test]
fn test_colorize_with_mode() {
    let colored = "test".red().mode(ColorMode::Always);
    let _ = colored.to_string();
}

#[test]
fn test_direct_colorize() {
    let red_text = "test".red().to_string();
    assert!(red_text.contains("test"));
}

#[test]
fn test_all_foreground_colors() {
    let _ = "test".red();
    let _ = "test".green();
    let _ = "test".yellow();
    let _ = "test".blue();
    let _ = "test".purple();
    let _ = "test".cyan();
    let _ = "test".white();
    let _ = "test".gray();
    let _ = "test".black();
}

#[test]
fn test_bold() {
    let _ = "test".bold();
}

#[test]
fn test_string_extension() {
    let text = String::from("test");
    let colored = text.red().to_string();
    assert!(colored.contains("test"));
}

#[test]
fn test_str_extension() {
    let colored = "test".blue().to_string();
    assert!(colored.contains("test"));
}

#[test]
fn test_nested_formatting() {
    let inner = "inner".red();
    let outer = format!("outer [{inner}] text");
    assert!(outer.contains("inner"));
}

#[test]
fn test_default_color_mode() {
    assert_eq!(ColorMode::default(), ColorMode::Auto);
}

#[test]
fn test_color_mode_equality() {
    assert_eq!(ColorMode::Always, ColorMode::Always);
    assert_eq!(ColorMode::Never, ColorMode::Never);
    assert_eq!(ColorMode::Auto, ColorMode::Auto);
    assert_ne!(ColorMode::Always, ColorMode::Never);
}

#[test]
fn test_colored_display() {
    let colored = "test".red().mode(ColorMode::Never);
    let _output = format!("{colored}");
}

#[test]
fn test_colorize_mode_behavior() {
    ic_cli::color::set_color_override(ColorMode::Auto);
    let never = "test".red().mode(ColorMode::Never);
    let output = format!("{never}");
    assert_eq!(output, "test");

    let always = "test".red().mode(ColorMode::Always);
    let output = format!("{always}");
    assert!(output.len() > 4);
}
