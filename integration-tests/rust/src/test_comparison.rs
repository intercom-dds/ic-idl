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

use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{circular_types, exception_types, struct_types, union_types};

fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[test]
fn struct_equality() {
    let p1 = struct_types::Point { x: 10, y: 20 };
    let p2 = struct_types::Point { x: 10, y: 20 };
    let p3 = struct_types::Point { x: 5, y: 10 };

    assert_eq!(p1, p2);
    assert_ne!(p1, p3);
    assert!(!(p1 == p3));
}

#[test]
fn struct_ordering() {
    let p1 = struct_types::Point { x: 1, y: 2 };
    let p2 = struct_types::Point { x: 2, y: 1 };
    let p3 = struct_types::Point { x: 1, y: 3 };

    assert!(p1 < p2);
    assert!(p1 < p3);
    assert!(!(p2 < p1));
    assert!(p2 > p1);
    assert!(p3 > p1);
    assert!(p1 <= p2);
    assert!(p1 >= p1);
}

#[test]
fn struct_sorting() {
    let mut points = Vec::new();

    points.push(struct_types::Point { x: 2, y: 1 });
    points.push(struct_types::Point { x: 1, y: 3 });
    points.push(struct_types::Point { x: 1, y: 2 });

    points.sort();

    assert_eq!(points[0], struct_types::Point { x: 1, y: 2 });
    assert_eq!(points[1], struct_types::Point { x: 1, y: 3 });
    assert_eq!(points[2], struct_types::Point { x: 2, y: 1 });
}

#[test]
fn struct_hashable() {
    let p1 = struct_types::Point { x: 10, y: 20 };
    let p2 = struct_types::Point { x: 10, y: 20 };
    assert_eq!(hash(&p1), hash(&p2));

    let mut set = std::collections::HashSet::new();
    set.insert(p1);
    assert!(set.contains(&p2));
}

#[test]
fn union_equality() {
    let u1 = union_types::IntOrString::IntVal(42);
    let u2 = union_types::IntOrString::IntVal(42);
    let u3 = union_types::IntOrString::IntVal(99);

    assert_eq!(u1, u2);
    assert_ne!(u1, u3);

    let u4 = union_types::IntOrString::StrVal("hello".into());
    let u5 = union_types::IntOrString::StrVal("hello".into());

    assert_eq!(u4, u5);
    assert_ne!(u1, u4);
}

#[test]
fn union_sorting() {
    let mut unions = Vec::new();

    unions.push(union_types::IntOrString::IntVal(50));
    unions.push(union_types::IntOrString::IntVal(10));
    unions.push(union_types::IntOrString::IntVal(30));

    unions.sort();

    assert!(matches!(unions[0], union_types::IntOrString::IntVal(x) if x == 10));
    assert!(matches!(unions[1], union_types::IntOrString::IntVal(x) if x == 30));
    assert!(matches!(unions[2], union_types::IntOrString::IntVal(x) if x == 50));
}

#[test]
fn union_hashable() {
    let u1 = union_types::IntOrString::IntVal(42);
    let u2 = union_types::IntOrString::IntVal(42);

    assert_eq!(hash(&u1), hash(&u2));

    let mut set = std::collections::HashSet::new();
    set.insert(u1);
    assert!(set.contains(&u2));

    let u3 = union_types::IntOrString::StrVal("test".into());
    set.insert(u3);
    assert_eq!(set.len(), 2);
}

#[test]
fn exception_equality() {
    let e1 = exception_types::SimpleError {
        error_code: 100,
        message: "error".into(),
    };
    let e2 = exception_types::SimpleError {
        error_code: 100,
        message: "error".into(),
    };
    let e3 = exception_types::SimpleError {
        error_code: 300,
        message: "different".into(),
    };

    assert_eq!(e1, e2);
    assert_ne!(e1, e3);

    assert_eq!(e1.error_code, e2.error_code);
    assert_eq!(e1.message, e2.message);
    assert_ne!(e1.error_code, e3.error_code);
    assert_ne!(e1.message, e3.message);
}

#[test]
fn exception_sorting() {
    let mut errors = Vec::new();
    errors.push(exception_types::SimpleError {
        error_code: 500,
        message: "server error".into(),
    });
    errors.push(exception_types::SimpleError {
        error_code: 100,
        message: "continue".into(),
    });
    errors.push(exception_types::SimpleError {
        error_code: 404,
        message: "not found".into(),
    });

    errors.sort();

    assert_eq!(errors[0].error_code, 100);
    assert_eq!(errors[1].error_code, 404);
    assert_eq!(errors[2].error_code, 500);
}

#[test]
fn exception_hashable() {
    let e1 = exception_types::SimpleError {
        error_code: 404,
        message: "not found".into(),
    };
    let e2 = exception_types::SimpleError {
        error_code: 404,
        message: "not found".into(),
    };

    assert_eq!(hash(&e1), hash(&e2));

    let mut set = std::collections::HashSet::new();
    set.insert(e1);
    assert!(set.contains(&e2));

    let e3 = exception_types::SimpleError {
        error_code: 500,
        message: "server error".into(),
    };
    set.insert(e3);
    assert_eq!(set.len(), 2);
}

#[test]
fn circular_type_hash() {
    let node = circular_types::TreeNode {
        value: 42,
        ..Default::default()
    };

    assert_ne!(hash(&node), 0);

    let node2 = circular_types::TreeNode {
        value: 42,
        ..Default::default()
    };
    assert_eq!(hash(&node), hash(&node2));
}
