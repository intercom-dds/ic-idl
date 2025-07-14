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

use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn test_parameters_without_direction() {
    let source = r"
        interface Service {
            void process(string data);
            long calculate(double x, double y);
            string format(long value, boolean uppercase);
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_mixed_direction_specifications() {
    let source = r"
        interface Buffer {
            void write(in octet data);
            void read(out octet data);
            void modify(inout string text);
            void broken(long size);  // Missing direction
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_all_parameters_with_direction() {
    let source = r"
        interface Correct {
            void send(in string message);
            void receive(out string message);
            void transform(inout sequence<long> data);
            boolean compare(in string a, in string b);
        };
    ";

    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Should not warn when all parameters have directions"
    );
}

#[test]
fn test_oneway_operations() {
    let source = r"
        interface Async {
            oneway void notify(string event);
            oneway void log(long level, string message);
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_complex_parameter_types() {
    let source = r"
        struct Data {
            long id;
            string name;
        };
        
        interface DataService {
            void store(Data item);
            Data retrieve(long id);
            void update(long id, Data item);
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_readonly_attribute() {
    let source = r"
        interface Config {
            readonly attribute string version;
            attribute long timeout;
        };
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for attributes");
}
