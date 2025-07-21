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

use ic_cli::color::{ColorMode, Colorize, ColorizeExt};

#[test]
fn test_colorize_with_mode() {
    // Test colorizing with explicit mode
    let colored = "test".colorize(ColorMode::Always);
    let _ = colored.to_string(); // ColorMode controls whether colors are applied
}

#[test]
fn test_direct_colorize() {
    // The simple Colorize trait is applied directly to strings
    let red_text = "test".red();
    // Whether it contains escape codes depends on terminal detection
    assert!(red_text.contains("test"));
}

#[test]
fn test_all_foreground_colors() {
    // Direct colorize methods depend on terminal detection
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
fn test_bold_and_clear() {
    let _ = "test".bold();
    let _ = "test".clear();
}

#[test]
fn test_string_extension() {
    // Test that the extension trait works on String
    let text = String::from("test");
    let colored = text.red();
    assert!(colored.contains("test"));
}

#[test]
fn test_str_extension() {
    // Test that the extension trait works on &str
    let colored = "test".blue();
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
    // Test that Auto is the default
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
    // Test that Colored implements Display
    let colored = "test".colorize(ColorMode::Never);
    let _output = format!("{colored}");
}

#[test]
fn test_colorize_mode_behavior() {
    // With Never mode, output should be plain text
    let never = "test".colorize(ColorMode::Never).red();
    let output = format!("{never}");
    assert_eq!(output, "test");

    // With Always mode, should add color codes
    let always = "test".colorize(ColorMode::Always).red();
    let output = format!("{always}");
    assert!(output.len() > 4); // More than just "test" due to ANSI codes
}
