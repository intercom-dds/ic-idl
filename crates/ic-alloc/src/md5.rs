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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#[rustfmt::skip]
const TRANSFORM: [u32; 65] = [
    0x0000_0000,

    // Round 1
    0xD76A_A478, 0xE8C7_B756, 0x2420_70DB, 0xC1BD_CEEE,
    0xF57C_0FAF, 0x4787_C62A, 0xa830_4613, 0xFD46_9501,
    0x6980_98D8, 0x8B44_F7AF, 0xFFFF_5BB1, 0x895_CD7BE,
    0x6B90_1122, 0xFD98_7193, 0xA679_438E, 0x49B4_0821,

    // Round 2
    0xF61E_2562, 0xC04_0B340, 0x265_E5A51, 0xE9B_6C7AA,
    0xD62_F105D, 0x024_41453, 0xD8A_1E681, 0xE7D_3FBC8,
    0x21E_1CDE6, 0xC33_707D6, 0xF4D_50D87, 0x455_A14ED,
    0xA9E_3E905, 0xFCE_FA3F8, 0x676_F02D9, 0x8D2_A4C8A,

    // Round 3
    0xFFF_A3942, 0x877_1F681, 0x6D9_D6122, 0xFDE_5380C,
    0xA4B_EEA44, 0x4BD_ECFA9, 0xF6B_B4B60, 0xBEB_FBC70,
    0x289_b7ec6, 0xEAA_127FA, 0xD4E_F3085, 0x048_81D05,
    0xD9D_4D039, 0xE6D_B99E5, 0x1fa_27cf8, 0xC4A_C5665,

    // Round 4
    0xF42_92244, 0x432_AFF97, 0xAB9_423A7, 0xFC9_3A039,
    0x655_B59C3, 0x8F0_CCC92, 0xFFE_FF47D, 0x858_45DD1,
    0x6FA_87E4F, 0xFE2_CE6E0, 0xA30_14314, 0x4E0_811A1,
    0xF75_37E82, 0xBD3_AF235, 0x2AD_7D2BB, 0xEB8_6D391,
];

#[rustfmt::skip]
const KSI_ROUND1: [[u32; 3]; 16] = [
    [0, 7, 1],   [1, 12, 2],   [2, 17, 3],   [3, 22, 4],
    [4, 7, 5],   [5, 12, 6],   [6, 17, 7],   [7, 22, 8],
    [8, 7, 9],   [9, 12, 10],  [10, 17, 11], [11, 22, 12],
    [12, 7, 13], [13, 12, 14], [14, 17, 15], [15, 22, 16],
];

#[rustfmt::skip]
const KSI_ROUND2: [[u32; 3]; 16] = [
    [1, 5, 17],  [6, 9, 18],  [11, 14, 19], [0, 20, 20],
    [5, 5, 21],  [10, 9, 22], [15, 14, 23], [4, 20, 24],
    [9, 5, 25],  [14, 9, 26], [3, 14, 27],  [8, 20, 28],
    [13, 5, 29], [2, 9, 30],  [7, 14, 31],  [12, 20, 32],
];

#[rustfmt::skip]
const KSI_ROUND3: [[u32; 3]; 16] = [
    [5, 4, 33],  [8, 11, 34],  [11, 16, 35], [14, 23, 36],
    [1, 4, 37],  [4, 11, 38],  [7, 16, 39],  [10, 23, 40],
    [13, 4, 41], [0, 11, 42],  [3, 16, 43],  [6, 23, 44],
    [9, 4, 45],  [12, 11, 46], [15, 16, 47], [2, 23, 48],
];

#[rustfmt::skip]
const KSI_ROUND4: [[u32; 3]; 16] = [
    [0, 6, 49],  [7, 10, 50],  [14, 15, 51], [5, 21, 52],
    [12, 6, 53], [3, 10, 54],  [10, 15, 55], [1, 21, 56],
    [8, 6, 57],  [15, 10, 58], [6, 15, 59],  [13, 21, 60],
    [4, 6, 61],  [11, 10, 62], [2, 15, 63],  [9, 21, 64],
];

const PADDING: [u8; 64] = [
    0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

struct Context {
    state: [u32; 4],
    count: usize,
    buffer: [u8; 65],
}

impl Default for Context {
    fn default() -> Self {
        Self {
            state: [0x6745_2301, 0xEFCD_AB89, 0x98B_ADCFE, 0x1032_5476],
            count: 0,
            buffer: [0; 65],
        }
    }
}

fn update(bytes: &[u8], len: usize, context: &mut Context) {
    let mut index = context.count & 0x3F;
    context.count += len;

    let mut i = 0;
    let part_len = 64 - index;

    if len >= part_len {
        context.buffer[index..index + part_len].copy_from_slice(&bytes[..part_len]);
        transform(context);

        i = part_len;
        while i + 63 < len {
            context.buffer.copy_from_slice(&bytes[i..i + 64]);
            transform(context);
            i += 64;
        }
        index = 0;
    }
    context.buffer[index..index + len - i].copy_from_slice(&bytes[i..len]);
}

#[allow(clippy::cast_possible_truncation)]
fn finalize(context: &mut Context) {
    let mut bits = [0_u8; 8];
    encode((context.count as u32) << 3, &mut bits[0..]);
    encode((context.count as u32) >> 29, &mut bits[4..]);

    let index = context.count & 0x3F;
    let pad_len = if index < 56 { 56 - index } else { 120 - index };
    update(&PADDING, pad_len, context);
    update(&bits, 8, context);
}

fn extract(context: &Context) -> [u8; 16] {
    let mut digest = [0; 16];
    for i in 0..4 {
        encode(context.state[i], &mut digest[4 * i..]);
    }
    digest
}

type KsiVec = [u32; 3];

fn rol(x: u32, steps: u32) -> u32 {
    x.wrapping_shl(steps) | (x.wrapping_shr(32 - steps))
}

fn aux_f(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (!x & z)
}

fn aux_g(x: u32, y: u32, z: u32) -> u32 {
    (x & z) | (y & !z)
}

fn aux_h(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

fn aux_i(x: u32, y: u32, z: u32) -> u32 {
    y ^ (x | !z)
}

fn round<F>(bytes: &[u32], a: u32, b: u32, c: u32, d: u32, ksi: &KsiVec, aux: F) -> u32
where
    F: FnOnce(u32, u32, u32) -> u32,
{
    let res = aux(b, c, d)
        .wrapping_add(bytes[ksi[0] as usize])
        .wrapping_add(TRANSFORM[ksi[2] as usize]);

    b.wrapping_add(rol(a.wrapping_add(res), ksi[1]))
}

#[allow(clippy::many_single_char_names)]
fn transform(context: &mut Context) {
    let mut x = [0_u32; 16];
    for (i, byte) in x.iter_mut().enumerate() {
        *byte = decode(&context.buffer[4 * i..]);
    }

    let mut a = context.state[0];
    let mut b = context.state[1];
    let mut c = context.state[2];
    let mut d = context.state[3];

    for i in 0..4 {
        a = round(&x, a, b, c, d, &KSI_ROUND1[4 * i], aux_f);
        d = round(&x, d, a, b, c, &KSI_ROUND1[1 + 4 * i], aux_f);
        c = round(&x, c, d, a, b, &KSI_ROUND1[2 + 4 * i], aux_f);
        b = round(&x, b, c, d, a, &KSI_ROUND1[3 + 4 * i], aux_f);
    }

    for i in 0..4 {
        a = round(&x, a, b, c, d, &KSI_ROUND2[4 * i], aux_g);
        d = round(&x, d, a, b, c, &KSI_ROUND2[1 + 4 * i], aux_g);
        c = round(&x, c, d, a, b, &KSI_ROUND2[2 + 4 * i], aux_g);
        b = round(&x, b, c, d, a, &KSI_ROUND2[3 + 4 * i], aux_g);
    }

    for i in 0..4 {
        a = round(&x, a, b, c, d, &KSI_ROUND3[4 * i], aux_h);
        d = round(&x, d, a, b, c, &KSI_ROUND3[1 + 4 * i], aux_h);
        c = round(&x, c, d, a, b, &KSI_ROUND3[2 + 4 * i], aux_h);
        b = round(&x, b, c, d, a, &KSI_ROUND3[3 + 4 * i], aux_h);
    }

    for i in 0..4 {
        a = round(&x, a, b, c, d, &KSI_ROUND4[4 * i], aux_i);
        d = round(&x, d, a, b, c, &KSI_ROUND4[1 + 4 * i], aux_i);
        c = round(&x, c, d, a, b, &KSI_ROUND4[2 + 4 * i], aux_i);
        b = round(&x, b, c, d, a, &KSI_ROUND4[3 + 4 * i], aux_i);
    }

    context.state[0] = context.state[0].wrapping_add(a);
    context.state[1] = context.state[1].wrapping_add(b);
    context.state[2] = context.state[2].wrapping_add(c);
    context.state[3] = context.state[3].wrapping_add(d);
}

fn decode(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn encode(value: u32, dest: &mut [u8]) {
    dest[0..4].copy_from_slice(&value.to_le_bytes());
}

/// Calculates the MD5 hash of the given data.
pub fn digest<T: AsRef<[u8]>>(data: T) -> [u8; 16] {
    let mut builder = Md5Builder::default();
    builder.append(data);
    builder.digest()
}

#[derive(Default)]
pub struct Md5Builder {
    ctx: Context,
}

impl Md5Builder {
    pub fn append<T: AsRef<[u8]>>(&mut self, data: T) {
        let bytes = data.as_ref();
        update(bytes, bytes.len(), &mut self.ctx);
    }

    #[must_use]
    pub fn digest(mut self) -> [u8; 16] {
        finalize(&mut self.ctx);
        extract(&self.ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::digest;

    #[test]
    fn test_md5() {
        let hash = digest(b"");
        assert_eq!(
            &hash,
            b"\xd4\x1d\x8c\xd9\x8f\x00\xb2\x04\xe9\x80\x09\x98\xec\xf8\x42\x7e"
        );

        let hash = digest(b"a");
        assert_eq!(
            &hash,
            b"\x0c\xc1\x75\xb9\xc0\xf1\xb6\xa8\x31\xc3\x99\xe2\x69\x77\x26\x61"
        );

        let hash = digest(b"abc");
        assert_eq!(
            &hash,
            b"\x90\x01\x50\x98\x3c\xd2\x4f\xb0\xd6\x96\x3f\x7d\x28\xe1\x7f\x72"
        );

        let hash = digest(b"message digest");
        assert_eq!(
            &hash,
            b"\xf9\x6b\x69\x7d\x7c\xb7\x93\x8d\x52\x5a\x2f\x31\xaa\xf1\x61\xd0",
        );

        let hash = digest(b"abcdefghijklmnopqrstuvwxyz");
        assert_eq!(
            &hash,
            b"\xc3\xfc\xd3\xd7\x61\x92\xe4\x00\x7d\xfb\x49\x6c\xca\x67\xe1\x3b"
        );
    }
}
