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

use crate::exception_types;

fn is_error(_err: &dyn std::error::Error) -> bool {
    true
}

#[test]
fn exception_inherits_from_exception() {
    let err = exception_types::SimpleError {
        error_code: 42,
        message: "test error".into(),
    };
    assert!(is_error(&err));
    assert_eq!(err.to_string(), "SimpleError".to_string()); // std::error::Error uses the Display trait as reason of error
}

#[test]
fn exception_instantiation() {
    let err = exception_types::SimpleError {
        error_code: 404,
        message: "Not found".into(),
    };
    assert_eq!(err.error_code, 404);
    assert_eq!(err.message, "Not found");
}

#[test]
fn exception_raise_and_catch() {
    let r: exception_types::SimpleErrorResult<()> = Err(exception_types::SimpleError {
        error_code: 500,
        message: "Internal error".into(),
    });
    match r {
        Ok(_) => unreachable!(),
        Err(e) => {
            assert_eq!(e.error_code, 500);
            assert_eq!(e.message, "Internal error");
        }
    }
}

#[test]
fn exception_catch_as_base() {
    let r: exception_types::SimpleErrorResult<()> = Err(exception_types::SimpleError {
        error_code: 403,
        message: "Forbidden".into(),
    });

    match r.map_err(|e| Box::new(e) as Box<dyn std::error::Error>) {
        Ok(_) => unreachable!(),
        Err(err) => {
            let e = err
                .downcast::<exception_types::SimpleError>()
                .expect("SimpleError downcast");
            assert_eq!(e.error_code, 403);
            assert_eq!(e.message, "Forbidden");
        }
    }
}

#[test]
fn empty_exception() {
    let empty = exception_types::EmptyError::new();
    assert!(is_error(&empty));

    let r: exception_types::EmptyErrorResult<()> = Err(empty);
    match r {
        Ok(_) => unreachable!(),
        Err(_) => assert!(true),
    };
}

#[test]
fn detailed_exception_fields() {
    let err = exception_types::DetailedError {
        code: 1001,
        message: "Database error".into(),
        details: "Connection timeout".into(),
        recoverable: true,
    };
    assert_eq!(err.code, 1001);
    assert_eq!(err.message, "Database error");
    assert_eq!(err.details, "Connection timeout");
    assert!(err.recoverable);

    let err2 = exception_types::DetailedError {
        code: 2002,
        message: "Fatal error".into(),
        details: "Out of memory".into(),
        recoverable: false,
    };
    assert_eq!(err2.code, 2002);
    assert!(!err2.recoverable);
}

#[test]
fn validation_error() {
    let verr = exception_types::ValidationError {
        field_name: "email".into(),
        error_message: "Invalid format".into(),
        position: 15,
    };
    assert_eq!(verr.field_name, "email");
    assert_eq!(verr.error_message, "Invalid format");
    assert_eq!(verr.position, 15);

    match Err(verr) as exception_types::ValidationErrorResult<()> {
        Ok(_) => unreachable!(),
        Err(e) => {
            assert_eq!(e.field_name, "email");
            assert_eq!(e.error_message, "Invalid format");
            assert_eq!(e.position, 15);
        }
    }
}

#[test]
fn exception_swap() {
    let mut e1 = exception_types::SimpleError {
        error_code: 404,
        message: "not found".into(),
    };
    let mut e2 = exception_types::SimpleError {
        error_code: 500,
        message: "server error".into(),
    };

    std::mem::swap(&mut e1, &mut e2);

    assert_eq!(e1.error_code, 500);
    assert_eq!(e1.message, "server error");
    assert_eq!(e2.error_code, 404);
    assert_eq!(e2.message, "not found");
}

#[test]
fn exception_swap_detailed() {
    let mut e1 = exception_types::DetailedError {
        code: 1001,
        message: "Error A".into(),
        details: "Details A".into(),
        recoverable: true,
    };
    let mut e2 = exception_types::DetailedError {
        code: 2002,
        message: "Error B".into(),
        details: "Details B".into(),
        recoverable: false,
    };

    std::mem::swap(&mut e1, &mut e2);

    assert_eq!(e1.code, 2002);
    assert_eq!(e1.message, "Error B");
    assert_eq!(e1.details, "Details B");
    assert!(!e1.recoverable);

    assert_eq!(e2.code, 1001);
    assert_eq!(e2.message, "Error A");
    assert_eq!(e2.details, "Details A");
    assert!(e2.recoverable);
}

#[test]
fn exception_result_error_type() {
    let r: exception_types::TResult<()> = Err(exception_types::T { code: 7 });
    std::assert_matches!(r, Err(exception_types::T { code: 7 }));
}
