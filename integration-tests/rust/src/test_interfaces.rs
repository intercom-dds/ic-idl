// Copyright 2026 KONGSBERG
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

use crate::interface_types;

#[test]
fn interface_is_abc() {
    fn _trait_exists(_t: &dyn interface_types::Reader) -> bool {
        true
    }
    assert!(true);
}

#[test]
fn interface_has_abstract_methods() {
    fn _trait_exists(r: &mut dyn interface_types::Reader) -> bool {
        r.read();
        r.has_more()
    }
    assert!(true);
}

#[test]
fn interface_method_signature_no_params() {
    struct R;
    impl interface_types::Reader for R {
        fn read(&mut self) -> String {
            unreachable!();
        }

        fn has_more(&mut self) -> bool {
            unreachable!();
        }
    }

    let _ = R {};
    assert!(true);
}

#[test]
fn interface_method_signature_with_params() {
    struct C;
    impl interface_types::Calculator for C {
        fn add(&mut self, _a: i32, _b: i32) -> i32 {
            unreachable!();
        }

        fn subtract(&mut self, _a: i32, _b: i32) -> i32 {
            unreachable!();
        }

        fn divide(&mut self, _a: f64, _b: f64) -> f64 {
            unreachable!();
        }
    }

    let _ = C {};
    assert!(true);
}

#[test]
fn interface_void_return() {
    struct W;
    impl interface_types::Writer for W {
        fn write(&mut self, _data: &str) {
            unreachable!();
        }

        fn flush(&mut self) {
            unreachable!();
        }
    }

    let _ = W {};
    assert!(true);
}

#[test]
fn empty_interface() {
    struct E;
    impl interface_types::Empty for E {}

    let _ = E {};
    assert!(true);
}

#[test]
fn operation_failed_exception() {
    let ex = interface_types::OperationFailed {
        error_code: 42,
        reason: "Test error".into(),
    };
    assert_eq!(ex.error_code, 42);
    assert_eq!(ex.reason, "Test error");
}

#[test]
fn invalid_input_exception() {
    let ex = interface_types::InvalidInput {
        parameter_name: "param_name".into(),
    };

    assert_eq!(ex.parameter_name, "param_name");
}

#[test]
fn exception_can_be_raised() {
    let r: interface_types::OperationFailedResult<()> = Err(interface_types::OperationFailed {
        error_code: 500,
        reason: "Server error".into(),
    });
    match r {
        Ok(_) => unreachable!(),
        Err(e) => {
            assert_eq!(e.error_code, 500);
            assert_eq!(e.reason, "Server error");
        }
    }
}

#[test]
fn interface_with_out_params_exists() {
    fn _trait_exists(_t: &dyn interface_types::WithOutParams) -> bool {
        true
    }
    assert!(true);
}

#[test]
fn interface_with_raises_exists() {
    fn _trait_exists(_t: &dyn interface_types::WithRaises) -> bool {
        true
    }
    assert!(true);
}

#[test]
fn combined_features_interface() {
    fn _trait_exists(_t: &dyn interface_types::CombinedFeatures) -> bool {
        true
    }
    assert!(true);
}

#[test]
fn interface_calculator_all_signatures() {
    struct C;
    impl interface_types::Calculator for C {
        fn add(&mut self, _a: i32, _b: i32) -> i32 {
            unreachable!();
        }

        fn subtract(&mut self, _a: i32, _b: i32) -> i32 {
            unreachable!();
        }

        fn divide(&mut self, _a: f64, _b: f64) -> f64 {
            unreachable!();
        }
    }

    let _ = C {};
    assert!(true);
}

#[test]
fn interface_writer_parameter_types() {
    fn _trait_typecheck(w: &mut dyn interface_types::Writer) {
        w.write("test")
    }
    assert!(true);
}

#[test]
fn interface_can_be_implemented() {
    use interface_types::Reader;

    struct R;
    impl Reader for R {
        fn read(&mut self) -> String {
            "test data".into()
        }

        fn has_more(&mut self) -> bool {
            false
        }
    }

    let mut reader = R {};

    assert_eq!(reader.read(), "test data");
    assert!(!reader.has_more());
}
