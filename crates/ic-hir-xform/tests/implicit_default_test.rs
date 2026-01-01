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

use ic_hir::hir::{DefKind, Numeric, TyKind};
use ic_hir_xform::implicit_default;

fn parse_and_transform(idl: &str) -> ic_hir::ResolvedGraph {
    let mut vfs = ic_vfs::SourceMap::default();
    let file_id = vfs.embed(idl);

    let args = ic_preproc::ProcArgs::default();
    let parsed = ic_parse::from_file(file_id, args, &mut vfs);

    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let hir = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed.tree));
    assert!(hir.errors.is_empty(), "HIR errors: {:?}", hir.errors);

    implicit_default::transform(hir)
}

#[test]
fn test_incomplete_boolean_union() {
    let idl = r"
        union BoolUnion switch (boolean) {
        case TRUE:
            long value;
        };
    ";

    let hir = parse_and_transform(idl);

    // Find the union
    let union_def = hir
        .iter()
        .find(|def| def.ident.name == "BoolUnion")
        .unwrap();
    let DefKind::Union(union_ty) = &union_def.kind else {
        panic!("Expected union")
    };

    // Should have 2 variants: original + implicit default
    assert_eq!(union_ty.variants.len(), 2);

    // Check the implicit default variant
    let implicit = union_ty
        .variants
        .iter()
        .find(|v| v.ident.name == "_implicit_default")
        .unwrap();
    assert!(matches!(implicit.ty.kind, TyKind::Null));
    assert_eq!(implicit.labels.len(), 1);
    assert_eq!(implicit.labels[0].value, Numeric::Bool(false));
}

#[test]
fn test_incomplete_octet_union() {
    let idl = r"
        union OctetUnion switch (octet) {
        case 0:
            string zero;
        case 1:
            string one;
        case 255:
            string max;
        };
    ";

    let hir = parse_and_transform(idl);

    let union_def = hir
        .iter()
        .find(|def| def.ident.name == "OctetUnion")
        .unwrap();
    let DefKind::Union(union_ty) = &union_def.kind else {
        panic!("Expected union")
    };

    // Should have 4 variants: 3 original + implicit default
    assert_eq!(union_ty.variants.len(), 4);

    // Check the implicit default variant
    let implicit = union_ty
        .variants
        .iter()
        .find(|v| v.ident.name == "_implicit_default")
        .unwrap();
    assert!(matches!(implicit.ty.kind, TyKind::Null));
    assert_eq!(implicit.labels.len(), 1);
    assert_eq!(implicit.labels[0].value, Numeric::UInt8(2));
}

#[test]
fn test_complete_boolean_union() {
    let idl = r"
        union CompleteUnion switch (boolean) {
        case TRUE:
            long value;
        case FALSE:
            string text;
        };
    ";

    let hir = parse_and_transform(idl);

    let union_def = hir
        .iter()
        .find(|def| def.ident.name == "CompleteUnion")
        .unwrap();
    let DefKind::Union(union_ty) = &union_def.kind else {
        panic!("Expected union")
    };

    // Should have only 2 variants - no implicit default needed
    assert_eq!(union_ty.variants.len(), 2);
    assert!(
        union_ty
            .variants
            .iter()
            .all(|v| v.ident.name != "_implicit_default")
    );
}

#[test]
fn test_union_with_default() {
    let idl = r"
        union WithDefault switch (long) {
        case 0:
            string zero;
        default:
            double value;
        };
    ";

    let hir = parse_and_transform(idl);

    let union_def = hir
        .iter()
        .find(|def| def.ident.name == "WithDefault")
        .unwrap();
    let DefKind::Union(union_ty) = &union_def.kind else {
        panic!("Expected union")
    };

    // Should have only 2 variants - no implicit default needed
    assert_eq!(union_ty.variants.len(), 2);
    assert!(
        union_ty
            .variants
            .iter()
            .all(|v| v.ident.name != "_implicit_default")
    );
}

#[test]
fn test_enum_discriminator() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };

        union ColorUnion switch (Color) {
        case RED:
            string red_value;
        case GREEN:
            string green_value;
        };
    ";

    let hir = parse_and_transform(idl);

    let union_def = hir
        .iter()
        .find(|def| def.ident.name == "ColorUnion")
        .unwrap();

    let (blue_id, _) = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "BLUE")
        .unwrap();

    let DefKind::Union(union_ty) = &union_def.kind else {
        panic!("Expected union")
    };

    // Should have 3 variants: 2 original + implicit default
    assert_eq!(union_ty.variants.len(), 3);

    // Check the implicit default variant
    let implicit = union_ty
        .variants
        .iter()
        .find(|v| v.ident.name == "_implicit_default")
        .unwrap();
    assert!(matches!(implicit.ty.kind, TyKind::Null));
    assert_eq!(implicit.labels.len(), 1);
    assert_eq!(implicit.labels[0].value, Numeric::Const(blue_id));
}

#[test]
fn test_char_discriminator() {
    let idl = r"
        union CharUnion switch (char) {
        case 'a':
            string a_value;
        case 'b':
            string b_value;
        };
    ";

    let hir = parse_and_transform(idl);

    let union_def = hir
        .iter()
        .find(|def| def.ident.name == "CharUnion")
        .unwrap();
    let DefKind::Union(union_ty) = &union_def.kind else {
        panic!("Expected union")
    };

    // Should have 3 variants: 2 original + implicit default
    assert_eq!(union_ty.variants.len(), 3);

    // Check the implicit default variant
    let implicit = union_ty
        .variants
        .iter()
        .find(|v| v.ident.name == "_implicit_default")
        .unwrap();
    assert!(matches!(implicit.ty.kind, TyKind::Null));
    assert_eq!(implicit.labels.len(), 1);
    // Should find first available char value
    match &implicit.labels[0].value {
        Numeric::Char(c) => assert!(*c != 'a' && *c != 'b'),
        _ => panic!("Expected char value"),
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_complete_octet_union() {
    let idl = r"
        union CompleteOctetUnion switch (octet) {
        case 0: string s0;
        case 1: string s1;
        case 2: string s2;
        case 3: string s3;
        case 4: string s4;
        case 5: string s5;
        case 6: string s6;
        case 7: string s7;
        case 8: string s8;
        case 9: string s9;
        case 10: string s10;
        case 11: string s11;
        case 12: string s12;
        case 13: string s13;
        case 14: string s14;
        case 15: string s15;
        case 16: string s16;
        case 17: string s17;
        case 18: string s18;
        case 19: string s19;
        case 20: string s20;
        case 21: string s21;
        case 22: string s22;
        case 23: string s23;
        case 24: string s24;
        case 25: string s25;
        case 26: string s26;
        case 27: string s27;
        case 28: string s28;
        case 29: string s29;
        case 30: string s30;
        case 31: string s31;
        case 32: string s32;
        case 33: string s33;
        case 34: string s34;
        case 35: string s35;
        case 36: string s36;
        case 37: string s37;
        case 38: string s38;
        case 39: string s39;
        case 40: string s40;
        case 41: string s41;
        case 42: string s42;
        case 43: string s43;
        case 44: string s44;
        case 45: string s45;
        case 46: string s46;
        case 47: string s47;
        case 48: string s48;
        case 49: string s49;
        case 50: string s50;
        case 51: string s51;
        case 52: string s52;
        case 53: string s53;
        case 54: string s54;
        case 55: string s55;
        case 56: string s56;
        case 57: string s57;
        case 58: string s58;
        case 59: string s59;
        case 60: string s60;
        case 61: string s61;
        case 62: string s62;
        case 63: string s63;
        case 64: string s64;
        case 65: string s65;
        case 66: string s66;
        case 67: string s67;
        case 68: string s68;
        case 69: string s69;
        case 70: string s70;
        case 71: string s71;
        case 72: string s72;
        case 73: string s73;
        case 74: string s74;
        case 75: string s75;
        case 76: string s76;
        case 77: string s77;
        case 78: string s78;
        case 79: string s79;
        case 80: string s80;
        case 81: string s81;
        case 82: string s82;
        case 83: string s83;
        case 84: string s84;
        case 85: string s85;
        case 86: string s86;
        case 87: string s87;
        case 88: string s88;
        case 89: string s89;
        case 90: string s90;
        case 91: string s91;
        case 92: string s92;
        case 93: string s93;
        case 94: string s94;
        case 95: string s95;
        case 96: string s96;
        case 97: string s97;
        case 98: string s98;
        case 99: string s99;
        case 100: string s100;
        case 101: string s101;
        case 102: string s102;
        case 103: string s103;
        case 104: string s104;
        case 105: string s105;
        case 106: string s106;
        case 107: string s107;
        case 108: string s108;
        case 109: string s109;
        case 110: string s110;
        case 111: string s111;
        case 112: string s112;
        case 113: string s113;
        case 114: string s114;
        case 115: string s115;
        case 116: string s116;
        case 117: string s117;
        case 118: string s118;
        case 119: string s119;
        case 120: string s120;
        case 121: string s121;
        case 122: string s122;
        case 123: string s123;
        case 124: string s124;
        case 125: string s125;
        case 126: string s126;
        case 127: string s127;
        case 128: string s128;
        case 129: string s129;
        case 130: string s130;
        case 131: string s131;
        case 132: string s132;
        case 133: string s133;
        case 134: string s134;
        case 135: string s135;
        case 136: string s136;
        case 137: string s137;
        case 138: string s138;
        case 139: string s139;
        case 140: string s140;
        case 141: string s141;
        case 142: string s142;
        case 143: string s143;
        case 144: string s144;
        case 145: string s145;
        case 146: string s146;
        case 147: string s147;
        case 148: string s148;
        case 149: string s149;
        case 150: string s150;
        case 151: string s151;
        case 152: string s152;
        case 153: string s153;
        case 154: string s154;
        case 155: string s155;
        case 156: string s156;
        case 157: string s157;
        case 158: string s158;
        case 159: string s159;
        case 160: string s160;
        case 161: string s161;
        case 162: string s162;
        case 163: string s163;
        case 164: string s164;
        case 165: string s165;
        case 166: string s166;
        case 167: string s167;
        case 168: string s168;
        case 169: string s169;
        case 170: string s170;
        case 171: string s171;
        case 172: string s172;
        case 173: string s173;
        case 174: string s174;
        case 175: string s175;
        case 176: string s176;
        case 177: string s177;
        case 178: string s178;
        case 179: string s179;
        case 180: string s180;
        case 181: string s181;
        case 182: string s182;
        case 183: string s183;
        case 184: string s184;
        case 185: string s185;
        case 186: string s186;
        case 187: string s187;
        case 188: string s188;
        case 189: string s189;
        case 190: string s190;
        case 191: string s191;
        case 192: string s192;
        case 193: string s193;
        case 194: string s194;
        case 195: string s195;
        case 196: string s196;
        case 197: string s197;
        case 198: string s198;
        case 199: string s199;
        case 200: string s200;
        case 201: string s201;
        case 202: string s202;
        case 203: string s203;
        case 204: string s204;
        case 205: string s205;
        case 206: string s206;
        case 207: string s207;
        case 208: string s208;
        case 209: string s209;
        case 210: string s210;
        case 211: string s211;
        case 212: string s212;
        case 213: string s213;
        case 214: string s214;
        case 215: string s215;
        case 216: string s216;
        case 217: string s217;
        case 218: string s218;
        case 219: string s219;
        case 220: string s220;
        case 221: string s221;
        case 222: string s222;
        case 223: string s223;
        case 224: string s224;
        case 225: string s225;
        case 226: string s226;
        case 227: string s227;
        case 228: string s228;
        case 229: string s229;
        case 230: string s230;
        case 231: string s231;
        case 232: string s232;
        case 233: string s233;
        case 234: string s234;
        case 235: string s235;
        case 236: string s236;
        case 237: string s237;
        case 238: string s238;
        case 239: string s239;
        case 240: string s240;
        case 241: string s241;
        case 242: string s242;
        case 243: string s243;
        case 244: string s244;
        case 245: string s245;
        case 246: string s246;
        case 247: string s247;
        case 248: string s248;
        case 249: string s249;
        case 250: string s250;
        case 251: string s251;
        case 252: string s252;
        case 253: string s253;
        case 254: string s254;
        case 255: string s255;
        };
    ";

    let hir = parse_and_transform(idl);

    let union_def = hir
        .iter()
        .find(|def| def.ident.name == "CompleteOctetUnion")
        .unwrap();
    let DefKind::Union(union_ty) = &union_def.kind else {
        panic!("Expected union")
    };

    // Should have exactly 256 variants - no implicit default needed
    assert_eq!(union_ty.variants.len(), 256);
    assert!(
        union_ty
            .variants
            .iter()
            .all(|v| v.ident.name != "_implicit_default")
    );
}

#[test]
fn test_multiple_label_variants() {
    let idl = r"
        union MultiLabelUnion switch (short) {
        case 1:
        case 2:
        case 3:
            string small;
        case 100:
        case 200:
            string large;
        };
    ";

    let hir = parse_and_transform(idl);

    let union_def = hir
        .iter()
        .find(|def| def.ident.name == "MultiLabelUnion")
        .unwrap();
    let DefKind::Union(union_ty) = &union_def.kind else {
        panic!("Expected union")
    };

    // Should have 3 variants: 2 original + implicit default
    assert_eq!(union_ty.variants.len(), 3);

    // Check the implicit default variant
    let implicit = union_ty
        .variants
        .iter()
        .find(|v| v.ident.name == "_implicit_default")
        .unwrap();
    assert!(matches!(implicit.ty.kind, TyKind::Null));
    assert_eq!(implicit.labels.len(), 1);
    // Should find first available value (0 in this case)
    assert_eq!(implicit.labels[0].value, Numeric::Int16(0));
}

#[test]
fn test_int8_discriminator() {
    let idl = r"
        union Int8Union switch (int8) {
        case -128:
            string min;
        case 0:
            string zero;
        case 127:
            string max;
        };
    ";

    let hir = parse_and_transform(idl);

    let union_def = hir
        .iter()
        .find(|def| def.ident.name == "Int8Union")
        .unwrap();
    let DefKind::Union(union_ty) = &union_def.kind else {
        panic!("Expected union")
    };

    // Should have 4 variants: 3 original + implicit default
    assert_eq!(union_ty.variants.len(), 4);

    // Check the implicit default variant
    let implicit = union_ty
        .variants
        .iter()
        .find(|v| v.ident.name == "_implicit_default")
        .unwrap();
    assert!(matches!(implicit.ty.kind, TyKind::Null));
    assert_eq!(implicit.labels.len(), 1);
    // Should find first available value
    match &implicit.labels[0].value {
        Numeric::Int8(v) => assert!(*v != -128 && *v != 0 && *v != 127),
        _ => panic!("Expected int8 value"),
    }
}
