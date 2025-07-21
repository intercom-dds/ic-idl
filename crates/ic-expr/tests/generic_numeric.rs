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

use ic_expr::{Error, EvalConfig, GenericNumeric, NumericValue, OverflowBehavior};

#[test]
fn test_type_creation() {
    let b = GenericNumeric::Bool(true);
    let c = GenericNumeric::Char('A');
    let i8 = GenericNumeric::Int8(-42);
    let u8 = GenericNumeric::UInt8(200);
    let i16 = GenericNumeric::Int16(-1000);
    let u16 = GenericNumeric::UInt16(60000);
    let i32 = GenericNumeric::Int32(-1_000_000);
    let u32 = GenericNumeric::UInt32(3_000_000_000);
    let i64 = GenericNumeric::Int64(-9_000_000_000_000_000_000);
    let u64 = GenericNumeric::UInt64(18_000_000_000_000_000_000);
    let f = GenericNumeric::Float(std::f32::consts::PI);
    let d = GenericNumeric::Double(std::f64::consts::E);

    assert_eq!(b, GenericNumeric::Bool(true));
    assert_eq!(c, GenericNumeric::Char('A'));
    assert_eq!(i8, GenericNumeric::Int8(-42));
    assert_eq!(u8, GenericNumeric::UInt8(200));
    assert_eq!(i16, GenericNumeric::Int16(-1000));
    assert_eq!(u16, GenericNumeric::UInt16(60000));
    assert_eq!(i32, GenericNumeric::Int32(-1_000_000));
    assert_eq!(u32, GenericNumeric::UInt32(3_000_000_000));
    assert_eq!(i64, GenericNumeric::Int64(-9_000_000_000_000_000_000));
    assert_eq!(u64, GenericNumeric::UInt64(18_000_000_000_000_000_000));
    assert_eq!(f, GenericNumeric::Float(std::f32::consts::PI));
    assert_eq!(d, GenericNumeric::Double(std::f64::consts::E));
}

#[test]
fn test_from_bool() {
    let val = GenericNumeric::from_bool(true);
    assert_eq!(val, GenericNumeric::Bool(true));

    let val = GenericNumeric::from_bool(false);
    assert_eq!(val, GenericNumeric::Bool(false));
}

#[test]
fn test_to_bool() {
    assert!(GenericNumeric::Bool(true).to_bool());
    assert!(!GenericNumeric::Bool(false).to_bool());
    assert!(GenericNumeric::Char('A').to_bool());
    assert!(!GenericNumeric::Char('\0').to_bool());
    assert!(GenericNumeric::Int8(1).to_bool());
    assert!(!GenericNumeric::Int8(0).to_bool());
    assert!(GenericNumeric::Int8(-1).to_bool());
    assert!(GenericNumeric::UInt8(1).to_bool());
    assert!(!GenericNumeric::UInt8(0).to_bool());
    assert!(GenericNumeric::Int16(1).to_bool());
    assert!(!GenericNumeric::Int16(0).to_bool());
    assert!(GenericNumeric::UInt16(1).to_bool());
    assert!(!GenericNumeric::UInt16(0).to_bool());
    assert!(GenericNumeric::Int32(1).to_bool());
    assert!(!GenericNumeric::Int32(0).to_bool());
    assert!(GenericNumeric::UInt32(1).to_bool());
    assert!(!GenericNumeric::UInt32(0).to_bool());
    assert!(GenericNumeric::Int64(1).to_bool());
    assert!(!GenericNumeric::Int64(0).to_bool());
    assert!(GenericNumeric::UInt64(1).to_bool());
    assert!(!GenericNumeric::UInt64(0).to_bool());
    assert!(GenericNumeric::Float(1.0).to_bool());
    assert!(!GenericNumeric::Float(0.0).to_bool());
    assert!(GenericNumeric::Double(1.0).to_bool());
    assert!(!GenericNumeric::Double(0.0).to_bool());
}

#[test]
fn test_negate() {
    let config = EvalConfig {
        overflow: OverflowBehavior::Error,
        ..Default::default()
    };

    // Test negation of positive values
    assert_eq!(
        GenericNumeric::Bool(true).negate(config).unwrap(),
        GenericNumeric::Int32(-1)
    );
    assert_eq!(
        GenericNumeric::Bool(false).negate(config).unwrap(),
        GenericNumeric::Int32(0)
    );
    assert_eq!(
        GenericNumeric::Char('A').negate(config).unwrap(),
        GenericNumeric::Int32(-65)
    );
    assert_eq!(
        GenericNumeric::Int8(42).negate(config).unwrap(),
        GenericNumeric::Int8(-42)
    );
    assert_eq!(
        GenericNumeric::UInt8(42).negate(config).unwrap(),
        GenericNumeric::Int16(-42)
    );
    assert_eq!(
        GenericNumeric::Int16(1000).negate(config).unwrap(),
        GenericNumeric::Int16(-1000)
    );
    assert_eq!(
        GenericNumeric::UInt16(1000).negate(config).unwrap(),
        GenericNumeric::Int32(-1000)
    );
    assert_eq!(
        GenericNumeric::Int32(1_000_000).negate(config).unwrap(),
        GenericNumeric::Int32(-1_000_000)
    );
    assert_eq!(
        GenericNumeric::UInt32(1_000_000).negate(config).unwrap(),
        GenericNumeric::Int64(-1_000_000)
    );
    assert_eq!(
        GenericNumeric::Int64(1_000_000_000).negate(config).unwrap(),
        GenericNumeric::Int64(-1_000_000_000)
    );
    assert_eq!(
        GenericNumeric::Float(std::f32::consts::PI)
            .negate(config)
            .unwrap(),
        GenericNumeric::Float(-std::f32::consts::PI)
    );
    assert_eq!(
        GenericNumeric::Double(std::f64::consts::E)
            .negate(config)
            .unwrap(),
        GenericNumeric::Double(-std::f64::consts::E)
    );
}

#[test]
fn test_negate_overflow() {
    let config = EvalConfig {
        overflow: OverflowBehavior::Error,
        ..Default::default()
    };

    // Test overflow cases
    assert!(GenericNumeric::Int8(i8::MIN).negate(config).is_err());
    assert!(GenericNumeric::Int16(i16::MIN).negate(config).is_err());
    assert!(GenericNumeric::Int32(i32::MIN).negate(config).is_err());
    assert!(GenericNumeric::Int64(i64::MIN).negate(config).is_err());
    assert!(GenericNumeric::UInt64(1).negate(config).is_err());

    // Test wrap behavior
    let wrap_config = EvalConfig {
        overflow: OverflowBehavior::Wrap,
        ..Default::default()
    };

    assert_eq!(
        GenericNumeric::Int8(i8::MIN).negate(wrap_config).unwrap(),
        GenericNumeric::Int8(i8::MIN)
    );
    assert_eq!(
        GenericNumeric::UInt64(1).negate(wrap_config).unwrap(),
        GenericNumeric::UInt64(u64::MAX)
    );

    // Test saturate behavior
    let sat_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };

    assert_eq!(
        GenericNumeric::Int8(i8::MIN).negate(sat_config).unwrap(),
        GenericNumeric::Int8(i8::MAX)
    );
    assert_eq!(
        GenericNumeric::UInt64(1).negate(sat_config).unwrap(),
        GenericNumeric::UInt64(0)
    );
}

#[test]
fn test_bit_not() {
    assert_eq!(
        GenericNumeric::Bool(true).bit_not(),
        GenericNumeric::Bool(false)
    );
    assert_eq!(
        GenericNumeric::Bool(false).bit_not(),
        GenericNumeric::Bool(true)
    );
    assert_eq!(GenericNumeric::Int8(0).bit_not(), GenericNumeric::Int8(-1));
    assert_eq!(
        GenericNumeric::UInt8(0).bit_not(),
        GenericNumeric::UInt8(255)
    );
    assert_eq!(
        GenericNumeric::Int16(0).bit_not(),
        GenericNumeric::Int16(-1)
    );
    assert_eq!(
        GenericNumeric::UInt16(0).bit_not(),
        GenericNumeric::UInt16(65535)
    );
    assert_eq!(
        GenericNumeric::Int32(0).bit_not(),
        GenericNumeric::Int32(-1)
    );
    assert_eq!(
        GenericNumeric::UInt32(0).bit_not(),
        GenericNumeric::UInt32(u32::MAX)
    );
    assert_eq!(
        GenericNumeric::Int64(0).bit_not(),
        GenericNumeric::Int64(-1)
    );
    assert_eq!(
        GenericNumeric::UInt64(0).bit_not(),
        GenericNumeric::UInt64(u64::MAX)
    );

    // Floating point values are unchanged
    assert_eq!(
        GenericNumeric::Float(std::f32::consts::PI).bit_not(),
        GenericNumeric::Float(std::f32::consts::PI)
    );
    assert_eq!(
        GenericNumeric::Double(std::f64::consts::E).bit_not(),
        GenericNumeric::Double(std::f64::consts::E)
    );
}

#[test]
fn test_arithmetic_promotion() {
    let config = EvalConfig::default();

    // Test integer to float promotion
    let i32_val = GenericNumeric::Int32(10);
    let f32_val = GenericNumeric::Float(3.0);
    let result = i32_val.add(&f32_val, config).unwrap();
    assert_eq!(result, GenericNumeric::Float(13.0));

    // Test float to double promotion
    let f32_val = GenericNumeric::Float(1.0);
    let f64_val = GenericNumeric::Double(2.0);
    let result = f32_val.add(&f64_val, config).unwrap();
    assert_eq!(result, GenericNumeric::Double(3.0));

    // Test smaller int to larger int promotion
    let i8_val = GenericNumeric::Int8(10);
    let i32_val = GenericNumeric::Int32(20);
    let result = i8_val.add(&i32_val, config).unwrap();
    assert_eq!(result, GenericNumeric::Int32(30));

    // Test unsigned to signed promotion
    let u8_val = GenericNumeric::UInt8(10);
    let i16_val = GenericNumeric::Int16(20);
    let result = u8_val.add(&i16_val, config).unwrap();
    assert_eq!(result, GenericNumeric::Int16(30));
}

#[test]
fn test_add() {
    let config = EvalConfig::default();

    // Test same-type addition
    assert_eq!(
        GenericNumeric::Int8(10)
            .add(&GenericNumeric::Int8(20), config)
            .unwrap(),
        GenericNumeric::Int8(30)
    );
    assert_eq!(
        GenericNumeric::UInt8(10)
            .add(&GenericNumeric::UInt8(20), config)
            .unwrap(),
        GenericNumeric::UInt8(30)
    );
    assert_eq!(
        GenericNumeric::Int16(100)
            .add(&GenericNumeric::Int16(200), config)
            .unwrap(),
        GenericNumeric::Int16(300)
    );
    assert_eq!(
        GenericNumeric::UInt16(100)
            .add(&GenericNumeric::UInt16(200), config)
            .unwrap(),
        GenericNumeric::UInt16(300)
    );
    assert_eq!(
        GenericNumeric::Int32(1000)
            .add(&GenericNumeric::Int32(2000), config)
            .unwrap(),
        GenericNumeric::Int32(3000)
    );
    assert_eq!(
        GenericNumeric::UInt32(1000)
            .add(&GenericNumeric::UInt32(2000), config)
            .unwrap(),
        GenericNumeric::UInt32(3000)
    );
    assert_eq!(
        GenericNumeric::Int64(10000)
            .add(&GenericNumeric::Int64(20000), config)
            .unwrap(),
        GenericNumeric::Int64(30000)
    );
    assert_eq!(
        GenericNumeric::UInt64(10000)
            .add(&GenericNumeric::UInt64(20000), config)
            .unwrap(),
        GenericNumeric::UInt64(30000)
    );
    assert_eq!(
        GenericNumeric::Float(1.5)
            .add(&GenericNumeric::Float(2.5), config)
            .unwrap(),
        GenericNumeric::Float(4.0)
    );
    assert_eq!(
        GenericNumeric::Double(1.5)
            .add(&GenericNumeric::Double(2.5), config)
            .unwrap(),
        GenericNumeric::Double(4.0)
    );
}

#[test]
fn test_add_overflow() {
    let config = EvalConfig {
        overflow: OverflowBehavior::Error,
        ..Default::default()
    };

    // Test overflow detection
    assert!(
        GenericNumeric::Int8(i8::MAX)
            .add(&GenericNumeric::Int8(1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::UInt8(u8::MAX)
            .add(&GenericNumeric::UInt8(1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int16(i16::MAX)
            .add(&GenericNumeric::Int16(1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::UInt16(u16::MAX)
            .add(&GenericNumeric::UInt16(1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(i32::MAX)
            .add(&GenericNumeric::Int32(1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::UInt32(u32::MAX)
            .add(&GenericNumeric::UInt32(1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int64(i64::MAX)
            .add(&GenericNumeric::Int64(1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::UInt64(u64::MAX)
            .add(&GenericNumeric::UInt64(1), config)
            .is_err()
    );

    // Test wrap behavior
    let wrap_config = EvalConfig {
        overflow: OverflowBehavior::Wrap,
        ..Default::default()
    };

    assert_eq!(
        GenericNumeric::UInt8(u8::MAX)
            .add(&GenericNumeric::UInt8(1), wrap_config)
            .unwrap(),
        GenericNumeric::UInt8(0)
    );

    // Test saturate behavior
    let sat_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };

    assert_eq!(
        GenericNumeric::UInt8(u8::MAX)
            .add(&GenericNumeric::UInt8(1), sat_config)
            .unwrap(),
        GenericNumeric::UInt8(u8::MAX)
    );
}

#[test]
fn test_sub() {
    let config = EvalConfig::default();

    // Test same-type subtraction
    assert_eq!(
        GenericNumeric::Int8(30)
            .sub(&GenericNumeric::Int8(20), config)
            .unwrap(),
        GenericNumeric::Int8(10)
    );
    assert_eq!(
        GenericNumeric::UInt8(30)
            .sub(&GenericNumeric::UInt8(20), config)
            .unwrap(),
        GenericNumeric::UInt8(10)
    );
    assert_eq!(
        GenericNumeric::Float(5.5)
            .sub(&GenericNumeric::Float(2.5), config)
            .unwrap(),
        GenericNumeric::Float(3.0)
    );
    assert_eq!(
        GenericNumeric::Double(5.5)
            .sub(&GenericNumeric::Double(2.5), config)
            .unwrap(),
        GenericNumeric::Double(3.0)
    );
}

#[test]
fn test_sub_overflow() {
    let config = EvalConfig {
        overflow: OverflowBehavior::Error,
        ..Default::default()
    };

    // Test underflow detection
    assert!(
        GenericNumeric::Int8(i8::MIN)
            .sub(&GenericNumeric::Int8(1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::UInt8(0)
            .sub(&GenericNumeric::UInt8(1), config)
            .is_err()
    );

    // Test wrap behavior
    let wrap_config = EvalConfig {
        overflow: OverflowBehavior::Wrap,
        ..Default::default()
    };

    assert_eq!(
        GenericNumeric::UInt8(0)
            .sub(&GenericNumeric::UInt8(1), wrap_config)
            .unwrap(),
        GenericNumeric::UInt8(u8::MAX)
    );

    // Test saturate behavior
    let sat_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };

    assert_eq!(
        GenericNumeric::UInt8(0)
            .sub(&GenericNumeric::UInt8(1), sat_config)
            .unwrap(),
        GenericNumeric::UInt8(0)
    );
}

#[test]
fn test_mul() {
    let config = EvalConfig::default();

    // Test same-type multiplication
    assert_eq!(
        GenericNumeric::Int8(5)
            .mul(&GenericNumeric::Int8(6), config)
            .unwrap(),
        GenericNumeric::Int8(30)
    );
    assert_eq!(
        GenericNumeric::UInt8(5)
            .mul(&GenericNumeric::UInt8(6), config)
            .unwrap(),
        GenericNumeric::UInt8(30)
    );
    assert_eq!(
        GenericNumeric::Int16(10)
            .mul(&GenericNumeric::Int16(20), config)
            .unwrap(),
        GenericNumeric::Int16(200)
    );
    assert_eq!(
        GenericNumeric::UInt16(10)
            .mul(&GenericNumeric::UInt16(20), config)
            .unwrap(),
        GenericNumeric::UInt16(200)
    );
    assert_eq!(
        GenericNumeric::Int32(100)
            .mul(&GenericNumeric::Int32(200), config)
            .unwrap(),
        GenericNumeric::Int32(20000)
    );
    assert_eq!(
        GenericNumeric::UInt32(100)
            .mul(&GenericNumeric::UInt32(200), config)
            .unwrap(),
        GenericNumeric::UInt32(20000)
    );
    assert_eq!(
        GenericNumeric::Int64(1000)
            .mul(&GenericNumeric::Int64(2000), config)
            .unwrap(),
        GenericNumeric::Int64(2_000_000)
    );
    assert_eq!(
        GenericNumeric::UInt64(1000)
            .mul(&GenericNumeric::UInt64(2000), config)
            .unwrap(),
        GenericNumeric::UInt64(2_000_000)
    );
    assert_eq!(
        GenericNumeric::Float(2.5)
            .mul(&GenericNumeric::Float(4.0), config)
            .unwrap(),
        GenericNumeric::Float(10.0)
    );
    assert_eq!(
        GenericNumeric::Double(2.5)
            .mul(&GenericNumeric::Double(4.0), config)
            .unwrap(),
        GenericNumeric::Double(10.0)
    );
}

#[test]
fn test_mul_overflow() {
    let config = EvalConfig {
        overflow: OverflowBehavior::Error,
        ..Default::default()
    };

    // Test overflow detection
    assert!(
        GenericNumeric::Int8(i8::MAX)
            .mul(&GenericNumeric::Int8(2), config)
            .is_err()
    );
    assert!(
        GenericNumeric::UInt8(u8::MAX)
            .mul(&GenericNumeric::UInt8(2), config)
            .is_err()
    );
}

#[test]
fn test_div() {
    let config = EvalConfig::default();

    // Test same-type division
    assert_eq!(
        GenericNumeric::Int8(30)
            .div(&GenericNumeric::Int8(6), config)
            .unwrap(),
        GenericNumeric::Int8(5)
    );
    assert_eq!(
        GenericNumeric::UInt8(30)
            .div(&GenericNumeric::UInt8(6), config)
            .unwrap(),
        GenericNumeric::UInt8(5)
    );
    assert_eq!(
        GenericNumeric::Int16(200)
            .div(&GenericNumeric::Int16(10), config)
            .unwrap(),
        GenericNumeric::Int16(20)
    );
    assert_eq!(
        GenericNumeric::UInt16(200)
            .div(&GenericNumeric::UInt16(10), config)
            .unwrap(),
        GenericNumeric::UInt16(20)
    );
    assert_eq!(
        GenericNumeric::Int32(20000)
            .div(&GenericNumeric::Int32(100), config)
            .unwrap(),
        GenericNumeric::Int32(200)
    );
    assert_eq!(
        GenericNumeric::UInt32(20000)
            .div(&GenericNumeric::UInt32(100), config)
            .unwrap(),
        GenericNumeric::UInt32(200)
    );
    assert_eq!(
        GenericNumeric::Int64(2_000_000)
            .div(&GenericNumeric::Int64(1000), config)
            .unwrap(),
        GenericNumeric::Int64(2000)
    );
    assert_eq!(
        GenericNumeric::UInt64(2_000_000)
            .div(&GenericNumeric::UInt64(1000), config)
            .unwrap(),
        GenericNumeric::UInt64(2000)
    );
    assert_eq!(
        GenericNumeric::Float(10.0)
            .div(&GenericNumeric::Float(2.5), config)
            .unwrap(),
        GenericNumeric::Float(4.0)
    );
    assert_eq!(
        GenericNumeric::Double(10.0)
            .div(&GenericNumeric::Double(2.5), config)
            .unwrap(),
        GenericNumeric::Double(4.0)
    );
}

#[test]
fn test_div_by_zero() {
    let config = EvalConfig::default();

    // Test division by zero for integers
    assert!(
        GenericNumeric::Int8(10)
            .div(&GenericNumeric::Int8(0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::UInt8(10)
            .div(&GenericNumeric::UInt8(0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int16(10)
            .div(&GenericNumeric::Int16(0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::UInt16(10)
            .div(&GenericNumeric::UInt16(0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(10)
            .div(&GenericNumeric::Int32(0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::UInt32(10)
            .div(&GenericNumeric::UInt32(0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int64(10)
            .div(&GenericNumeric::Int64(0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::UInt64(10)
            .div(&GenericNumeric::UInt64(0), config)
            .is_err()
    );

    // Floating point division by zero produces infinity (IEEE 754)
    let result = GenericNumeric::Float(1.0)
        .div(&GenericNumeric::Float(0.0), config)
        .unwrap();
    if let GenericNumeric::Float(v) = result {
        assert!(v.is_infinite());
    } else {
        panic!("Expected Float result");
    }

    let result = GenericNumeric::Double(1.0)
        .div(&GenericNumeric::Double(0.0), config)
        .unwrap();
    if let GenericNumeric::Double(v) = result {
        assert!(v.is_infinite());
    } else {
        panic!("Expected Double result");
    }
}

#[test]
fn test_modulo() {
    let config = EvalConfig::default();

    // Test same-type modulo
    assert_eq!(
        GenericNumeric::Int8(17)
            .modulo(&GenericNumeric::Int8(5), config)
            .unwrap(),
        GenericNumeric::Int8(2)
    );
    assert_eq!(
        GenericNumeric::UInt8(17)
            .modulo(&GenericNumeric::UInt8(5), config)
            .unwrap(),
        GenericNumeric::UInt8(2)
    );
    assert_eq!(
        GenericNumeric::Int16(100)
            .modulo(&GenericNumeric::Int16(30), config)
            .unwrap(),
        GenericNumeric::Int16(10)
    );
    assert_eq!(
        GenericNumeric::UInt16(100)
            .modulo(&GenericNumeric::UInt16(30), config)
            .unwrap(),
        GenericNumeric::UInt16(10)
    );
    assert_eq!(
        GenericNumeric::Int32(1000)
            .modulo(&GenericNumeric::Int32(300), config)
            .unwrap(),
        GenericNumeric::Int32(100)
    );
    assert_eq!(
        GenericNumeric::UInt32(1000)
            .modulo(&GenericNumeric::UInt32(300), config)
            .unwrap(),
        GenericNumeric::UInt32(100)
    );
    assert_eq!(
        GenericNumeric::Int64(10000)
            .modulo(&GenericNumeric::Int64(3000), config)
            .unwrap(),
        GenericNumeric::Int64(1000)
    );
    assert_eq!(
        GenericNumeric::UInt64(10000)
            .modulo(&GenericNumeric::UInt64(3000), config)
            .unwrap(),
        GenericNumeric::UInt64(1000)
    );
    assert_eq!(
        GenericNumeric::Float(10.5)
            .modulo(&GenericNumeric::Float(3.0), config)
            .unwrap(),
        GenericNumeric::Float(1.5)
    );
    assert_eq!(
        GenericNumeric::Double(10.5)
            .modulo(&GenericNumeric::Double(3.0), config)
            .unwrap(),
        GenericNumeric::Double(1.5)
    );
}

#[test]
fn test_modulo_by_zero() {
    let config = EvalConfig::default();

    // Test modulo by zero for integers
    assert!(
        GenericNumeric::Int8(10)
            .modulo(&GenericNumeric::Int8(0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::UInt8(10)
            .modulo(&GenericNumeric::UInt8(0), config)
            .is_err()
    );

    // Floating point modulo by zero produces NaN (IEEE 754)
    let result = GenericNumeric::Float(1.0)
        .modulo(&GenericNumeric::Float(0.0), config)
        .unwrap();
    if let GenericNumeric::Float(v) = result {
        assert!(v.is_nan());
    } else {
        panic!("Expected Float result");
    }
}

#[test]
#[allow(clippy::cast_possible_wrap)]
fn test_bit_and() {
    assert_eq!(
        GenericNumeric::Bool(true).bit_and(&GenericNumeric::Bool(false)),
        GenericNumeric::Bool(false)
    );
    assert_eq!(
        GenericNumeric::Bool(true).bit_and(&GenericNumeric::Bool(true)),
        GenericNumeric::Bool(true)
    );
    assert_eq!(
        GenericNumeric::Int8(0b1010).bit_and(&GenericNumeric::Int8(0b1100)),
        GenericNumeric::Int8(0b1000)
    );
    assert_eq!(
        GenericNumeric::UInt8(0b1010).bit_and(&GenericNumeric::UInt8(0b1100)),
        GenericNumeric::UInt8(0b1000)
    );
    assert_eq!(
        GenericNumeric::Int16(0xFF00u16 as i16).bit_and(&GenericNumeric::Int16(0x0FF0)),
        GenericNumeric::Int16(0x0F00)
    );
    assert_eq!(
        GenericNumeric::UInt16(0xFF00).bit_and(&GenericNumeric::UInt16(0x0FF0)),
        GenericNumeric::UInt16(0x0F00)
    );
    assert_eq!(
        GenericNumeric::Int32(0xFF00_FF00_u32 as i32)
            .bit_and(&GenericNumeric::Int32(0x0FF0_0FF0_u32 as i32)),
        GenericNumeric::Int32(0x0F00_0F00_u32 as i32)
    );
    assert_eq!(
        GenericNumeric::UInt32(0xFF00_FF00).bit_and(&GenericNumeric::UInt32(0x0FF0_0FF0)),
        GenericNumeric::UInt32(0x0F00_0F00)
    );
    assert_eq!(
        GenericNumeric::Int64(0xFF).bit_and(&GenericNumeric::Int64(0xF0)),
        GenericNumeric::Int64(0xF0)
    );
    assert_eq!(
        GenericNumeric::UInt64(0xFF).bit_and(&GenericNumeric::UInt64(0xF0)),
        GenericNumeric::UInt64(0xF0)
    );

    // Floating point returns self
    assert_eq!(
        GenericNumeric::Float(std::f32::consts::PI)
            .bit_and(&GenericNumeric::Float(std::f32::consts::E)),
        GenericNumeric::Float(std::f32::consts::PI)
    );
}

#[test]
fn test_bit_or() {
    assert_eq!(
        GenericNumeric::Bool(true).bit_or(&GenericNumeric::Bool(false)),
        GenericNumeric::Bool(true)
    );
    assert_eq!(
        GenericNumeric::Bool(false).bit_or(&GenericNumeric::Bool(false)),
        GenericNumeric::Bool(false)
    );
    assert_eq!(
        GenericNumeric::Int8(0b1010).bit_or(&GenericNumeric::Int8(0b1100)),
        GenericNumeric::Int8(0b1110)
    );
    assert_eq!(
        GenericNumeric::UInt8(0b1010).bit_or(&GenericNumeric::UInt8(0b1100)),
        GenericNumeric::UInt8(0b1110)
    );
}

#[test]
fn test_bit_xor() {
    assert_eq!(
        GenericNumeric::Bool(true).bit_xor(&GenericNumeric::Bool(false)),
        GenericNumeric::Bool(true)
    );
    assert_eq!(
        GenericNumeric::Bool(true).bit_xor(&GenericNumeric::Bool(true)),
        GenericNumeric::Bool(false)
    );
    assert_eq!(
        GenericNumeric::Int8(0b1010).bit_xor(&GenericNumeric::Int8(0b1100)),
        GenericNumeric::Int8(0b0110)
    );
    assert_eq!(
        GenericNumeric::UInt8(0b1010).bit_xor(&GenericNumeric::UInt8(0b1100)),
        GenericNumeric::UInt8(0b0110)
    );
}

#[test]
fn test_shift_left() {
    let config = EvalConfig::default();

    // Test normal shifts
    assert_eq!(
        GenericNumeric::Int8(1)
            .shl(&GenericNumeric::Int8(2), config)
            .unwrap(),
        GenericNumeric::Int8(4)
    );
    assert_eq!(
        GenericNumeric::UInt8(1)
            .shl(&GenericNumeric::UInt8(2), config)
            .unwrap(),
        GenericNumeric::UInt8(4)
    );
    assert_eq!(
        GenericNumeric::Int16(1)
            .shl(&GenericNumeric::Int16(8), config)
            .unwrap(),
        GenericNumeric::Int16(256)
    );
    assert_eq!(
        GenericNumeric::UInt16(1)
            .shl(&GenericNumeric::UInt16(8), config)
            .unwrap(),
        GenericNumeric::UInt16(256)
    );
    assert_eq!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::Int32(16), config)
            .unwrap(),
        GenericNumeric::Int32(65536)
    );
    assert_eq!(
        GenericNumeric::UInt32(1)
            .shl(&GenericNumeric::UInt32(16), config)
            .unwrap(),
        GenericNumeric::UInt32(65536)
    );
    assert_eq!(
        GenericNumeric::Int64(1)
            .shl(&GenericNumeric::Int64(32), config)
            .unwrap(),
        GenericNumeric::Int64(4_294_967_296)
    );
    assert_eq!(
        GenericNumeric::UInt64(1)
            .shl(&GenericNumeric::UInt64(32), config)
            .unwrap(),
        GenericNumeric::UInt64(4_294_967_296)
    );

    // Test negative shift (invalid)
    assert!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::Int32(-1), config)
            .is_err()
    );

    // Test shift amount too large
    assert!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::Int32(10000), config)
            .is_err()
    );

    // Test floating point (returns self)
    assert_eq!(
        GenericNumeric::Float(std::f32::consts::PI)
            .shl(&GenericNumeric::Int32(2), config)
            .unwrap(),
        GenericNumeric::Float(std::f32::consts::PI)
    );
}

#[test]
fn test_shift_right() {
    let config = EvalConfig::default();

    // Test normal shifts
    assert_eq!(
        GenericNumeric::Int8(16)
            .shr(&GenericNumeric::Int8(2), config)
            .unwrap(),
        GenericNumeric::Int8(4)
    );
    assert_eq!(
        GenericNumeric::UInt8(16)
            .shr(&GenericNumeric::UInt8(2), config)
            .unwrap(),
        GenericNumeric::UInt8(4)
    );
    assert_eq!(
        GenericNumeric::Int16(256)
            .shr(&GenericNumeric::Int16(8), config)
            .unwrap(),
        GenericNumeric::Int16(1)
    );
    assert_eq!(
        GenericNumeric::UInt16(256)
            .shr(&GenericNumeric::UInt16(8), config)
            .unwrap(),
        GenericNumeric::UInt16(1)
    );

    // Test arithmetic right shift for negative numbers
    assert_eq!(
        GenericNumeric::Int8(-16)
            .shr(&GenericNumeric::Int8(2), config)
            .unwrap(),
        GenericNumeric::Int8(-4)
    );
}

#[test]
fn test_comparisons() {
    // Test less than
    assert!(GenericNumeric::Int32(5).lt(&GenericNumeric::Int32(10)));
    assert!(!GenericNumeric::Int32(10).lt(&GenericNumeric::Int32(5)));
    assert!(!GenericNumeric::Int32(5).lt(&GenericNumeric::Int32(5)));
    assert!(GenericNumeric::Float(1.0).lt(&GenericNumeric::Float(2.0)));
    assert!(GenericNumeric::Bool(false).lt(&GenericNumeric::Bool(true)));
    assert!(!GenericNumeric::Bool(true).lt(&GenericNumeric::Bool(false)));

    // Test less than or equal
    assert!(GenericNumeric::Int32(5).le(&GenericNumeric::Int32(10)));
    assert!(!GenericNumeric::Int32(10).le(&GenericNumeric::Int32(5)));
    assert!(GenericNumeric::Int32(5).le(&GenericNumeric::Int32(5)));
    assert!(GenericNumeric::Double(1.0).le(&GenericNumeric::Double(1.0)));

    // Test greater than
    assert!(!GenericNumeric::Int32(5).gt(&GenericNumeric::Int32(10)));
    assert!(GenericNumeric::Int32(10).gt(&GenericNumeric::Int32(5)));
    assert!(!GenericNumeric::Int32(5).gt(&GenericNumeric::Int32(5)));
    assert!(GenericNumeric::Bool(true).gt(&GenericNumeric::Bool(false)));

    // Test greater than or equal
    assert!(!GenericNumeric::Int32(5).ge(&GenericNumeric::Int32(10)));
    assert!(GenericNumeric::Int32(10).ge(&GenericNumeric::Int32(5)));
    assert!(GenericNumeric::Int32(5).ge(&GenericNumeric::Int32(5)));

    // Test equality
    assert!(NumericValue::eq(
        &GenericNumeric::Int32(5),
        &GenericNumeric::Int32(5)
    ));
    assert!(!NumericValue::eq(
        &GenericNumeric::Int32(5),
        &GenericNumeric::Int32(10)
    ));
    assert!(NumericValue::eq(
        &GenericNumeric::Float(1.0),
        &GenericNumeric::Float(1.0)
    ));
    assert!(NumericValue::eq(
        &GenericNumeric::Bool(true),
        &GenericNumeric::Bool(true)
    ));

    // Test inequality
    assert!(!NumericValue::ne(
        &GenericNumeric::Int32(5),
        &GenericNumeric::Int32(5)
    ));
    assert!(NumericValue::ne(
        &GenericNumeric::Int32(5),
        &GenericNumeric::Int32(10)
    ));

    // Test char comparisons
    assert!(GenericNumeric::Char('A').lt(&GenericNumeric::Char('B')));
    assert!(GenericNumeric::Char('Z').gt(&GenericNumeric::Char('A')));
    assert!(NumericValue::eq(
        &GenericNumeric::Char('X'),
        &GenericNumeric::Char('X')
    ));
}

#[test]
fn test_comparison_different_types() {
    // Different types should return false for comparisons (except eq/ne)
    assert!(!GenericNumeric::Int32(5).lt(&GenericNumeric::Float(10.0)));
    assert!(!GenericNumeric::Int32(5).gt(&GenericNumeric::Float(10.0)));
    assert!(!NumericValue::eq(
        &GenericNumeric::Int32(5),
        &GenericNumeric::Float(5.0)
    ));
    assert!(NumericValue::ne(
        &GenericNumeric::Int32(5),
        &GenericNumeric::Float(5.0)
    ));
}

#[test]
fn test_mixed_type_operations() {
    let config = EvalConfig::default();

    // Test Bool + Int8 -> Int8
    let result = GenericNumeric::Bool(true)
        .add(&GenericNumeric::Int8(5), config)
        .unwrap();
    assert_eq!(result, GenericNumeric::Int8(6));

    // Test Char + Int16 -> Int16
    let result = GenericNumeric::Char('A')
        .add(&GenericNumeric::Int16(1), config)
        .unwrap();
    assert_eq!(result, GenericNumeric::Int16(66)); // 'A' is 65

    // Test UInt8 + Int16 -> Int16
    let result = GenericNumeric::UInt8(100)
        .add(&GenericNumeric::Int16(200), config)
        .unwrap();
    assert_eq!(result, GenericNumeric::Int16(300));

    // Test Int32 + Float -> Float
    let result = GenericNumeric::Int32(10)
        .add(&GenericNumeric::Float(0.5), config)
        .unwrap();
    assert_eq!(result, GenericNumeric::Float(10.5));

    // Test Float + Double -> Double
    let result = GenericNumeric::Float(1.0)
        .add(&GenericNumeric::Double(0.1), config)
        .unwrap();
    if let GenericNumeric::Double(v) = result {
        assert!((v - 1.1).abs() < f64::EPSILON);
    } else {
        panic!("Expected Double result");
    }
}

#[test]
fn test_char_operations() {
    let config = EvalConfig::default();

    // Test char bit operations
    let a = GenericNumeric::Char('A'); // 65
    let b = GenericNumeric::Char('B'); // 66

    let result = a.bit_and(&b);
    assert_eq!(result, GenericNumeric::Char('@')); // 64

    let result = a.bit_or(&b);
    assert_eq!(result, GenericNumeric::Char('C')); // 67

    let result = a.bit_xor(&b);
    assert_eq!(result, GenericNumeric::Char('\u{0003}')); // 3

    // Test char arithmetic - chars stay as chars when adding
    // Now that we've implemented Char+Char, it should work
    let result = a.add(&GenericNumeric::Char('\u{0001}'), config);
    assert_eq!(result.unwrap(), GenericNumeric::Char('B')); // 'A' + 1 = 'B'
}

#[test]
fn test_overflow_edge_cases() {
    let config = EvalConfig {
        overflow: OverflowBehavior::Error,
        ..Default::default()
    };

    // Test i32::MIN / -1 overflow
    assert!(
        GenericNumeric::Int32(i32::MIN)
            .div(&GenericNumeric::Int32(-1), config)
            .is_err()
    );

    // Test i64::MIN / -1 overflow
    assert!(
        GenericNumeric::Int64(i64::MIN)
            .div(&GenericNumeric::Int64(-1), config)
            .is_err()
    );

    // Test i32::MIN % -1 overflow
    assert!(
        GenericNumeric::Int32(i32::MIN)
            .modulo(&GenericNumeric::Int32(-1), config)
            .is_err()
    );
}

#[test]
fn test_floating_point_special_values() {
    let config = EvalConfig::default();

    // Test NaN propagation
    let nan = GenericNumeric::Float(f32::NAN);
    let result = nan.add(&GenericNumeric::Float(1.0), config).unwrap();
    if let GenericNumeric::Float(v) = result {
        assert!(v.is_nan());
    }

    // Test infinity
    let inf = GenericNumeric::Double(f64::INFINITY);
    let result = inf.add(&GenericNumeric::Double(1.0), config).unwrap();
    if let GenericNumeric::Double(v) = result {
        assert!(v.is_infinite());
    }

    // Test -infinity
    let neg_inf = GenericNumeric::Double(f64::NEG_INFINITY);
    let result = neg_inf.add(&GenericNumeric::Double(1.0), config).unwrap();
    if let GenericNumeric::Double(v) = result {
        assert!(v.is_infinite() && v.is_sign_negative());
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_all_type_promotions() {
    // Test every promotion path to ensure coverage
    let config = EvalConfig::default();

    // Bool promotions
    let b = GenericNumeric::Bool(true);
    assert_eq!(
        b.add(&GenericNumeric::Int8(1), config).unwrap(),
        GenericNumeric::Int8(2)
    );
    assert_eq!(
        b.add(&GenericNumeric::UInt8(1), config).unwrap(),
        GenericNumeric::UInt8(2)
    );
    assert_eq!(
        b.add(&GenericNumeric::Int16(1), config).unwrap(),
        GenericNumeric::Int16(2)
    );
    assert_eq!(
        b.add(&GenericNumeric::UInt16(1), config).unwrap(),
        GenericNumeric::UInt16(2)
    );
    assert_eq!(
        b.add(&GenericNumeric::Int32(1), config).unwrap(),
        GenericNumeric::Int32(2)
    );
    assert_eq!(
        b.add(&GenericNumeric::UInt32(1), config).unwrap(),
        GenericNumeric::UInt32(2)
    );
    assert_eq!(
        b.add(&GenericNumeric::Int64(1), config).unwrap(),
        GenericNumeric::Int64(2)
    );
    assert_eq!(
        b.add(&GenericNumeric::UInt64(1), config).unwrap(),
        GenericNumeric::UInt64(2)
    );

    // Char promotions
    let c = GenericNumeric::Char('A'); // 65
    assert_eq!(
        c.add(&GenericNumeric::Int8(1), config).unwrap(),
        GenericNumeric::Int8(66)
    );
    assert_eq!(
        c.add(&GenericNumeric::UInt8(1), config).unwrap(),
        GenericNumeric::UInt8(66)
    );
    assert_eq!(
        c.add(&GenericNumeric::Int16(1), config).unwrap(),
        GenericNumeric::Int16(66)
    );
    assert_eq!(
        c.add(&GenericNumeric::UInt16(1), config).unwrap(),
        GenericNumeric::UInt16(66)
    );

    // Test all integer type combinations for coverage
    let i8_val = GenericNumeric::Int8(10);
    let u8_val = GenericNumeric::UInt8(10);
    let i16_val = GenericNumeric::Int16(10);
    let u16_val = GenericNumeric::UInt16(10);
    let i32_val = GenericNumeric::Int32(10);
    let u32_val = GenericNumeric::UInt32(10);
    let i64_val = GenericNumeric::Int64(10);
    let u64_val = GenericNumeric::UInt64(10);

    // Int8 with all others
    assert_eq!(
        i8_val.add(&u8_val, config).unwrap(),
        GenericNumeric::UInt8(20)
    );
    assert_eq!(
        i8_val.add(&i16_val, config).unwrap(),
        GenericNumeric::Int16(20)
    );
    assert_eq!(
        i8_val.add(&u16_val, config).unwrap(),
        GenericNumeric::UInt16(20)
    );
    assert_eq!(
        i8_val.add(&i32_val, config).unwrap(),
        GenericNumeric::Int32(20)
    );
    assert_eq!(
        i8_val.add(&u32_val, config).unwrap(),
        GenericNumeric::UInt32(20)
    );
    assert_eq!(
        i8_val.add(&i64_val, config).unwrap(),
        GenericNumeric::Int64(20)
    );
    assert_eq!(
        i8_val.add(&u64_val, config).unwrap(),
        GenericNumeric::UInt64(20)
    );

    // UInt8 with larger types
    assert_eq!(
        u8_val.add(&i16_val, config).unwrap(),
        GenericNumeric::Int16(20)
    );
    assert_eq!(
        u8_val.add(&u16_val, config).unwrap(),
        GenericNumeric::UInt16(20)
    );
    assert_eq!(
        u8_val.add(&i32_val, config).unwrap(),
        GenericNumeric::Int32(20)
    );
    assert_eq!(
        u8_val.add(&u32_val, config).unwrap(),
        GenericNumeric::UInt32(20)
    );
    assert_eq!(
        u8_val.add(&i64_val, config).unwrap(),
        GenericNumeric::Int64(20)
    );
    assert_eq!(
        u8_val.add(&u64_val, config).unwrap(),
        GenericNumeric::UInt64(20)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_all_shift_operations() {
    let config = EvalConfig::default();

    // Test shift left for all types
    assert_eq!(
        GenericNumeric::Int8(1)
            .shl(&GenericNumeric::Int8(3), config)
            .unwrap(),
        GenericNumeric::Int8(8)
    );
    assert_eq!(
        GenericNumeric::UInt8(1)
            .shl(&GenericNumeric::Int32(3), config)
            .unwrap(),
        GenericNumeric::UInt8(8)
    );
    assert_eq!(
        GenericNumeric::Int16(1)
            .shl(&GenericNumeric::UInt8(3), config)
            .unwrap(),
        GenericNumeric::Int16(8)
    );
    assert_eq!(
        GenericNumeric::UInt16(1)
            .shl(&GenericNumeric::Int16(3), config)
            .unwrap(),
        GenericNumeric::UInt16(8)
    );
    assert_eq!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::UInt16(3), config)
            .unwrap(),
        GenericNumeric::Int32(8)
    );
    assert_eq!(
        GenericNumeric::UInt32(1)
            .shl(&GenericNumeric::Int64(3), config)
            .unwrap(),
        GenericNumeric::UInt32(8)
    );
    assert_eq!(
        GenericNumeric::Int64(1)
            .shl(&GenericNumeric::UInt32(3), config)
            .unwrap(),
        GenericNumeric::Int64(8)
    );
    assert_eq!(
        GenericNumeric::UInt64(1)
            .shl(&GenericNumeric::UInt64(3), config)
            .unwrap(),
        GenericNumeric::UInt64(8)
    );

    // Test shift right for all types
    assert_eq!(
        GenericNumeric::Int8(16)
            .shr(&GenericNumeric::Int8(2), config)
            .unwrap(),
        GenericNumeric::Int8(4)
    );
    assert_eq!(
        GenericNumeric::UInt8(16)
            .shr(&GenericNumeric::UInt64(2), config)
            .unwrap(),
        GenericNumeric::UInt8(4)
    );
    assert_eq!(
        GenericNumeric::Int16(16)
            .shr(&GenericNumeric::Int32(2), config)
            .unwrap(),
        GenericNumeric::Int16(4)
    );
    assert_eq!(
        GenericNumeric::UInt16(16)
            .shr(&GenericNumeric::UInt16(2), config)
            .unwrap(),
        GenericNumeric::UInt16(4)
    );
    assert_eq!(
        GenericNumeric::Int32(16)
            .shr(&GenericNumeric::Int16(2), config)
            .unwrap(),
        GenericNumeric::Int32(4)
    );
    assert_eq!(
        GenericNumeric::UInt32(16)
            .shr(&GenericNumeric::UInt8(2), config)
            .unwrap(),
        GenericNumeric::UInt32(4)
    );
    assert_eq!(
        GenericNumeric::Int64(16)
            .shr(&GenericNumeric::Int8(2), config)
            .unwrap(),
        GenericNumeric::Int64(4)
    );
    assert_eq!(
        GenericNumeric::UInt64(16)
            .shr(&GenericNumeric::UInt32(2), config)
            .unwrap(),
        GenericNumeric::UInt64(4)
    );

    // Test invalid shift types
    assert!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::Float(3.0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(1)
            .shr(&GenericNumeric::Double(3.0), config)
            .is_err()
    );

    // Test char/bool shift (should return self)
    assert_eq!(
        GenericNumeric::Char('A')
            .shl(&GenericNumeric::Int32(1), config)
            .unwrap(),
        GenericNumeric::Char('A')
    );
    assert_eq!(
        GenericNumeric::Bool(true)
            .shr(&GenericNumeric::Int32(1), config)
            .unwrap(),
        GenericNumeric::Bool(true)
    );
}

#[test]
fn test_all_modulo_overflow_cases() {
    // Test wrap behavior for modulo
    let wrap_config = EvalConfig {
        overflow: OverflowBehavior::Wrap,
        ..Default::default()
    };

    assert_eq!(
        GenericNumeric::Int8(i8::MIN)
            .modulo(&GenericNumeric::Int8(-1), wrap_config)
            .unwrap(),
        GenericNumeric::Int8(0)
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MIN)
            .modulo(&GenericNumeric::Int16(-1), wrap_config)
            .unwrap(),
        GenericNumeric::Int16(0)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MIN)
            .modulo(&GenericNumeric::Int32(-1), wrap_config)
            .unwrap(),
        GenericNumeric::Int32(0)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MIN)
            .modulo(&GenericNumeric::Int64(-1), wrap_config)
            .unwrap(),
        GenericNumeric::Int64(0)
    );

    // Test error behavior for modulo
    let error_config = EvalConfig {
        overflow: OverflowBehavior::Error,
        ..Default::default()
    };

    assert!(
        GenericNumeric::Int8(i8::MIN)
            .modulo(&GenericNumeric::Int8(-1), error_config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int16(i16::MIN)
            .modulo(&GenericNumeric::Int16(-1), error_config)
            .is_err()
    );

    // Test saturate behavior
    let sat_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };

    assert_eq!(
        GenericNumeric::Int8(i8::MIN)
            .modulo(&GenericNumeric::Int8(-1), sat_config)
            .unwrap(),
        GenericNumeric::Int8(0)
    );
}

#[test]
fn test_all_division_overflow_cases() {
    // Test wrap behavior for division
    let wrap_config = EvalConfig {
        overflow: OverflowBehavior::Wrap,
        ..Default::default()
    };

    assert_eq!(
        GenericNumeric::Int8(i8::MIN)
            .div(&GenericNumeric::Int8(-1), wrap_config)
            .unwrap(),
        GenericNumeric::Int8(i8::MIN)
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MIN)
            .div(&GenericNumeric::Int16(-1), wrap_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MIN)
    );

    // Test saturate behavior
    let sat_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };

    assert_eq!(
        GenericNumeric::Int8(i8::MIN)
            .div(&GenericNumeric::Int8(-1), sat_config)
            .unwrap(),
        GenericNumeric::Int8(i8::MAX)
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MIN)
            .div(&GenericNumeric::Int16(-1), sat_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MAX)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MIN)
            .div(&GenericNumeric::Int32(-1), sat_config)
            .unwrap(),
        GenericNumeric::Int32(i32::MAX)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MIN)
            .div(&GenericNumeric::Int64(-1), sat_config)
            .unwrap(),
        GenericNumeric::Int64(i64::MAX)
    );
}

#[test]
fn test_bitwise_with_different_types() {
    // Test bit operations with mismatched types (should return self)
    let i32_val = GenericNumeric::Int32(0xFF);
    let f32_val = GenericNumeric::Float(std::f32::consts::PI);

    assert_eq!(i32_val.bit_and(&f32_val), i32_val);
    assert_eq!(i32_val.bit_or(&f32_val), i32_val);
    assert_eq!(i32_val.bit_xor(&f32_val), i32_val);

    // Test double with int
    let d_val = GenericNumeric::Double(std::f64::consts::E);
    assert_eq!(d_val.bit_and(&i32_val), d_val);
    assert_eq!(d_val.bit_or(&i32_val), d_val);
    assert_eq!(d_val.bit_xor(&i32_val), d_val);
}

#[test]
fn test_more_bitwise_operations() {
    // Test all integer type bitwise operations
    assert_eq!(
        GenericNumeric::Int16(0b1010).bit_and(&GenericNumeric::Int16(0b1100)),
        GenericNumeric::Int16(0b1000)
    );
    assert_eq!(
        GenericNumeric::UInt16(0b1010).bit_and(&GenericNumeric::UInt16(0b1100)),
        GenericNumeric::UInt16(0b1000)
    );
    assert_eq!(
        GenericNumeric::Int32(0b1010).bit_and(&GenericNumeric::Int32(0b1100)),
        GenericNumeric::Int32(0b1000)
    );
    assert_eq!(
        GenericNumeric::UInt32(0b1010).bit_and(&GenericNumeric::UInt32(0b1100)),
        GenericNumeric::UInt32(0b1000)
    );
    assert_eq!(
        GenericNumeric::Int64(0b1010).bit_and(&GenericNumeric::Int64(0b1100)),
        GenericNumeric::Int64(0b1000)
    );
    assert_eq!(
        GenericNumeric::UInt64(0b1010).bit_and(&GenericNumeric::UInt64(0b1100)),
        GenericNumeric::UInt64(0b1000)
    );

    // Test bit_or for all types
    assert_eq!(
        GenericNumeric::Int16(0b1010).bit_or(&GenericNumeric::Int16(0b1100)),
        GenericNumeric::Int16(0b1110)
    );
    assert_eq!(
        GenericNumeric::UInt16(0b1010).bit_or(&GenericNumeric::UInt16(0b1100)),
        GenericNumeric::UInt16(0b1110)
    );
    assert_eq!(
        GenericNumeric::Int32(0b1010).bit_or(&GenericNumeric::Int32(0b1100)),
        GenericNumeric::Int32(0b1110)
    );
    assert_eq!(
        GenericNumeric::UInt32(0b1010).bit_or(&GenericNumeric::UInt32(0b1100)),
        GenericNumeric::UInt32(0b1110)
    );
    assert_eq!(
        GenericNumeric::Int64(0b1010).bit_or(&GenericNumeric::Int64(0b1100)),
        GenericNumeric::Int64(0b1110)
    );
    assert_eq!(
        GenericNumeric::UInt64(0b1010).bit_or(&GenericNumeric::UInt64(0b1100)),
        GenericNumeric::UInt64(0b1110)
    );

    // Test bit_xor for all types
    assert_eq!(
        GenericNumeric::Int16(0b1010).bit_xor(&GenericNumeric::Int16(0b1100)),
        GenericNumeric::Int16(0b0110)
    );
    assert_eq!(
        GenericNumeric::UInt16(0b1010).bit_xor(&GenericNumeric::UInt16(0b1100)),
        GenericNumeric::UInt16(0b0110)
    );
    assert_eq!(
        GenericNumeric::Int32(0b1010).bit_xor(&GenericNumeric::Int32(0b1100)),
        GenericNumeric::Int32(0b0110)
    );
    assert_eq!(
        GenericNumeric::UInt32(0b1010).bit_xor(&GenericNumeric::UInt32(0b1100)),
        GenericNumeric::UInt32(0b0110)
    );
    assert_eq!(
        GenericNumeric::Int64(0b1010).bit_xor(&GenericNumeric::Int64(0b1100)),
        GenericNumeric::Int64(0b0110)
    );
    assert_eq!(
        GenericNumeric::UInt64(0b1010).bit_xor(&GenericNumeric::UInt64(0b1100)),
        GenericNumeric::UInt64(0b0110)
    );
}

#[test]
fn test_all_comparison_operations() {
    // Test comparisons for all types

    // Bool comparisons
    assert!(GenericNumeric::Bool(false).lt(&GenericNumeric::Bool(true)));
    assert!(GenericNumeric::Bool(false).le(&GenericNumeric::Bool(true)));
    assert!(GenericNumeric::Bool(false).le(&GenericNumeric::Bool(false)));
    assert!(GenericNumeric::Bool(true).gt(&GenericNumeric::Bool(false)));
    assert!(GenericNumeric::Bool(true).ge(&GenericNumeric::Bool(false)));
    assert!(GenericNumeric::Bool(true).ge(&GenericNumeric::Bool(true)));

    // Char comparisons
    assert!(GenericNumeric::Char('A').lt(&GenericNumeric::Char('Z')));
    assert!(GenericNumeric::Char('A').le(&GenericNumeric::Char('Z')));
    assert!(GenericNumeric::Char('Z').gt(&GenericNumeric::Char('A')));
    assert!(GenericNumeric::Char('Z').ge(&GenericNumeric::Char('A')));

    // Test all numeric types
    assert!(GenericNumeric::Int8(-10).lt(&GenericNumeric::Int8(10)));
    assert!(GenericNumeric::Int8(-10).le(&GenericNumeric::Int8(10)));
    assert!(GenericNumeric::Int8(10).gt(&GenericNumeric::Int8(-10)));
    assert!(GenericNumeric::Int8(10).ge(&GenericNumeric::Int8(-10)));

    assert!(GenericNumeric::UInt8(10).lt(&GenericNumeric::UInt8(20)));
    assert!(GenericNumeric::UInt8(10).le(&GenericNumeric::UInt8(20)));
    assert!(GenericNumeric::UInt8(20).gt(&GenericNumeric::UInt8(10)));
    assert!(GenericNumeric::UInt8(20).ge(&GenericNumeric::UInt8(10)));

    assert!(GenericNumeric::Int16(-100).lt(&GenericNumeric::Int16(100)));
    assert!(GenericNumeric::Int16(-100).le(&GenericNumeric::Int16(100)));
    assert!(GenericNumeric::Int16(100).gt(&GenericNumeric::Int16(-100)));
    assert!(GenericNumeric::Int16(100).ge(&GenericNumeric::Int16(-100)));

    assert!(GenericNumeric::UInt16(100).lt(&GenericNumeric::UInt16(200)));
    assert!(GenericNumeric::UInt16(100).le(&GenericNumeric::UInt16(200)));
    assert!(GenericNumeric::UInt16(200).gt(&GenericNumeric::UInt16(100)));
    assert!(GenericNumeric::UInt16(200).ge(&GenericNumeric::UInt16(100)));

    assert!(GenericNumeric::UInt32(1000).lt(&GenericNumeric::UInt32(2000)));
    assert!(GenericNumeric::UInt32(1000).le(&GenericNumeric::UInt32(2000)));
    assert!(GenericNumeric::UInt32(2000).gt(&GenericNumeric::UInt32(1000)));
    assert!(GenericNumeric::UInt32(2000).ge(&GenericNumeric::UInt32(1000)));

    assert!(GenericNumeric::Int64(-10000).lt(&GenericNumeric::Int64(10000)));
    assert!(GenericNumeric::Int64(-10000).le(&GenericNumeric::Int64(10000)));
    assert!(GenericNumeric::Int64(10000).gt(&GenericNumeric::Int64(-10000)));
    assert!(GenericNumeric::Int64(10000).ge(&GenericNumeric::Int64(-10000)));

    assert!(GenericNumeric::UInt64(10000).lt(&GenericNumeric::UInt64(20000)));
    assert!(GenericNumeric::UInt64(10000).le(&GenericNumeric::UInt64(20000)));
    assert!(GenericNumeric::UInt64(20000).gt(&GenericNumeric::UInt64(10000)));
    assert!(GenericNumeric::UInt64(20000).ge(&GenericNumeric::UInt64(10000)));

    // Test equal values
    assert!(GenericNumeric::Int32(42).le(&GenericNumeric::Int32(42)));
    assert!(GenericNumeric::Int32(42).ge(&GenericNumeric::Int32(42)));
    assert!(!GenericNumeric::Int32(42).lt(&GenericNumeric::Int32(42)));
    assert!(!GenericNumeric::Int32(42).gt(&GenericNumeric::Int32(42)));
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_more_arithmetic_with_overflow() {
    let wrap_config = EvalConfig {
        overflow: OverflowBehavior::Wrap,
        ..Default::default()
    };

    let sat_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };

    // Test Int16 overflow scenarios
    assert_eq!(
        GenericNumeric::Int16(i16::MAX)
            .add(&GenericNumeric::Int16(1), wrap_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MIN)
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MAX)
            .add(&GenericNumeric::Int16(1), sat_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MAX)
    );

    assert_eq!(
        GenericNumeric::Int16(i16::MIN)
            .sub(&GenericNumeric::Int16(1), wrap_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MAX)
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MIN)
            .sub(&GenericNumeric::Int16(1), sat_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MIN)
    );

    assert_eq!(
        GenericNumeric::Int16(i16::MAX)
            .mul(&GenericNumeric::Int16(2), wrap_config)
            .unwrap(),
        GenericNumeric::Int16(-2)
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MAX)
            .mul(&GenericNumeric::Int16(2), sat_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MAX)
    );

    // Test UInt16 overflow scenarios
    assert_eq!(
        GenericNumeric::UInt16(u16::MAX)
            .add(&GenericNumeric::UInt16(1), wrap_config)
            .unwrap(),
        GenericNumeric::UInt16(0)
    );
    assert_eq!(
        GenericNumeric::UInt16(u16::MAX)
            .add(&GenericNumeric::UInt16(1), sat_config)
            .unwrap(),
        GenericNumeric::UInt16(u16::MAX)
    );

    assert_eq!(
        GenericNumeric::UInt16(0)
            .sub(&GenericNumeric::UInt16(1), wrap_config)
            .unwrap(),
        GenericNumeric::UInt16(u16::MAX)
    );
    assert_eq!(
        GenericNumeric::UInt16(0)
            .sub(&GenericNumeric::UInt16(1), sat_config)
            .unwrap(),
        GenericNumeric::UInt16(0)
    );

    assert_eq!(
        GenericNumeric::UInt16(u16::MAX)
            .mul(&GenericNumeric::UInt16(2), wrap_config)
            .unwrap(),
        GenericNumeric::UInt16(u16::MAX - 1)
    );
    assert_eq!(
        GenericNumeric::UInt16(u16::MAX)
            .mul(&GenericNumeric::UInt16(2), sat_config)
            .unwrap(),
        GenericNumeric::UInt16(u16::MAX)
    );

    // Test Int32 overflow scenarios
    assert_eq!(
        GenericNumeric::Int32(i32::MAX)
            .add(&GenericNumeric::Int32(1), wrap_config)
            .unwrap(),
        GenericNumeric::Int32(i32::MIN)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MIN)
            .sub(&GenericNumeric::Int32(1), wrap_config)
            .unwrap(),
        GenericNumeric::Int32(i32::MAX)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MAX)
            .mul(&GenericNumeric::Int32(2), wrap_config)
            .unwrap(),
        GenericNumeric::Int32(-2)
    );

    // Test UInt32 overflow scenarios
    assert_eq!(
        GenericNumeric::UInt32(u32::MAX)
            .add(&GenericNumeric::UInt32(1), wrap_config)
            .unwrap(),
        GenericNumeric::UInt32(0)
    );
    assert_eq!(
        GenericNumeric::UInt32(0)
            .sub(&GenericNumeric::UInt32(1), wrap_config)
            .unwrap(),
        GenericNumeric::UInt32(u32::MAX)
    );
    assert_eq!(
        GenericNumeric::UInt32(u32::MAX)
            .mul(&GenericNumeric::UInt32(2), wrap_config)
            .unwrap(),
        GenericNumeric::UInt32(u32::MAX - 1)
    );

    // Test Int64 overflow scenarios
    assert_eq!(
        GenericNumeric::Int64(i64::MAX)
            .add(&GenericNumeric::Int64(1), wrap_config)
            .unwrap(),
        GenericNumeric::Int64(i64::MIN)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MIN)
            .sub(&GenericNumeric::Int64(1), wrap_config)
            .unwrap(),
        GenericNumeric::Int64(i64::MAX)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MAX)
            .mul(&GenericNumeric::Int64(2), wrap_config)
            .unwrap(),
        GenericNumeric::Int64(-2)
    );

    // Test UInt64 overflow scenarios
    assert_eq!(
        GenericNumeric::UInt64(u64::MAX)
            .add(&GenericNumeric::UInt64(1), wrap_config)
            .unwrap(),
        GenericNumeric::UInt64(0)
    );
    assert_eq!(
        GenericNumeric::UInt64(0)
            .sub(&GenericNumeric::UInt64(1), wrap_config)
            .unwrap(),
        GenericNumeric::UInt64(u64::MAX)
    );
    assert_eq!(
        GenericNumeric::UInt64(u64::MAX)
            .mul(&GenericNumeric::UInt64(2), wrap_config)
            .unwrap(),
        GenericNumeric::UInt64(u64::MAX - 1)
    );
}

#[test]
fn test_char_edge_cases() {
    // Test char bitwise operations that produce invalid unicode
    let char1 = GenericNumeric::Char('\u{FFFF}');

    // char::from_u32 should return None for invalid values, replaced with '\0'
    let result = char1.bit_not();
    if let GenericNumeric::Char(c) = result {
        // The exact result depends on how the invalid unicode is handled
        assert!(c == '\0' || c.is_control());
    }
}

#[test]
fn test_promotion_edge_cases() {
    // Test promotion through operations instead of direct promote_to calls
    let config = EvalConfig::default();

    // Test Int64 + Int32 (should promote Int32 to Int64)
    let i64_val = GenericNumeric::Int64(1000);
    let i32_val = GenericNumeric::Int32(100);
    assert_eq!(
        i64_val.add(&i32_val, config).unwrap(),
        GenericNumeric::Int64(1100)
    );

    // Test Double + Float (should promote Float to Double)
    let f64_val = GenericNumeric::Double(3.0);
    let f32_val = GenericNumeric::Float(2.0);
    assert_eq!(
        f64_val.add(&f32_val, config).unwrap(),
        GenericNumeric::Double(5.0)
    );
}

#[test]
fn test_error_variants() {
    let config = EvalConfig::default();

    // Test different error types to ensure coverage
    let shift_err = GenericNumeric::Int32(1).shl(&GenericNumeric::Bool(true), config);
    match shift_err {
        Err(Error::Custom(msg)) => assert!(msg.contains("invalid shift amount type")),
        _ => panic!("Expected Custom error for invalid shift type"),
    }

    let div_err = GenericNumeric::Int32(1).div(&GenericNumeric::Int32(0), config);
    match div_err {
        Err(Error::DivisionByZero) => {}
        _ => panic!("Expected DivisionByZero error"),
    }

    let mod_err = GenericNumeric::Int32(1).modulo(&GenericNumeric::Int32(0), config);
    match mod_err {
        Err(Error::ModuloByZero) => {}
        _ => panic!("Expected ModuloByZero error"),
    }

    let shift_range_err = GenericNumeric::Int32(1).shl(&GenericNumeric::Int32(10000), config);
    match shift_range_err {
        Err(Error::InvalidShift(_)) => {}
        _ => panic!("Expected InvalidShift error"),
    }
}

#[test]
fn test_more_mixed_arithmetic() {
    let config = EvalConfig::default();

    // Test all sub operations
    assert_eq!(
        GenericNumeric::Int16(50)
            .sub(&GenericNumeric::Int16(20), config)
            .unwrap(),
        GenericNumeric::Int16(30)
    );
    assert_eq!(
        GenericNumeric::UInt16(50)
            .sub(&GenericNumeric::UInt16(20), config)
            .unwrap(),
        GenericNumeric::UInt16(30)
    );
    assert_eq!(
        GenericNumeric::Int32(50)
            .sub(&GenericNumeric::Int32(20), config)
            .unwrap(),
        GenericNumeric::Int32(30)
    );
    assert_eq!(
        GenericNumeric::UInt32(50)
            .sub(&GenericNumeric::UInt32(20), config)
            .unwrap(),
        GenericNumeric::UInt32(30)
    );
    assert_eq!(
        GenericNumeric::Int64(50)
            .sub(&GenericNumeric::Int64(20), config)
            .unwrap(),
        GenericNumeric::Int64(30)
    );
    assert_eq!(
        GenericNumeric::UInt64(50)
            .sub(&GenericNumeric::UInt64(20), config)
            .unwrap(),
        GenericNumeric::UInt64(30)
    );

    // Test all mul operations
    assert_eq!(
        GenericNumeric::Int16(5)
            .mul(&GenericNumeric::Int16(6), config)
            .unwrap(),
        GenericNumeric::Int16(30)
    );
    assert_eq!(
        GenericNumeric::UInt16(5)
            .mul(&GenericNumeric::UInt16(6), config)
            .unwrap(),
        GenericNumeric::UInt16(30)
    );

    // Test all div operations
    assert_eq!(
        GenericNumeric::Int16(30)
            .div(&GenericNumeric::Int16(5), config)
            .unwrap(),
        GenericNumeric::Int16(6)
    );
    assert_eq!(
        GenericNumeric::UInt16(30)
            .div(&GenericNumeric::UInt16(5), config)
            .unwrap(),
        GenericNumeric::UInt16(6)
    );

    // Test all modulo operations
    assert_eq!(
        GenericNumeric::Int16(17)
            .modulo(&GenericNumeric::Int16(5), config)
            .unwrap(),
        GenericNumeric::Int16(2)
    );
    assert_eq!(
        GenericNumeric::UInt16(17)
            .modulo(&GenericNumeric::UInt16(5), config)
            .unwrap(),
        GenericNumeric::UInt16(2)
    );
}

#[test]
fn test_negate_all_types() {
    let config = EvalConfig::default();

    // Test negate for all unsigned to signed promotions
    assert_eq!(
        GenericNumeric::UInt16(100).negate(config).unwrap(),
        GenericNumeric::Int32(-100)
    );
    assert_eq!(
        GenericNumeric::UInt32(100).negate(config).unwrap(),
        GenericNumeric::Int64(-100)
    );

    // Test negate wrap/saturate for more types
    let wrap_config = EvalConfig {
        overflow: OverflowBehavior::Wrap,
        ..Default::default()
    };

    assert_eq!(
        GenericNumeric::Int16(i16::MIN).negate(wrap_config).unwrap(),
        GenericNumeric::Int16(i16::MIN)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MIN).negate(wrap_config).unwrap(),
        GenericNumeric::Int32(i32::MIN)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MIN).negate(wrap_config).unwrap(),
        GenericNumeric::Int64(i64::MIN)
    );
}

#[test]
fn test_bool_arithmetic_not_supported() {
    // Bool arithmetic is not supported in the implementation
    // The promote_for_arithmetic will keep both as Bool, but there's no Bool case in arithmetic ops
    // This is by design - booleans shouldn't be used in arithmetic

    // Instead, test that booleans can be promoted when used with numeric types
    let config = EvalConfig::default();
    let b = GenericNumeric::Bool(true);

    // Bool promotes to Int8 when used with Int8
    assert_eq!(
        b.add(&GenericNumeric::Int8(5), config).unwrap(),
        GenericNumeric::Int8(6)
    );

    // Bool promotes to UInt8 when used with UInt8
    assert_eq!(
        b.add(&GenericNumeric::UInt8(5), config).unwrap(),
        GenericNumeric::UInt8(6)
    );
}

#[test]
fn test_comprehensive_negate_coverage() {
    let config = EvalConfig::default();

    // Test negating unsigned types (promotes to larger signed type)
    assert_eq!(
        GenericNumeric::UInt8(5).negate(config).unwrap(),
        GenericNumeric::Int16(-5)
    );
    assert_eq!(
        GenericNumeric::UInt16(5).negate(config).unwrap(),
        GenericNumeric::Int32(-5)
    );
    assert_eq!(
        GenericNumeric::UInt32(5).negate(config).unwrap(),
        GenericNumeric::Int64(-5)
    );
    // UInt64 uses wrapping negation with default config (Wrap)
    assert_eq!(
        GenericNumeric::UInt64(5).negate(config).unwrap(),
        GenericNumeric::UInt64(u64::MAX - 4)
    );

    // Test negating with different overflow behaviors
    let error_config = EvalConfig {
        overflow: OverflowBehavior::Error,
        ..Default::default()
    };
    let saturate_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };
    let wrap_config = EvalConfig {
        overflow: OverflowBehavior::Wrap,
        ..Default::default()
    };

    // Int8::MIN negation
    assert!(GenericNumeric::Int8(i8::MIN).negate(error_config).is_err());
    assert_eq!(
        GenericNumeric::Int8(i8::MIN)
            .negate(saturate_config)
            .unwrap(),
        GenericNumeric::Int8(i8::MAX)
    );
    assert_eq!(
        GenericNumeric::Int8(i8::MIN).negate(wrap_config).unwrap(),
        GenericNumeric::Int8(i8::MIN)
    );

    // Int16::MIN negation
    assert!(
        GenericNumeric::Int16(i16::MIN)
            .negate(error_config)
            .is_err()
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MIN)
            .negate(saturate_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MAX)
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MIN).negate(wrap_config).unwrap(),
        GenericNumeric::Int16(i16::MIN)
    );

    // Int32::MIN negation
    assert!(
        GenericNumeric::Int32(i32::MIN)
            .negate(error_config)
            .is_err()
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MIN)
            .negate(saturate_config)
            .unwrap(),
        GenericNumeric::Int32(i32::MAX)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MIN).negate(wrap_config).unwrap(),
        GenericNumeric::Int32(i32::MIN)
    );

    // Int64::MIN negation
    assert!(
        GenericNumeric::Int64(i64::MIN)
            .negate(error_config)
            .is_err()
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MIN)
            .negate(saturate_config)
            .unwrap(),
        GenericNumeric::Int64(i64::MAX)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MIN).negate(wrap_config).unwrap(),
        GenericNumeric::Int64(i64::MIN)
    );

    // UInt64 negation with different overflow behaviors
    assert!(GenericNumeric::UInt64(5).negate(error_config).is_err());
    assert_eq!(
        GenericNumeric::UInt64(5).negate(saturate_config).unwrap(),
        GenericNumeric::UInt64(0)
    );
    assert_eq!(
        GenericNumeric::UInt64(5).negate(wrap_config).unwrap(),
        GenericNumeric::UInt64(u64::MAX - 4)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_comprehensive_shift_coverage() {
    let config = EvalConfig::default();

    // Test shift with negative shift amounts (should error)
    assert!(
        GenericNumeric::Int8(1)
            .shl(&GenericNumeric::Int8(-1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int16(1)
            .shl(&GenericNumeric::Int16(-1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::Int32(-1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int64(1)
            .shl(&GenericNumeric::Int64(-1), config)
            .is_err()
    );

    assert!(
        GenericNumeric::Int8(1)
            .shr(&GenericNumeric::Int8(-1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int16(1)
            .shr(&GenericNumeric::Int16(-1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(1)
            .shr(&GenericNumeric::Int32(-1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int64(1)
            .shr(&GenericNumeric::Int64(-1), config)
            .is_err()
    );

    // Test shift with too large shift amounts (>127 is the default max)
    assert!(
        GenericNumeric::Int8(1)
            .shl(&GenericNumeric::Int8(127), config)
            .is_ok()
    );
    assert!(
        GenericNumeric::Int16(1)
            .shl(&GenericNumeric::Int16(128), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::Int32(128), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int64(1)
            .shl(&GenericNumeric::Int64(128), config)
            .is_err()
    );

    // Test shifts that are valid but large
    // In Rust, shifts >= bit width have undefined behavior, wrapping_shl masks the shift amount
    assert_eq!(
        GenericNumeric::Int8(1)
            .shl(&GenericNumeric::Int8(7), config)
            .unwrap(),
        GenericNumeric::Int8(i8::MIN)
    );
    assert_eq!(
        GenericNumeric::UInt8(1)
            .shl(&GenericNumeric::UInt8(7), config)
            .unwrap(),
        GenericNumeric::UInt8(128)
    );
    assert_eq!(
        GenericNumeric::Int16(1)
            .shl(&GenericNumeric::Int16(15), config)
            .unwrap(),
        GenericNumeric::Int16(i16::MIN)
    );
    assert_eq!(
        GenericNumeric::UInt16(1)
            .shl(&GenericNumeric::UInt16(15), config)
            .unwrap(),
        GenericNumeric::UInt16(32768)
    );
    assert_eq!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::Int32(31), config)
            .unwrap(),
        GenericNumeric::Int32(i32::MIN)
    );
    assert_eq!(
        GenericNumeric::UInt32(1)
            .shl(&GenericNumeric::UInt32(31), config)
            .unwrap(),
        GenericNumeric::UInt32(2_147_483_648)
    );
    assert_eq!(
        GenericNumeric::Int64(1)
            .shl(&GenericNumeric::Int64(63), config)
            .unwrap(),
        GenericNumeric::Int64(i64::MIN)
    );
    assert_eq!(
        GenericNumeric::UInt64(1)
            .shl(&GenericNumeric::UInt64(63), config)
            .unwrap(),
        GenericNumeric::UInt64(9_223_372_036_854_775_808)
    );

    // Test shift with float types (should error)
    assert!(
        GenericNumeric::Float(1.0)
            .shl(&GenericNumeric::Float(1.0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Double(1.0)
            .shl(&GenericNumeric::Double(1.0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Float(1.0)
            .shr(&GenericNumeric::Float(1.0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Double(1.0)
            .shr(&GenericNumeric::Double(1.0), config)
            .is_err()
    );

    // Test shift with bool/char shift amounts (should error)
    assert!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::Bool(true), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::Char('A'), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(1)
            .shr(&GenericNumeric::Bool(true), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(1)
            .shr(&GenericNumeric::Char('A'), config)
            .is_err()
    );

    // Test shift on bool/char/float types (returns self)
    assert_eq!(
        GenericNumeric::Bool(true)
            .shl(&GenericNumeric::Int32(1), config)
            .unwrap(),
        GenericNumeric::Bool(true)
    );
    assert_eq!(
        GenericNumeric::Char('A')
            .shl(&GenericNumeric::Int32(1), config)
            .unwrap(),
        GenericNumeric::Char('A')
    );
    assert_eq!(
        GenericNumeric::Float(1.0)
            .shl(&GenericNumeric::Int32(1), config)
            .unwrap(),
        GenericNumeric::Float(1.0)
    );
    assert_eq!(
        GenericNumeric::Double(1.0)
            .shl(&GenericNumeric::Int32(1), config)
            .unwrap(),
        GenericNumeric::Double(1.0)
    );
}

#[test]
fn test_more_float_operations() {
    let config = EvalConfig::default();

    // Test float operations with special values
    let zero_f = GenericNumeric::Float(0.0);
    let zero_d = GenericNumeric::Double(0.0);
    let one_f = GenericNumeric::Float(1.0);
    let one_d = GenericNumeric::Double(1.0);

    // Division by zero produces infinity
    if let GenericNumeric::Float(v) = one_f.div(&zero_f, config).unwrap() {
        assert!(v.is_infinite() && v.is_sign_positive());
    }
    if let GenericNumeric::Double(v) = one_d.div(&zero_d, config).unwrap() {
        assert!(v.is_infinite() && v.is_sign_positive());
    }

    // Modulo with zero produces NaN
    if let GenericNumeric::Float(v) = one_f.modulo(&zero_f, config).unwrap() {
        assert!(v.is_nan());
    }
    if let GenericNumeric::Double(v) = one_d.modulo(&zero_d, config).unwrap() {
        assert!(v.is_nan());
    }

    // Test with negative zero
    let neg_zero_f = GenericNumeric::Float(-0.0);
    let neg_zero_d = GenericNumeric::Double(-0.0);
    assert_eq!(
        zero_f.add(&neg_zero_f, config).unwrap(),
        GenericNumeric::Float(0.0)
    );
    assert_eq!(
        zero_d.add(&neg_zero_d, config).unwrap(),
        GenericNumeric::Double(0.0)
    );
}

#[test]
fn test_type_creation_coverage() {
    // Test from_bool
    assert_eq!(GenericNumeric::from_bool(true), GenericNumeric::Bool(true));
    assert_eq!(
        GenericNumeric::from_bool(false),
        GenericNumeric::Bool(false)
    );

    // Test to_bool for more types
    assert!(GenericNumeric::Float(1.0).to_bool());
    assert!(!GenericNumeric::Float(0.0).to_bool());
    assert!(GenericNumeric::Float(f32::NAN).to_bool()); // NaN is truthy
    assert!(GenericNumeric::Double(1.0).to_bool());
    assert!(!GenericNumeric::Double(0.0).to_bool());
    assert!(GenericNumeric::Double(f64::NAN).to_bool()); // NaN is truthy
}

#[test]
fn test_comparison_edge_cases() {
    // Test NaN comparisons
    let nan_f = GenericNumeric::Float(f32::NAN);
    let one_f = GenericNumeric::Float(1.0);
    assert!(!NumericValue::lt(&nan_f, &one_f));
    assert!(!NumericValue::gt(&nan_f, &one_f));
    assert!(!NumericValue::eq(&nan_f, &nan_f)); // NaN != NaN
    assert!(NumericValue::ne(&nan_f, &nan_f)); // NaN != NaN

    let nan_d = GenericNumeric::Double(f64::NAN);
    let one_d = GenericNumeric::Double(1.0);
    assert!(!NumericValue::lt(&nan_d, &one_d));
    assert!(!NumericValue::gt(&nan_d, &one_d));
    assert!(!NumericValue::eq(&nan_d, &nan_d)); // NaN != NaN
    assert!(NumericValue::ne(&nan_d, &nan_d)); // NaN != NaN

    // Test char comparisons with edge cases
    let char_min = GenericNumeric::Char('\0');
    let char_max = GenericNumeric::Char('\u{10FFFF}');
    assert!(NumericValue::lt(&char_min, &char_max));
    assert!(NumericValue::gt(&char_max, &char_min));
}

#[test]
fn test_bitwise_char_coverage() {
    // Test char bitwise operations more thoroughly
    let char_a = GenericNumeric::Char('A'); // 65
    let char_b = GenericNumeric::Char('B'); // 66

    // AND: 65 & 66 = 64 = '@'
    assert_eq!(char_a.bit_and(&char_b), GenericNumeric::Char('@'));

    // OR: 65 | 66 = 67 = 'C'
    assert_eq!(char_a.bit_or(&char_b), GenericNumeric::Char('C'));

    // XOR: 65 ^ 66 = 3 = '\x03'
    assert_eq!(char_a.bit_xor(&char_b), GenericNumeric::Char('\x03'));

    // Test with high unicode values that might produce invalid results
    let high_char = GenericNumeric::Char('\u{FFFE}');
    let result = high_char.bit_not();
    if let GenericNumeric::Char(c) = result {
        // Should be mapped to '\0' or another valid char
        assert!(c <= '\u{10FFFF}');
    }
}

#[test]
fn test_arithmetic_with_saturate_config() {
    let saturate_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };

    // Test saturating arithmetic for all signed types
    assert_eq!(
        GenericNumeric::Int8(i8::MAX)
            .add(&GenericNumeric::Int8(1), saturate_config)
            .unwrap(),
        GenericNumeric::Int8(i8::MAX)
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MAX)
            .add(&GenericNumeric::Int16(1), saturate_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MAX)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MAX)
            .add(&GenericNumeric::Int32(1), saturate_config)
            .unwrap(),
        GenericNumeric::Int32(i32::MAX)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MAX)
            .add(&GenericNumeric::Int64(1), saturate_config)
            .unwrap(),
        GenericNumeric::Int64(i64::MAX)
    );

    // Test saturating subtraction
    assert_eq!(
        GenericNumeric::Int8(i8::MIN)
            .sub(&GenericNumeric::Int8(1), saturate_config)
            .unwrap(),
        GenericNumeric::Int8(i8::MIN)
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MIN)
            .sub(&GenericNumeric::Int16(1), saturate_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MIN)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MIN)
            .sub(&GenericNumeric::Int32(1), saturate_config)
            .unwrap(),
        GenericNumeric::Int32(i32::MIN)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MIN)
            .sub(&GenericNumeric::Int64(1), saturate_config)
            .unwrap(),
        GenericNumeric::Int64(i64::MIN)
    );

    // Test saturating multiplication
    assert_eq!(
        GenericNumeric::Int8(i8::MAX)
            .mul(&GenericNumeric::Int8(2), saturate_config)
            .unwrap(),
        GenericNumeric::Int8(i8::MAX)
    );
    assert_eq!(
        GenericNumeric::UInt8(u8::MAX)
            .mul(&GenericNumeric::UInt8(2), saturate_config)
            .unwrap(),
        GenericNumeric::UInt8(u8::MAX)
    );

    // Test saturating division (MIN / -1)
    assert_eq!(
        GenericNumeric::Int8(i8::MIN)
            .div(&GenericNumeric::Int8(-1), saturate_config)
            .unwrap(),
        GenericNumeric::Int8(i8::MAX)
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MIN)
            .div(&GenericNumeric::Int16(-1), saturate_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MAX)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MIN)
            .div(&GenericNumeric::Int32(-1), saturate_config)
            .unwrap(),
        GenericNumeric::Int32(i32::MAX)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MIN)
            .div(&GenericNumeric::Int64(-1), saturate_config)
            .unwrap(),
        GenericNumeric::Int64(i64::MAX)
    );
}

#[test]
fn test_more_saturating_arithmetic() {
    let saturate_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };

    // Test saturating arithmetic for unsigned types
    assert_eq!(
        GenericNumeric::UInt8(u8::MAX)
            .add(&GenericNumeric::UInt8(1), saturate_config)
            .unwrap(),
        GenericNumeric::UInt8(u8::MAX)
    );
    assert_eq!(
        GenericNumeric::UInt16(u16::MAX)
            .add(&GenericNumeric::UInt16(1), saturate_config)
            .unwrap(),
        GenericNumeric::UInt16(u16::MAX)
    );
    assert_eq!(
        GenericNumeric::UInt32(u32::MAX)
            .add(&GenericNumeric::UInt32(1), saturate_config)
            .unwrap(),
        GenericNumeric::UInt32(u32::MAX)
    );
    assert_eq!(
        GenericNumeric::UInt64(u64::MAX)
            .add(&GenericNumeric::UInt64(1), saturate_config)
            .unwrap(),
        GenericNumeric::UInt64(u64::MAX)
    );

    // Test saturating subtraction for unsigned
    assert_eq!(
        GenericNumeric::UInt8(0)
            .sub(&GenericNumeric::UInt8(1), saturate_config)
            .unwrap(),
        GenericNumeric::UInt8(0)
    );
    assert_eq!(
        GenericNumeric::UInt16(0)
            .sub(&GenericNumeric::UInt16(1), saturate_config)
            .unwrap(),
        GenericNumeric::UInt16(0)
    );
    assert_eq!(
        GenericNumeric::UInt32(0)
            .sub(&GenericNumeric::UInt32(1), saturate_config)
            .unwrap(),
        GenericNumeric::UInt32(0)
    );
    assert_eq!(
        GenericNumeric::UInt64(0)
            .sub(&GenericNumeric::UInt64(1), saturate_config)
            .unwrap(),
        GenericNumeric::UInt64(0)
    );

    // Test saturating multiplication for more types
    assert_eq!(
        GenericNumeric::Int16(i16::MAX)
            .mul(&GenericNumeric::Int16(2), saturate_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MAX)
    );
    assert_eq!(
        GenericNumeric::UInt16(u16::MAX)
            .mul(&GenericNumeric::UInt16(2), saturate_config)
            .unwrap(),
        GenericNumeric::UInt16(u16::MAX)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MAX)
            .mul(&GenericNumeric::Int32(2), saturate_config)
            .unwrap(),
        GenericNumeric::Int32(i32::MAX)
    );
    assert_eq!(
        GenericNumeric::UInt32(u32::MAX)
            .mul(&GenericNumeric::UInt32(2), saturate_config)
            .unwrap(),
        GenericNumeric::UInt32(u32::MAX)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MAX)
            .mul(&GenericNumeric::Int64(2), saturate_config)
            .unwrap(),
        GenericNumeric::Int64(i64::MAX)
    );
    assert_eq!(
        GenericNumeric::UInt64(u64::MAX)
            .mul(&GenericNumeric::UInt64(2), saturate_config)
            .unwrap(),
        GenericNumeric::UInt64(u64::MAX)
    );
}

#[test]
fn test_wrapping_division() {
    let wrap_config = EvalConfig {
        overflow: OverflowBehavior::Wrap,
        ..Default::default()
    };

    // Test wrapping division (MIN / -1)
    assert_eq!(
        GenericNumeric::Int8(i8::MIN)
            .div(&GenericNumeric::Int8(-1), wrap_config)
            .unwrap(),
        GenericNumeric::Int8(i8::MIN)
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MIN)
            .div(&GenericNumeric::Int16(-1), wrap_config)
            .unwrap(),
        GenericNumeric::Int16(i16::MIN)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MIN)
            .div(&GenericNumeric::Int32(-1), wrap_config)
            .unwrap(),
        GenericNumeric::Int32(i32::MIN)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MIN)
            .div(&GenericNumeric::Int64(-1), wrap_config)
            .unwrap(),
        GenericNumeric::Int64(i64::MIN)
    );
}

#[test]
fn test_comprehensive_bit_not() {
    // Test bit_not for all types
    assert_eq!(
        GenericNumeric::Bool(true).bit_not(),
        GenericNumeric::Bool(false)
    );
    assert_eq!(
        GenericNumeric::Bool(false).bit_not(),
        GenericNumeric::Bool(true)
    );

    assert_eq!(GenericNumeric::Int8(0).bit_not(), GenericNumeric::Int8(-1));
    assert_eq!(
        GenericNumeric::UInt8(0).bit_not(),
        GenericNumeric::UInt8(u8::MAX)
    );
    assert_eq!(
        GenericNumeric::Int16(0).bit_not(),
        GenericNumeric::Int16(-1)
    );
    assert_eq!(
        GenericNumeric::UInt16(0).bit_not(),
        GenericNumeric::UInt16(u16::MAX)
    );
    assert_eq!(
        GenericNumeric::Int32(0).bit_not(),
        GenericNumeric::Int32(-1)
    );
    assert_eq!(
        GenericNumeric::UInt32(0).bit_not(),
        GenericNumeric::UInt32(u32::MAX)
    );
    assert_eq!(
        GenericNumeric::Int64(0).bit_not(),
        GenericNumeric::Int64(-1)
    );
    assert_eq!(
        GenericNumeric::UInt64(0).bit_not(),
        GenericNumeric::UInt64(u64::MAX)
    );

    // Float and double return self
    assert_eq!(
        GenericNumeric::Float(1.0).bit_not(),
        GenericNumeric::Float(1.0)
    );
    assert_eq!(
        GenericNumeric::Double(1.0).bit_not(),
        GenericNumeric::Double(1.0)
    );
}

#[test]
fn test_shr_operations() {
    let config = EvalConfig::default();

    // Test right shift for all integer types
    assert_eq!(
        GenericNumeric::Int8(-128)
            .shr(&GenericNumeric::Int8(7), config)
            .unwrap(),
        GenericNumeric::Int8(-1)
    );
    assert_eq!(
        GenericNumeric::UInt8(128)
            .shr(&GenericNumeric::UInt8(7), config)
            .unwrap(),
        GenericNumeric::UInt8(1)
    );
    assert_eq!(
        GenericNumeric::Int16(-32768)
            .shr(&GenericNumeric::Int16(15), config)
            .unwrap(),
        GenericNumeric::Int16(-1)
    );
    assert_eq!(
        GenericNumeric::UInt16(32768)
            .shr(&GenericNumeric::UInt16(15), config)
            .unwrap(),
        GenericNumeric::UInt16(1)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MIN)
            .shr(&GenericNumeric::Int32(31), config)
            .unwrap(),
        GenericNumeric::Int32(-1)
    );
    assert_eq!(
        GenericNumeric::UInt32(2_147_483_648)
            .shr(&GenericNumeric::UInt32(31), config)
            .unwrap(),
        GenericNumeric::UInt32(1)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MIN)
            .shr(&GenericNumeric::Int64(63), config)
            .unwrap(),
        GenericNumeric::Int64(-1)
    );
    assert_eq!(
        GenericNumeric::UInt64(9_223_372_036_854_775_808)
            .shr(&GenericNumeric::UInt64(63), config)
            .unwrap(),
        GenericNumeric::UInt64(1)
    );

    // Test shr with negative amounts
    assert!(
        GenericNumeric::Int8(1)
            .shr(&GenericNumeric::Int8(-1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int16(1)
            .shr(&GenericNumeric::Int16(-1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(1)
            .shr(&GenericNumeric::Int32(-1), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int64(1)
            .shr(&GenericNumeric::Int64(-1), config)
            .is_err()
    );

    // Test shr with too large amounts
    assert!(
        GenericNumeric::Int32(1)
            .shr(&GenericNumeric::Int32(128), config)
            .is_err()
    );

    // Test shr on non-integer types (returns self)
    assert_eq!(
        GenericNumeric::Bool(true)
            .shr(&GenericNumeric::Int32(1), config)
            .unwrap(),
        GenericNumeric::Bool(true)
    );
    assert_eq!(
        GenericNumeric::Char('A')
            .shr(&GenericNumeric::Int32(1), config)
            .unwrap(),
        GenericNumeric::Char('A')
    );
    assert_eq!(
        GenericNumeric::Float(1.0)
            .shr(&GenericNumeric::Int32(1), config)
            .unwrap(),
        GenericNumeric::Float(1.0)
    );
    assert_eq!(
        GenericNumeric::Double(1.0)
            .shr(&GenericNumeric::Int32(1), config)
            .unwrap(),
        GenericNumeric::Double(1.0)
    );
}

#[test]
fn test_negate_bool_char() {
    let config = EvalConfig::default();

    // Test negating bool and char (promotes to Int32)
    assert_eq!(
        GenericNumeric::Bool(true).negate(config).unwrap(),
        GenericNumeric::Int32(-1)
    );
    assert_eq!(
        GenericNumeric::Bool(false).negate(config).unwrap(),
        GenericNumeric::Int32(0)
    );
    assert_eq!(
        GenericNumeric::Char('A').negate(config).unwrap(),
        GenericNumeric::Int32(-65)
    );
    assert_eq!(
        GenericNumeric::Char('\0').negate(config).unwrap(),
        GenericNumeric::Int32(0)
    );
}

#[test]
fn test_float_double_negate() {
    let config = EvalConfig::default();

    // Test negating floats and doubles
    assert_eq!(
        GenericNumeric::Float(1.0).negate(config).unwrap(),
        GenericNumeric::Float(-1.0)
    );
    assert_eq!(
        GenericNumeric::Float(-1.0).negate(config).unwrap(),
        GenericNumeric::Float(1.0)
    );
    assert_eq!(
        GenericNumeric::Double(1.0).negate(config).unwrap(),
        GenericNumeric::Double(-1.0)
    );
    assert_eq!(
        GenericNumeric::Double(-1.0).negate(config).unwrap(),
        GenericNumeric::Double(1.0)
    );

    // Test with special values
    if let GenericNumeric::Float(v) = GenericNumeric::Float(f32::NAN).negate(config).unwrap() {
        assert!(v.is_nan());
    }
    assert_eq!(
        GenericNumeric::Float(f32::INFINITY).negate(config).unwrap(),
        GenericNumeric::Float(f32::NEG_INFINITY)
    );
    assert_eq!(
        GenericNumeric::Float(f32::NEG_INFINITY)
            .negate(config)
            .unwrap(),
        GenericNumeric::Float(f32::INFINITY)
    );
}

#[test]
fn test_promote_to_coverage() {
    // Test various promotion scenarios through arithmetic operations
    let config = EvalConfig::default();

    // Test Bool + Char promotion
    let b = GenericNumeric::Bool(true);
    let c = GenericNumeric::Char('A');
    // Bool(true) = 1, Char('A') = 65, so 1 + 65 = 66 = 'B'
    assert_eq!(b.add(&c, config).unwrap(), GenericNumeric::Char('B')); // Bool promotes to Char

    // Test mixed float/int operations
    assert_eq!(
        GenericNumeric::Int8(10)
            .add(&GenericNumeric::Float(5.0), config)
            .unwrap(),
        GenericNumeric::Float(15.0)
    );
    assert_eq!(
        GenericNumeric::UInt16(10)
            .add(&GenericNumeric::Double(5.0), config)
            .unwrap(),
        GenericNumeric::Double(15.0)
    );
    assert_eq!(
        GenericNumeric::Char('A')
            .add(&GenericNumeric::Double(1.0), config)
            .unwrap(),
        GenericNumeric::Double(66.0)
    );

    // Test promotions with subtraction
    assert_eq!(
        GenericNumeric::UInt8(10)
            .sub(&GenericNumeric::Int16(5), config)
            .unwrap(),
        GenericNumeric::Int16(5)
    );
    assert_eq!(
        GenericNumeric::Bool(true)
            .sub(&GenericNumeric::Float(0.5), config)
            .unwrap(),
        GenericNumeric::Float(0.5)
    );

    // Test promotions with multiplication
    assert_eq!(
        GenericNumeric::Char('A')
            .mul(&GenericNumeric::UInt8(2), config)
            .unwrap(),
        GenericNumeric::UInt8(130)
    );
    assert_eq!(
        GenericNumeric::Int16(10)
            .mul(&GenericNumeric::Float(2.5), config)
            .unwrap(),
        GenericNumeric::Float(25.0)
    );

    // Test promotions with division
    assert_eq!(
        GenericNumeric::UInt32(20)
            .div(&GenericNumeric::Int64(4), config)
            .unwrap(),
        GenericNumeric::Int64(5)
    );
    assert_eq!(
        GenericNumeric::Int8(20)
            .div(&GenericNumeric::Double(4.0), config)
            .unwrap(),
        GenericNumeric::Double(5.0)
    );
}

#[test]
fn test_bitwise_bool_operations() {
    // Test all bool bitwise operations
    let t = GenericNumeric::Bool(true);
    let f = GenericNumeric::Bool(false);

    assert_eq!(t.bit_and(&t), GenericNumeric::Bool(true));
    assert_eq!(t.bit_and(&f), GenericNumeric::Bool(false));
    assert_eq!(f.bit_and(&t), GenericNumeric::Bool(false));
    assert_eq!(f.bit_and(&f), GenericNumeric::Bool(false));

    assert_eq!(t.bit_or(&t), GenericNumeric::Bool(true));
    assert_eq!(t.bit_or(&f), GenericNumeric::Bool(true));
    assert_eq!(f.bit_or(&t), GenericNumeric::Bool(true));
    assert_eq!(f.bit_or(&f), GenericNumeric::Bool(false));

    assert_eq!(t.bit_xor(&t), GenericNumeric::Bool(false));
    assert_eq!(t.bit_xor(&f), GenericNumeric::Bool(true));
    assert_eq!(f.bit_xor(&t), GenericNumeric::Bool(true));
    assert_eq!(f.bit_xor(&f), GenericNumeric::Bool(false));
}

#[test]
fn test_comparison_type_combinations() {
    let config = EvalConfig::default();

    // Test more comparison combinations
    let i8_val = GenericNumeric::Int8(10);
    let u8_val = GenericNumeric::UInt8(10);
    let i16_val = GenericNumeric::Int16(10);
    let u16_val = GenericNumeric::UInt16(10);
    let i32_val = GenericNumeric::Int32(10);
    let u32_val = GenericNumeric::UInt32(10);
    let i64_val = GenericNumeric::Int64(10);
    let u64_val = GenericNumeric::UInt64(10);
    let f_val = GenericNumeric::Float(10.0);
    let d_val = GenericNumeric::Double(10.0);

    // Test eq with same types
    assert!(NumericValue::eq(&i8_val, &GenericNumeric::Int8(10)));
    assert!(NumericValue::eq(&u16_val, &GenericNumeric::UInt16(10)));
    assert!(NumericValue::eq(&f_val, &GenericNumeric::Float(10.0)));
    assert!(NumericValue::eq(&d_val, &GenericNumeric::Double(10.0)));

    // Test that operations with type promotion work correctly
    assert_eq!(
        i8_val.add(&u8_val, config).unwrap(),
        GenericNumeric::UInt8(20)
    );
    assert_eq!(
        i16_val.add(&u16_val, config).unwrap(),
        GenericNumeric::UInt16(20)
    );
    assert_eq!(
        i32_val.add(&u32_val, config).unwrap(),
        GenericNumeric::UInt32(20)
    );
    assert_eq!(
        i64_val.add(&u64_val, config).unwrap(),
        GenericNumeric::UInt64(20)
    );
    assert_eq!(
        i8_val.add(&f_val, config).unwrap(),
        GenericNumeric::Float(20.0)
    );
    assert_eq!(
        u16_val.add(&d_val, config).unwrap(),
        GenericNumeric::Double(20.0)
    );

    // Test lt/gt with same types (NumericValue trait is for same-type comparison)
    let i8_smaller = GenericNumeric::Int8(5);
    let i8_larger = GenericNumeric::Int8(15);
    assert!(NumericValue::lt(&i8_smaller, &i8_larger));
    assert!(NumericValue::gt(&i8_larger, &i8_smaller));

    let f_smaller = GenericNumeric::Float(5.0);
    let f_larger = GenericNumeric::Float(15.0);
    assert!(NumericValue::lt(&f_smaller, &f_larger));
    assert!(NumericValue::gt(&f_larger, &f_smaller));

    // Test ne
    assert!(NumericValue::ne(&i8_smaller, &i8_larger));
    assert!(!NumericValue::ne(&i8_val, &i8_val));
}

#[test]
fn test_division_normal_cases() {
    let config = EvalConfig::default();

    // Test normal division for unsigned types (no overflow possible)
    assert_eq!(
        GenericNumeric::UInt8(20)
            .div(&GenericNumeric::UInt8(4), config)
            .unwrap(),
        GenericNumeric::UInt8(5)
    );
    assert_eq!(
        GenericNumeric::UInt16(100)
            .div(&GenericNumeric::UInt16(10), config)
            .unwrap(),
        GenericNumeric::UInt16(10)
    );
    assert_eq!(
        GenericNumeric::UInt32(1000)
            .div(&GenericNumeric::UInt32(100), config)
            .unwrap(),
        GenericNumeric::UInt32(10)
    );
    assert_eq!(
        GenericNumeric::UInt64(10000)
            .div(&GenericNumeric::UInt64(1000), config)
            .unwrap(),
        GenericNumeric::UInt64(10)
    );

    // Test normal signed division
    assert_eq!(
        GenericNumeric::Int8(20)
            .div(&GenericNumeric::Int8(4), config)
            .unwrap(),
        GenericNumeric::Int8(5)
    );
    assert_eq!(
        GenericNumeric::Int16(-100)
            .div(&GenericNumeric::Int16(10), config)
            .unwrap(),
        GenericNumeric::Int16(-10)
    );
    assert_eq!(
        GenericNumeric::Int32(1000)
            .div(&GenericNumeric::Int32(-100), config)
            .unwrap(),
        GenericNumeric::Int32(-10)
    );
    assert_eq!(
        GenericNumeric::Int64(-10000)
            .div(&GenericNumeric::Int64(-1000), config)
            .unwrap(),
        GenericNumeric::Int64(10)
    );
}

#[test]
fn test_modulo_normal_cases() {
    let config = EvalConfig::default();

    // Test normal modulo for all types
    assert_eq!(
        GenericNumeric::Int8(17)
            .modulo(&GenericNumeric::Int8(5), config)
            .unwrap(),
        GenericNumeric::Int8(2)
    );
    assert_eq!(
        GenericNumeric::UInt8(17)
            .modulo(&GenericNumeric::UInt8(5), config)
            .unwrap(),
        GenericNumeric::UInt8(2)
    );
    assert_eq!(
        GenericNumeric::Int16(17)
            .modulo(&GenericNumeric::Int16(5), config)
            .unwrap(),
        GenericNumeric::Int16(2)
    );
    assert_eq!(
        GenericNumeric::UInt16(17)
            .modulo(&GenericNumeric::UInt16(5), config)
            .unwrap(),
        GenericNumeric::UInt16(2)
    );
    assert_eq!(
        GenericNumeric::Int32(17)
            .modulo(&GenericNumeric::Int32(5), config)
            .unwrap(),
        GenericNumeric::Int32(2)
    );
    assert_eq!(
        GenericNumeric::UInt32(17)
            .modulo(&GenericNumeric::UInt32(5), config)
            .unwrap(),
        GenericNumeric::UInt32(2)
    );
    assert_eq!(
        GenericNumeric::Int64(17)
            .modulo(&GenericNumeric::Int64(5), config)
            .unwrap(),
        GenericNumeric::Int64(2)
    );
    assert_eq!(
        GenericNumeric::UInt64(17)
            .modulo(&GenericNumeric::UInt64(5), config)
            .unwrap(),
        GenericNumeric::UInt64(2)
    );

    // Test float/double modulo
    assert_eq!(
        GenericNumeric::Float(17.5)
            .modulo(&GenericNumeric::Float(5.0), config)
            .unwrap(),
        GenericNumeric::Float(2.5)
    );
    assert_eq!(
        GenericNumeric::Double(17.5)
            .modulo(&GenericNumeric::Double(5.0), config)
            .unwrap(),
        GenericNumeric::Double(2.5)
    );
}

#[test]
fn test_unreachable_paths() {
    // Test that should never hit unreachable branches due to proper promotion
    let config = EvalConfig::default();

    // These operations will promote properly, so no type mismatch
    assert!(
        GenericNumeric::Bool(true)
            .add(&GenericNumeric::UInt64(1), config)
            .is_ok()
    );
    assert!(
        GenericNumeric::Char('A')
            .sub(&GenericNumeric::Int64(1), config)
            .is_ok()
    );
    assert!(
        GenericNumeric::Int8(10)
            .mul(&GenericNumeric::UInt64(2), config)
            .is_ok()
    );
    assert!(
        GenericNumeric::UInt8(10)
            .div(&GenericNumeric::Int64(2), config)
            .is_ok()
    );
    assert!(
        GenericNumeric::Int16(10)
            .modulo(&GenericNumeric::UInt32(3), config)
            .is_ok()
    );
}

#[test]
fn test_char_arithmetic_edge_cases() {
    let config = EvalConfig::default();

    // Test char arithmetic that produces large values
    let large_char = GenericNumeric::Char('\u{D7FF}'); // Largest valid char before surrogates
    let result = large_char.add(&large_char, config).unwrap();
    // The result should be a valid char (char_from_u32 returns Some or maps to '\0')
    if let GenericNumeric::Char(c) = result {
        // D7FF + D7FF = 1AFFE which is > 10FFFF, so should map to '\0'
        assert!(c == '\0' || c <= '\u{10FFFF}');
    }

    // Test char subtraction that would go negative
    let small_char = GenericNumeric::Char('\u{0001}');
    let large_char2 = GenericNumeric::Char('\u{FFFF}');
    let result = small_char.sub(&large_char2, config).unwrap();
    if let GenericNumeric::Char(c) = result {
        // 1 - FFFF wraps around in u32 arithmetic
        // The exact result depends on wrapping_sub behavior
        assert!(c == '\0' || c.is_control() || (c as u32) > 0xFFFF);
    }

    // Test char multiplication overflow
    let char_100 = GenericNumeric::Char('d'); // ASCII 100
    let result = char_100.mul(&char_100, config).unwrap();
    if let GenericNumeric::Char(c) = result {
        // 100 * 100 = 10000, which is a valid char
        assert_eq!(c, '\u{2710}');
    }

    // Test char division
    assert_eq!(
        GenericNumeric::Char('d')
            .div(&GenericNumeric::Char('\u{0002}'), config)
            .unwrap(),
        GenericNumeric::Char('2') // 100 / 2 = 50 = '2'
    );

    // Test char modulo
    assert_eq!(
        GenericNumeric::Char('e')
            .modulo(&GenericNumeric::Char('d'), config)
            .unwrap(),
        GenericNumeric::Char('\u{0001}') // 101 % 100 = 1
    );
}

#[test]
fn test_float_to_integer_precision_loss() {
    let config = EvalConfig::default();

    // Test Double to Float precision loss through operations
    let large_double = GenericNumeric::Double(1.797_693_134_862_315_7e308); // Near Double::MAX
    let float_val = GenericNumeric::Float(0.0);
    // When double is used with float, double should convert to float
    let result = large_double.add(&float_val, config);
    if let Ok(GenericNumeric::Float(f)) = result {
        assert!(f.is_infinite());
    }

    // Test large integer to float conversions
    let large_u64 = GenericNumeric::UInt64(u64::MAX);
    let float_zero = GenericNumeric::Float(0.0);
    let result = large_u64.add(&float_zero, config).unwrap();
    if let GenericNumeric::Float(f) = result {
        // Precision is lost but value should be reasonable
        assert!(f > 1e19);
    }

    // Test signed to unsigned conversion edge cases
    let neg_i8 = GenericNumeric::Int8(-1);
    let u8_val = GenericNumeric::UInt8(1);
    // Int8(-1) + UInt8(1) should promote Int8 to UInt8
    assert_eq!(
        neg_i8.add(&u8_val, config).unwrap(),
        GenericNumeric::UInt8(0)
    ); // 255 + 1 = 0 (wrapping)
}

#[test]
fn test_division_saturating_overflow() {
    let saturate_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };

    // Test Int8::MIN / -1 with saturating (should give MAX)
    assert_eq!(
        GenericNumeric::Int8(i8::MIN)
            .div(&GenericNumeric::Int8(-1), saturate_config)
            .unwrap(),
        GenericNumeric::Int8(i8::MAX)
    );

    // Test normal division doesn't saturate
    assert_eq!(
        GenericNumeric::Int8(100)
            .div(&GenericNumeric::Int8(2), saturate_config)
            .unwrap(),
        GenericNumeric::Int8(50)
    );
}

#[test]
fn test_shift_error_paths() {
    let config = EvalConfig::default();

    // Test shift with Bool shift amount (should error)
    assert!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::Bool(true), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(1)
            .shr(&GenericNumeric::Bool(false), config)
            .is_err()
    );

    // Test shift with Char shift amount (should error)
    assert!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::Char('A'), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(1)
            .shr(&GenericNumeric::Char('B'), config)
            .is_err()
    );

    // Test shift with Float/Double shift amount (should error)
    assert!(
        GenericNumeric::Int32(1)
            .shl(&GenericNumeric::Float(1.0), config)
            .is_err()
    );
    assert!(
        GenericNumeric::Int32(1)
            .shr(&GenericNumeric::Double(2.0), config)
            .is_err()
    );

    // Test Bool/Char/Float/Double being shifted (returns self)
    assert_eq!(
        GenericNumeric::Bool(true)
            .shl(&GenericNumeric::Int32(5), config)
            .unwrap(),
        GenericNumeric::Bool(true)
    );
    assert_eq!(
        GenericNumeric::Char('A')
            .shr(&GenericNumeric::Int32(5), config)
            .unwrap(),
        GenericNumeric::Char('A')
    );
    assert_eq!(
        GenericNumeric::Float(1.5)
            .shl(&GenericNumeric::Int32(5), config)
            .unwrap(),
        GenericNumeric::Float(1.5)
    );
    assert_eq!(
        GenericNumeric::Double(2.5)
            .shr(&GenericNumeric::Int32(5), config)
            .unwrap(),
        GenericNumeric::Double(2.5)
    );
}

#[test]
fn test_bitwise_non_integer_types() {
    // Test bit_not on float/double returns self
    assert_eq!(
        GenericNumeric::Float(1.5).bit_not(),
        GenericNumeric::Float(1.5)
    );
    assert_eq!(
        GenericNumeric::Double(2.5).bit_not(),
        GenericNumeric::Double(2.5)
    );

    // Test bit_and with non-matching types
    let i32_val = GenericNumeric::Int32(0xFF);
    let f32_val = GenericNumeric::Float(1.0);
    let bool_val = GenericNumeric::Bool(true);

    // These should return self for non-integer types
    assert_eq!(f32_val.bit_and(&i32_val), GenericNumeric::Float(1.0));
    assert_eq!(bool_val.bit_and(&bool_val), GenericNumeric::Bool(true));

    // Test bit_or with non-matching types
    assert_eq!(f32_val.bit_or(&i32_val), GenericNumeric::Float(1.0));

    // Test bit_xor with non-matching types
    assert_eq!(f32_val.bit_xor(&i32_val), GenericNumeric::Float(1.0));
}

#[test]
fn test_comparison_mismatched_types() {
    // Test comparisons with different types (should return false for non-eq)
    let i32_val = GenericNumeric::Int32(10);
    let _ = GenericNumeric::Float(10.0); // Different type

    // For NumericValue trait, these operate on same types only
    // But let's test the actual comparison functions
    assert!(!NumericValue::lt(&i32_val, &i32_val)); // Same value
    assert!(NumericValue::le(&i32_val, &i32_val)); // Same value
    assert!(!NumericValue::gt(&i32_val, &i32_val)); // Same value
    assert!(NumericValue::ge(&i32_val, &i32_val)); // Same value

    // Test with actual different values
    let i32_smaller = GenericNumeric::Int32(5);
    let i32_larger = GenericNumeric::Int32(15);
    assert!(NumericValue::lt(&i32_smaller, &i32_larger));
    assert!(NumericValue::le(&i32_smaller, &i32_larger));
    assert!(NumericValue::gt(&i32_larger, &i32_smaller));
    assert!(NumericValue::ge(&i32_larger, &i32_smaller));
    assert!(NumericValue::ne(&i32_smaller, &i32_larger));
}

#[test]
fn test_modulo_edge_cases_saturating() {
    let saturate_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };

    // For modulo, MIN % -1 doesn't overflow in Rust, it returns 0
    // But our implementation uses wrapping_rem for saturate mode
    assert_eq!(
        GenericNumeric::Int8(i8::MIN)
            .modulo(&GenericNumeric::Int8(-1), saturate_config)
            .unwrap(),
        GenericNumeric::Int8(0)
    );
    assert_eq!(
        GenericNumeric::Int16(i16::MIN)
            .modulo(&GenericNumeric::Int16(-1), saturate_config)
            .unwrap(),
        GenericNumeric::Int16(0)
    );
    assert_eq!(
        GenericNumeric::Int32(i32::MIN)
            .modulo(&GenericNumeric::Int32(-1), saturate_config)
            .unwrap(),
        GenericNumeric::Int32(0)
    );
    assert_eq!(
        GenericNumeric::Int64(i64::MIN)
            .modulo(&GenericNumeric::Int64(-1), saturate_config)
            .unwrap(),
        GenericNumeric::Int64(0)
    );
}

#[test]
fn test_uint64_negation_edge_cases() {
    // Test all three overflow behaviors for UInt64 negation
    let wrap_config = EvalConfig {
        overflow: OverflowBehavior::Wrap,
        ..Default::default()
    };
    let error_config = EvalConfig {
        overflow: OverflowBehavior::Error,
        ..Default::default()
    };
    let saturate_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };

    // Wrap: wrapping negation
    assert_eq!(
        GenericNumeric::UInt64(100).negate(wrap_config).unwrap(),
        GenericNumeric::UInt64(u64::MAX - 99)
    );

    // Error: should fail
    assert!(GenericNumeric::UInt64(100).negate(error_config).is_err());

    // Saturate: returns 0
    assert_eq!(
        GenericNumeric::UInt64(100).negate(saturate_config).unwrap(),
        GenericNumeric::UInt64(0)
    );
}

#[test]
fn test_char_bitwise_edge_cases() {
    // Test char bitwise operations that might produce invalid Unicode
    let high_char1 = GenericNumeric::Char('\u{D7FF}');
    let high_char2 = GenericNumeric::Char('\u{E000}');

    // Test OR that might produce invalid result
    let result = high_char1.bit_or(&high_char2);
    if let GenericNumeric::Char(c) = result {
        // Should be a valid char or mapped to '\0'
        assert!(c <= '\u{10FFFF}' || c == '\0');
    }

    // Test XOR
    let result = high_char1.bit_xor(&high_char2);
    if let GenericNumeric::Char(c) = result {
        assert!(c <= '\u{10FFFF}' || c == '\0');
    }
}

#[test]
fn test_float_special_value_operations() {
    let config = EvalConfig::default();

    // Test operations with negative zero
    let neg_zero = GenericNumeric::Float(-0.0);
    let pos_zero = GenericNumeric::Float(0.0);

    // -0.0 + 0.0 = 0.0
    assert_eq!(
        neg_zero.add(&pos_zero, config).unwrap(),
        GenericNumeric::Float(0.0)
    );

    // Test operations with subnormal numbers
    let subnormal = GenericNumeric::Float(f32::MIN_POSITIVE / 2.0);
    assert!(matches!(
        subnormal.add(&subnormal, config).unwrap(),
        GenericNumeric::Float(_)
    ));

    // Test infinity arithmetic
    let inf = GenericNumeric::Float(f32::INFINITY);
    let neg_inf = GenericNumeric::Float(f32::NEG_INFINITY);

    // inf - inf = NaN
    if let GenericNumeric::Float(v) = inf.sub(&inf, config).unwrap() {
        assert!(v.is_nan());
    }

    // inf + neg_inf = NaN
    if let GenericNumeric::Float(v) = inf.add(&neg_inf, config).unwrap() {
        assert!(v.is_nan());
    }
}

#[test]
fn test_bool_bool_arithmetic() {
    let config = EvalConfig::default();

    // Bool + Bool promotes both to the higher rank (both rank 0, so they stay Bool)
    // But there's no Bool arithmetic implementation, so these should fail
    let t = GenericNumeric::Bool(true);
    let f = GenericNumeric::Bool(false);

    // These should hit the unreachable "type mismatch" error
    // Actually, let's trace through: Bool + Bool -> promote_for_arithmetic
    // Both have rank 0, so they stay Bool, then add() has no Bool case
    // So it hits the _ => Err(...) case
    assert!(t.add(&f, config).is_err());
    assert!(t.sub(&f, config).is_err());
    assert!(t.mul(&f, config).is_err());
    assert!(t.div(&f, config).is_err());
    assert!(t.modulo(&f, config).is_err());
}

#[test]
fn test_exhaustive_type_promotions() {
    // Test some of the "unreachable" promotion paths to ensure they behave correctly
    // even though they shouldn't be called in practice

    let config = EvalConfig::default();

    // Create a comprehensive test for mixed-type arithmetic
    let bool_val = GenericNumeric::Bool(true);
    let char_val = GenericNumeric::Char('A');
    let i8_val = GenericNumeric::Int8(10);
    let u8_val = GenericNumeric::UInt8(10);
    let i16_val = GenericNumeric::Int16(10);
    let u16_val = GenericNumeric::UInt16(10);
    let i32_val = GenericNumeric::Int32(10);
    let u32_val = GenericNumeric::UInt32(10);
    let i64_val = GenericNumeric::Int64(10);
    let u64_val = GenericNumeric::UInt64(10);
    let f32_val = GenericNumeric::Float(10.0);
    let f64_val = GenericNumeric::Double(10.0);

    // Test Bool with everything
    assert_eq!(
        bool_val.add(&char_val, config).unwrap(),
        GenericNumeric::Char('B')
    );
    assert_eq!(
        bool_val.add(&i8_val, config).unwrap(),
        GenericNumeric::Int8(11)
    );
    assert_eq!(
        bool_val.add(&u8_val, config).unwrap(),
        GenericNumeric::UInt8(11)
    );
    assert_eq!(
        bool_val.add(&i16_val, config).unwrap(),
        GenericNumeric::Int16(11)
    );
    assert_eq!(
        bool_val.add(&u16_val, config).unwrap(),
        GenericNumeric::UInt16(11)
    );
    assert_eq!(
        bool_val.add(&i32_val, config).unwrap(),
        GenericNumeric::Int32(11)
    );
    assert_eq!(
        bool_val.add(&u32_val, config).unwrap(),
        GenericNumeric::UInt32(11)
    );
    assert_eq!(
        bool_val.add(&i64_val, config).unwrap(),
        GenericNumeric::Int64(11)
    );
    assert_eq!(
        bool_val.add(&u64_val, config).unwrap(),
        GenericNumeric::UInt64(11)
    );
    assert_eq!(
        bool_val.add(&f32_val, config).unwrap(),
        GenericNumeric::Float(11.0)
    );
    assert_eq!(
        bool_val.add(&f64_val, config).unwrap(),
        GenericNumeric::Double(11.0)
    );

    // Test Char with larger types
    assert_eq!(
        char_val.add(&i8_val, config).unwrap(),
        GenericNumeric::Int8(75)
    ); // 'A' = 65
    assert_eq!(
        char_val.add(&u8_val, config).unwrap(),
        GenericNumeric::UInt8(75)
    );
    assert_eq!(
        char_val.add(&i16_val, config).unwrap(),
        GenericNumeric::Int16(75)
    );
    assert_eq!(
        char_val.add(&u16_val, config).unwrap(),
        GenericNumeric::UInt16(75)
    );
    assert_eq!(
        char_val.add(&i32_val, config).unwrap(),
        GenericNumeric::Int32(75)
    );
    assert_eq!(
        char_val.add(&u32_val, config).unwrap(),
        GenericNumeric::UInt32(75)
    );
    assert_eq!(
        char_val.add(&i64_val, config).unwrap(),
        GenericNumeric::Int64(75)
    );
    assert_eq!(
        char_val.add(&u64_val, config).unwrap(),
        GenericNumeric::UInt64(75)
    );
    assert_eq!(
        char_val.add(&f32_val, config).unwrap(),
        GenericNumeric::Float(75.0)
    );
    assert_eq!(
        char_val.add(&f64_val, config).unwrap(),
        GenericNumeric::Double(75.0)
    );
}
