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

use ic_expr::{EvalConfig, GenericNumeric, NumericValue, OverflowBehavior};

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
    let f = GenericNumeric::Float(3.14);
    let d = GenericNumeric::Double(2.718281828);
    
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
    assert_eq!(f, GenericNumeric::Float(3.14));
    assert_eq!(d, GenericNumeric::Double(2.718281828));
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
    assert_eq!(GenericNumeric::Bool(true).to_bool(), true);
    assert_eq!(GenericNumeric::Bool(false).to_bool(), false);
    assert_eq!(GenericNumeric::Char('A').to_bool(), true);
    assert_eq!(GenericNumeric::Char('\0').to_bool(), false);
    assert_eq!(GenericNumeric::Int8(1).to_bool(), true);
    assert_eq!(GenericNumeric::Int8(0).to_bool(), false);
    assert_eq!(GenericNumeric::Int8(-1).to_bool(), true);
    assert_eq!(GenericNumeric::UInt8(1).to_bool(), true);
    assert_eq!(GenericNumeric::UInt8(0).to_bool(), false);
    assert_eq!(GenericNumeric::Int16(1).to_bool(), true);
    assert_eq!(GenericNumeric::Int16(0).to_bool(), false);
    assert_eq!(GenericNumeric::UInt16(1).to_bool(), true);
    assert_eq!(GenericNumeric::UInt16(0).to_bool(), false);
    assert_eq!(GenericNumeric::Int32(1).to_bool(), true);
    assert_eq!(GenericNumeric::Int32(0).to_bool(), false);
    assert_eq!(GenericNumeric::UInt32(1).to_bool(), true);
    assert_eq!(GenericNumeric::UInt32(0).to_bool(), false);
    assert_eq!(GenericNumeric::Int64(1).to_bool(), true);
    assert_eq!(GenericNumeric::Int64(0).to_bool(), false);
    assert_eq!(GenericNumeric::UInt64(1).to_bool(), true);
    assert_eq!(GenericNumeric::UInt64(0).to_bool(), false);
    assert_eq!(GenericNumeric::Float(1.0).to_bool(), true);
    assert_eq!(GenericNumeric::Float(0.0).to_bool(), false);
    assert_eq!(GenericNumeric::Double(1.0).to_bool(), true);
    assert_eq!(GenericNumeric::Double(0.0).to_bool(), false);
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
        GenericNumeric::Float(3.14).negate(config).unwrap(),
        GenericNumeric::Float(-3.14)
    );
    assert_eq!(
        GenericNumeric::Double(2.718).negate(config).unwrap(),
        GenericNumeric::Double(-2.718)
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
    assert_eq!(
        GenericNumeric::Int8(0).bit_not(),
        GenericNumeric::Int8(-1)
    );
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
        GenericNumeric::Float(3.14).bit_not(),
        GenericNumeric::Float(3.14)
    );
    assert_eq!(
        GenericNumeric::Double(2.718).bit_not(),
        GenericNumeric::Double(2.718)
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
        GenericNumeric::Int8(10).add(&GenericNumeric::Int8(20), config).unwrap(),
        GenericNumeric::Int8(30)
    );
    assert_eq!(
        GenericNumeric::UInt8(10).add(&GenericNumeric::UInt8(20), config).unwrap(),
        GenericNumeric::UInt8(30)
    );
    assert_eq!(
        GenericNumeric::Int16(100).add(&GenericNumeric::Int16(200), config).unwrap(),
        GenericNumeric::Int16(300)
    );
    assert_eq!(
        GenericNumeric::UInt16(100).add(&GenericNumeric::UInt16(200), config).unwrap(),
        GenericNumeric::UInt16(300)
    );
    assert_eq!(
        GenericNumeric::Int32(1000).add(&GenericNumeric::Int32(2000), config).unwrap(),
        GenericNumeric::Int32(3000)
    );
    assert_eq!(
        GenericNumeric::UInt32(1000).add(&GenericNumeric::UInt32(2000), config).unwrap(),
        GenericNumeric::UInt32(3000)
    );
    assert_eq!(
        GenericNumeric::Int64(10000).add(&GenericNumeric::Int64(20000), config).unwrap(),
        GenericNumeric::Int64(30000)
    );
    assert_eq!(
        GenericNumeric::UInt64(10000).add(&GenericNumeric::UInt64(20000), config).unwrap(),
        GenericNumeric::UInt64(30000)
    );
    assert_eq!(
        GenericNumeric::Float(1.5).add(&GenericNumeric::Float(2.5), config).unwrap(),
        GenericNumeric::Float(4.0)
    );
    assert_eq!(
        GenericNumeric::Double(1.5).add(&GenericNumeric::Double(2.5), config).unwrap(),
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
    assert!(GenericNumeric::Int8(i8::MAX).add(&GenericNumeric::Int8(1), config).is_err());
    assert!(GenericNumeric::UInt8(u8::MAX).add(&GenericNumeric::UInt8(1), config).is_err());
    assert!(GenericNumeric::Int16(i16::MAX).add(&GenericNumeric::Int16(1), config).is_err());
    assert!(GenericNumeric::UInt16(u16::MAX).add(&GenericNumeric::UInt16(1), config).is_err());
    assert!(GenericNumeric::Int32(i32::MAX).add(&GenericNumeric::Int32(1), config).is_err());
    assert!(GenericNumeric::UInt32(u32::MAX).add(&GenericNumeric::UInt32(1), config).is_err());
    assert!(GenericNumeric::Int64(i64::MAX).add(&GenericNumeric::Int64(1), config).is_err());
    assert!(GenericNumeric::UInt64(u64::MAX).add(&GenericNumeric::UInt64(1), config).is_err());
    
    // Test wrap behavior
    let wrap_config = EvalConfig {
        overflow: OverflowBehavior::Wrap,
        ..Default::default()
    };
    
    assert_eq!(
        GenericNumeric::UInt8(u8::MAX).add(&GenericNumeric::UInt8(1), wrap_config).unwrap(),
        GenericNumeric::UInt8(0)
    );
    
    // Test saturate behavior
    let sat_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };
    
    assert_eq!(
        GenericNumeric::UInt8(u8::MAX).add(&GenericNumeric::UInt8(1), sat_config).unwrap(),
        GenericNumeric::UInt8(u8::MAX)
    );
}

#[test]
fn test_sub() {
    let config = EvalConfig::default();
    
    // Test same-type subtraction
    assert_eq!(
        GenericNumeric::Int8(30).sub(&GenericNumeric::Int8(20), config).unwrap(),
        GenericNumeric::Int8(10)
    );
    assert_eq!(
        GenericNumeric::UInt8(30).sub(&GenericNumeric::UInt8(20), config).unwrap(),
        GenericNumeric::UInt8(10)
    );
    assert_eq!(
        GenericNumeric::Float(5.5).sub(&GenericNumeric::Float(2.5), config).unwrap(),
        GenericNumeric::Float(3.0)
    );
    assert_eq!(
        GenericNumeric::Double(5.5).sub(&GenericNumeric::Double(2.5), config).unwrap(),
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
    assert!(GenericNumeric::Int8(i8::MIN).sub(&GenericNumeric::Int8(1), config).is_err());
    assert!(GenericNumeric::UInt8(0).sub(&GenericNumeric::UInt8(1), config).is_err());
    
    // Test wrap behavior
    let wrap_config = EvalConfig {
        overflow: OverflowBehavior::Wrap,
        ..Default::default()
    };
    
    assert_eq!(
        GenericNumeric::UInt8(0).sub(&GenericNumeric::UInt8(1), wrap_config).unwrap(),
        GenericNumeric::UInt8(u8::MAX)
    );
    
    // Test saturate behavior
    let sat_config = EvalConfig {
        overflow: OverflowBehavior::Saturate,
        ..Default::default()
    };
    
    assert_eq!(
        GenericNumeric::UInt8(0).sub(&GenericNumeric::UInt8(1), sat_config).unwrap(),
        GenericNumeric::UInt8(0)
    );
}

#[test]
fn test_mul() {
    let config = EvalConfig::default();
    
    // Test same-type multiplication
    assert_eq!(
        GenericNumeric::Int8(5).mul(&GenericNumeric::Int8(6), config).unwrap(),
        GenericNumeric::Int8(30)
    );
    assert_eq!(
        GenericNumeric::UInt8(5).mul(&GenericNumeric::UInt8(6), config).unwrap(),
        GenericNumeric::UInt8(30)
    );
    assert_eq!(
        GenericNumeric::Int16(10).mul(&GenericNumeric::Int16(20), config).unwrap(),
        GenericNumeric::Int16(200)
    );
    assert_eq!(
        GenericNumeric::UInt16(10).mul(&GenericNumeric::UInt16(20), config).unwrap(),
        GenericNumeric::UInt16(200)
    );
    assert_eq!(
        GenericNumeric::Int32(100).mul(&GenericNumeric::Int32(200), config).unwrap(),
        GenericNumeric::Int32(20000)
    );
    assert_eq!(
        GenericNumeric::UInt32(100).mul(&GenericNumeric::UInt32(200), config).unwrap(),
        GenericNumeric::UInt32(20000)
    );
    assert_eq!(
        GenericNumeric::Int64(1000).mul(&GenericNumeric::Int64(2000), config).unwrap(),
        GenericNumeric::Int64(2000000)
    );
    assert_eq!(
        GenericNumeric::UInt64(1000).mul(&GenericNumeric::UInt64(2000), config).unwrap(),
        GenericNumeric::UInt64(2000000)
    );
    assert_eq!(
        GenericNumeric::Float(2.5).mul(&GenericNumeric::Float(4.0), config).unwrap(),
        GenericNumeric::Float(10.0)
    );
    assert_eq!(
        GenericNumeric::Double(2.5).mul(&GenericNumeric::Double(4.0), config).unwrap(),
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
    assert!(GenericNumeric::Int8(i8::MAX).mul(&GenericNumeric::Int8(2), config).is_err());
    assert!(GenericNumeric::UInt8(u8::MAX).mul(&GenericNumeric::UInt8(2), config).is_err());
}

#[test]
fn test_div() {
    let config = EvalConfig::default();
    
    // Test same-type division
    assert_eq!(
        GenericNumeric::Int8(30).div(&GenericNumeric::Int8(6), config).unwrap(),
        GenericNumeric::Int8(5)
    );
    assert_eq!(
        GenericNumeric::UInt8(30).div(&GenericNumeric::UInt8(6), config).unwrap(),
        GenericNumeric::UInt8(5)
    );
    assert_eq!(
        GenericNumeric::Int16(200).div(&GenericNumeric::Int16(10), config).unwrap(),
        GenericNumeric::Int16(20)
    );
    assert_eq!(
        GenericNumeric::UInt16(200).div(&GenericNumeric::UInt16(10), config).unwrap(),
        GenericNumeric::UInt16(20)
    );
    assert_eq!(
        GenericNumeric::Int32(20000).div(&GenericNumeric::Int32(100), config).unwrap(),
        GenericNumeric::Int32(200)
    );
    assert_eq!(
        GenericNumeric::UInt32(20000).div(&GenericNumeric::UInt32(100), config).unwrap(),
        GenericNumeric::UInt32(200)
    );
    assert_eq!(
        GenericNumeric::Int64(2000000).div(&GenericNumeric::Int64(1000), config).unwrap(),
        GenericNumeric::Int64(2000)
    );
    assert_eq!(
        GenericNumeric::UInt64(2000000).div(&GenericNumeric::UInt64(1000), config).unwrap(),
        GenericNumeric::UInt64(2000)
    );
    assert_eq!(
        GenericNumeric::Float(10.0).div(&GenericNumeric::Float(2.5), config).unwrap(),
        GenericNumeric::Float(4.0)
    );
    assert_eq!(
        GenericNumeric::Double(10.0).div(&GenericNumeric::Double(2.5), config).unwrap(),
        GenericNumeric::Double(4.0)
    );
}

#[test]
fn test_div_by_zero() {
    let config = EvalConfig::default();
    
    // Test division by zero for integers
    assert!(GenericNumeric::Int8(10).div(&GenericNumeric::Int8(0), config).is_err());
    assert!(GenericNumeric::UInt8(10).div(&GenericNumeric::UInt8(0), config).is_err());
    assert!(GenericNumeric::Int16(10).div(&GenericNumeric::Int16(0), config).is_err());
    assert!(GenericNumeric::UInt16(10).div(&GenericNumeric::UInt16(0), config).is_err());
    assert!(GenericNumeric::Int32(10).div(&GenericNumeric::Int32(0), config).is_err());
    assert!(GenericNumeric::UInt32(10).div(&GenericNumeric::UInt32(0), config).is_err());
    assert!(GenericNumeric::Int64(10).div(&GenericNumeric::Int64(0), config).is_err());
    assert!(GenericNumeric::UInt64(10).div(&GenericNumeric::UInt64(0), config).is_err());
    
    // Floating point division by zero produces infinity (IEEE 754)
    let result = GenericNumeric::Float(1.0).div(&GenericNumeric::Float(0.0), config).unwrap();
    if let GenericNumeric::Float(v) = result {
        assert!(v.is_infinite());
    } else {
        panic!("Expected Float result");
    }
    
    let result = GenericNumeric::Double(1.0).div(&GenericNumeric::Double(0.0), config).unwrap();
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
        GenericNumeric::Int8(17).modulo(&GenericNumeric::Int8(5), config).unwrap(),
        GenericNumeric::Int8(2)
    );
    assert_eq!(
        GenericNumeric::UInt8(17).modulo(&GenericNumeric::UInt8(5), config).unwrap(),
        GenericNumeric::UInt8(2)
    );
    assert_eq!(
        GenericNumeric::Int16(100).modulo(&GenericNumeric::Int16(30), config).unwrap(),
        GenericNumeric::Int16(10)
    );
    assert_eq!(
        GenericNumeric::UInt16(100).modulo(&GenericNumeric::UInt16(30), config).unwrap(),
        GenericNumeric::UInt16(10)
    );
    assert_eq!(
        GenericNumeric::Int32(1000).modulo(&GenericNumeric::Int32(300), config).unwrap(),
        GenericNumeric::Int32(100)
    );
    assert_eq!(
        GenericNumeric::UInt32(1000).modulo(&GenericNumeric::UInt32(300), config).unwrap(),
        GenericNumeric::UInt32(100)
    );
    assert_eq!(
        GenericNumeric::Int64(10000).modulo(&GenericNumeric::Int64(3000), config).unwrap(),
        GenericNumeric::Int64(1000)
    );
    assert_eq!(
        GenericNumeric::UInt64(10000).modulo(&GenericNumeric::UInt64(3000), config).unwrap(),
        GenericNumeric::UInt64(1000)
    );
    assert_eq!(
        GenericNumeric::Float(10.5).modulo(&GenericNumeric::Float(3.0), config).unwrap(),
        GenericNumeric::Float(1.5)
    );
    assert_eq!(
        GenericNumeric::Double(10.5).modulo(&GenericNumeric::Double(3.0), config).unwrap(),
        GenericNumeric::Double(1.5)
    );
}

#[test]
fn test_modulo_by_zero() {
    let config = EvalConfig::default();
    
    // Test modulo by zero for integers
    assert!(GenericNumeric::Int8(10).modulo(&GenericNumeric::Int8(0), config).is_err());
    assert!(GenericNumeric::UInt8(10).modulo(&GenericNumeric::UInt8(0), config).is_err());
    
    // Floating point modulo by zero produces NaN (IEEE 754)
    let result = GenericNumeric::Float(1.0).modulo(&GenericNumeric::Float(0.0), config).unwrap();
    if let GenericNumeric::Float(v) = result {
        assert!(v.is_nan());
    } else {
        panic!("Expected Float result");
    }
}

#[test]
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
        GenericNumeric::Int32(0xFF00FF00u32 as i32).bit_and(&GenericNumeric::Int32(0x0FF00FF0u32 as i32)),
        GenericNumeric::Int32(0x0F000F00u32 as i32)
    );
    assert_eq!(
        GenericNumeric::UInt32(0xFF00FF00).bit_and(&GenericNumeric::UInt32(0x0FF00FF0)),
        GenericNumeric::UInt32(0x0F000F00)
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
        GenericNumeric::Float(3.14).bit_and(&GenericNumeric::Float(2.718)),
        GenericNumeric::Float(3.14)
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
        GenericNumeric::Int8(1).shl(&GenericNumeric::Int8(2), config).unwrap(),
        GenericNumeric::Int8(4)
    );
    assert_eq!(
        GenericNumeric::UInt8(1).shl(&GenericNumeric::UInt8(2), config).unwrap(),
        GenericNumeric::UInt8(4)
    );
    assert_eq!(
        GenericNumeric::Int16(1).shl(&GenericNumeric::Int16(8), config).unwrap(),
        GenericNumeric::Int16(256)
    );
    assert_eq!(
        GenericNumeric::UInt16(1).shl(&GenericNumeric::UInt16(8), config).unwrap(),
        GenericNumeric::UInt16(256)
    );
    assert_eq!(
        GenericNumeric::Int32(1).shl(&GenericNumeric::Int32(16), config).unwrap(),
        GenericNumeric::Int32(65536)
    );
    assert_eq!(
        GenericNumeric::UInt32(1).shl(&GenericNumeric::UInt32(16), config).unwrap(),
        GenericNumeric::UInt32(65536)
    );
    assert_eq!(
        GenericNumeric::Int64(1).shl(&GenericNumeric::Int64(32), config).unwrap(),
        GenericNumeric::Int64(4294967296)
    );
    assert_eq!(
        GenericNumeric::UInt64(1).shl(&GenericNumeric::UInt64(32), config).unwrap(),
        GenericNumeric::UInt64(4294967296)
    );
    
    // Test negative shift (invalid)
    assert!(GenericNumeric::Int32(1).shl(&GenericNumeric::Int32(-1), config).is_err());
    
    // Test shift amount too large
    assert!(GenericNumeric::Int32(1).shl(&GenericNumeric::Int32(10000), config).is_err());
    
    // Test floating point (returns self)
    assert_eq!(
        GenericNumeric::Float(3.14).shl(&GenericNumeric::Int32(2), config).unwrap(),
        GenericNumeric::Float(3.14)
    );
}

#[test]
fn test_shift_right() {
    let config = EvalConfig::default();
    
    // Test normal shifts
    assert_eq!(
        GenericNumeric::Int8(16).shr(&GenericNumeric::Int8(2), config).unwrap(),
        GenericNumeric::Int8(4)
    );
    assert_eq!(
        GenericNumeric::UInt8(16).shr(&GenericNumeric::UInt8(2), config).unwrap(),
        GenericNumeric::UInt8(4)
    );
    assert_eq!(
        GenericNumeric::Int16(256).shr(&GenericNumeric::Int16(8), config).unwrap(),
        GenericNumeric::Int16(1)
    );
    assert_eq!(
        GenericNumeric::UInt16(256).shr(&GenericNumeric::UInt16(8), config).unwrap(),
        GenericNumeric::UInt16(1)
    );
    
    // Test arithmetic right shift for negative numbers
    assert_eq!(
        GenericNumeric::Int8(-16).shr(&GenericNumeric::Int8(2), config).unwrap(),
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
    assert!(NumericValue::eq(&GenericNumeric::Int32(5), &GenericNumeric::Int32(5)));
    assert!(!NumericValue::eq(&GenericNumeric::Int32(5), &GenericNumeric::Int32(10)));
    assert!(NumericValue::eq(&GenericNumeric::Float(1.0), &GenericNumeric::Float(1.0)));
    assert!(NumericValue::eq(&GenericNumeric::Bool(true), &GenericNumeric::Bool(true)));
    
    // Test inequality
    assert!(!NumericValue::ne(&GenericNumeric::Int32(5), &GenericNumeric::Int32(5)));
    assert!(NumericValue::ne(&GenericNumeric::Int32(5), &GenericNumeric::Int32(10)));
    
    // Test char comparisons
    assert!(GenericNumeric::Char('A').lt(&GenericNumeric::Char('B')));
    assert!(GenericNumeric::Char('Z').gt(&GenericNumeric::Char('A')));
    assert!(NumericValue::eq(&GenericNumeric::Char('X'), &GenericNumeric::Char('X')));
}

#[test]
fn test_comparison_different_types() {
    // Different types should return false for comparisons (except eq/ne)
    assert!(!GenericNumeric::Int32(5).lt(&GenericNumeric::Float(10.0)));
    assert!(!GenericNumeric::Int32(5).gt(&GenericNumeric::Float(10.0)));
    assert!(!NumericValue::eq(&GenericNumeric::Int32(5), &GenericNumeric::Float(5.0)));
    assert!(NumericValue::ne(&GenericNumeric::Int32(5), &GenericNumeric::Float(5.0)));
}

#[test]
fn test_mixed_type_operations() {
    let config = EvalConfig::default();
    
    // Test Bool + Int8 -> Int8
    let result = GenericNumeric::Bool(true).add(&GenericNumeric::Int8(5), config).unwrap();
    assert_eq!(result, GenericNumeric::Int8(6));
    
    // Test Char + Int16 -> Int16
    let result = GenericNumeric::Char('A').add(&GenericNumeric::Int16(1), config).unwrap();
    assert_eq!(result, GenericNumeric::Int16(66)); // 'A' is 65
    
    // Test UInt8 + Int16 -> Int16
    let result = GenericNumeric::UInt8(100).add(&GenericNumeric::Int16(200), config).unwrap();
    assert_eq!(result, GenericNumeric::Int16(300));
    
    // Test Int32 + Float -> Float
    let result = GenericNumeric::Int32(10).add(&GenericNumeric::Float(0.5), config).unwrap();
    assert_eq!(result, GenericNumeric::Float(10.5));
    
    // Test Float + Double -> Double
    let result = GenericNumeric::Float(1.0).add(&GenericNumeric::Double(0.1), config).unwrap();
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
    // But since add() doesn't handle Char+Char, we expect it to fail
    let result = a.add(&GenericNumeric::Char('\u{0001}'), config);
    assert!(result.is_err(), "Char arithmetic not implemented");
}

#[test]
fn test_overflow_edge_cases() {
    let config = EvalConfig {
        overflow: OverflowBehavior::Error,
        ..Default::default()
    };
    
    // Test i32::MIN / -1 overflow
    assert!(GenericNumeric::Int32(i32::MIN).div(&GenericNumeric::Int32(-1), config).is_err());
    
    // Test i64::MIN / -1 overflow  
    assert!(GenericNumeric::Int64(i64::MIN).div(&GenericNumeric::Int64(-1), config).is_err());
    
    // Test i32::MIN % -1 overflow
    assert!(GenericNumeric::Int32(i32::MIN).modulo(&GenericNumeric::Int32(-1), config).is_err());
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