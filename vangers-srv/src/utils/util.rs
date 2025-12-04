use std::convert::TryInto;

use ::encoding::all::IBM866;
use ::encoding::{DecoderTrap, Encoding};

/// Returns a valid CStr with null-terminate byte from `bytes` slice.
/// Returns `None` if the first byte is `0x00`
pub fn get_first_cstr(bytes: &[u8]) -> Option<&[u8]> {
    if bytes.is_empty() || bytes[0] == 0 {
        // return `None` if first byte is null-terminator
        return None;
    }

    bytes
        .iter()
        .enumerate()
        .find(|(_, &v)| v == 0)
        .map(|(i, _)| &bytes[0..=i])
}

#[allow(dead_code)]
pub fn convert_cp866_to_utf8(cstr: &[u8]) -> Option<String> {
    IBM866.decode(cstr, DecoderTrap::Replace).ok()
}

#[allow(dead_code)]
#[inline(always)]
pub fn slice_le_to_u16(s: &[u8]) -> u16 {
    u16::from_le_bytes(s.try_into().unwrap())
}

#[inline]
pub fn slice_le_to_i16(s: &[u8]) -> i16 {
    i16::from_le_bytes(s.try_into().unwrap())
    // if s.len() != 2 {
    //     panic!("slice length must be equal 2");
    // }

    // (s[0] as i16) | ((s[1] as i16) << 8)
}

#[inline(always)]
pub fn slice_le_to_u32(s: &[u8]) -> u32 {
    u32::from_le_bytes(s.try_into().unwrap())
}

#[inline]
pub fn slice_le_to_i32(s: &[u8]) -> i32 {
    i32::from_le_bytes(s.try_into().unwrap())
    // if s.len() != 4 {
    //     panic!("slice length must be equal 4");
    // }

    // (s[0] as i32) | ((s[1] as i32) << 8) | ((s[2] as i32) << 16) | ((s[3] as i32) << 24)
}

#[allow(dead_code)]
pub fn slice_i32_to_vec_u8(slice: &[i32]) -> Vec<u8> {
    slice
        .iter()
        .map(|s| s.to_le_bytes())
        .fold(vec![], |mut acc, x| {
            x.iter().for_each(|&x| acc.push(x));
            acc
        })
}

// use std::iter::{FromIterator, Iterator};
// helper that help remove boilerplate code like `.map(|&byte| byte).collect()`
// impl<T> CollectValuesByRefs<A> for T {}
// trait CollectValuesByRefs {
//     fn collect_refs<'a, A, B>(self) -> B
//     where
//     A: Copy + 'a,
//     Self: Sized + Iterator<Item=&'a A>,
//     B: FromIterator<A> {
//         return self.map(|a| *a).collect();
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    mod panic {
        use super::*;

        #[test]
        #[should_panic]
        fn test_slice_to_i16_should_panic_0() {
            slice_le_to_i16(&[]);
        }

        #[test]
        #[should_panic]
        fn test_slice_to_i16_should_panic_1() {
            slice_le_to_i16(&[1]);
        }

        #[test]
        #[should_panic]
        fn test_slice_to_i16_should_panic_3() {
            slice_le_to_i16(&[1, 2, 3]);
        }

        #[test]
        #[should_panic]
        fn test_slice_to_i32_should_panic_0() {
            slice_le_to_i32(&[]);
        }

        #[test]
        #[should_panic]
        fn test_slice_to_i32_should_panic_3() {
            slice_le_to_i32(&[1, 2, 3]);
        }

        #[test]
        #[should_panic]
        fn test_slice_to_i32_should_panic_5() {
            slice_le_to_i32(&[1, 2, 3, 4, 5]);
        }
    }

    #[test]
    fn test_slice_to_i32() {
        let digit = 9452468i32;
        let arr = digit.to_le_bytes();
        assert_eq!(digit, slice_le_to_i32(&arr));

        let digit = -452001i32;
        let arr = digit.to_le_bytes();
        assert_eq!(digit, slice_le_to_i32(&arr));
    }

    #[test]
    fn test_slice_to_i16() {
        let digit = 10419i16;
        let arr = digit.to_le_bytes();
        assert_eq!(digit, slice_le_to_i16(&arr));

        let digit = -2845i16;
        let arr = digit.to_le_bytes();
        assert_eq!(digit, slice_le_to_i16(&arr));
    }

    #[allow(non_upper_case_globals)]
    const data: &'static [u8] = &[
        /* == CP-866 == */
        0x74, 0x65, 0x73, 0x74, // test
        0x00, // null-terminator
        0x82, 0x82, 0x85, 0x84, 0x88, 0x92, 0x85, // ВВЕДИТЕ
        0x20, // space symbol
        0x91, 0x92, 0x90, 0x8E, 0x8A, 0x93, // СТРОКУ
        0x00, 0x35, 0x00, 0x94, 0x00, 0x00, 0x00, 0x00, 0x00, // (cargo-fmt)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00,
    ];

    #[test]
    fn test_get_first_str() {
        assert_eq!(get_first_cstr(&data[0..]), Some(&data[0..5]));
        assert_eq!(get_first_cstr(&data[2..]), Some(&data[2..5]));
        assert_eq!(get_first_cstr(&data[0..2]), None);
        assert_eq!(get_first_cstr(&data[1..3]), None);
        assert_eq!(get_first_cstr(&data[4..]), None);
        assert_eq!(get_first_cstr(&data[5..]), Some(&data[5..20]));
    }
}

// #[test]
// fn test() {
//     let digit = 1927i16;
//     let bytes = digit.to_le_bytes();

//     let digit_recover = (bytes[0] as i16) | ((bytes[1] as i16) << 8);

//     assert_eq!(digit, digit_recover);
//     assert_eq!(bytes[1], (digit >> 8) as u8); // 7
//     assert_eq!(bytes[0], digit as u8); // 135
// }
