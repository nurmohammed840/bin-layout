use databuf::error::IntegerOverflow;
use databuf::var_int::*;
use databuf::{
    config::num::{LE, LEB128},
    Decode, Encode,
};

#[test]
fn test_beb128() {
    fn to_bytes<const CONFIG: u8>(
        num: impl Encode + for<'a> Decode<'a> + std::cmp::PartialEq + std::fmt::Debug,
    ) -> Vec<u8> {
        let bytes = num.to_bytes::<CONFIG>();
        let new_num = Decode::from_bytes::<CONFIG>(&bytes).unwrap();
        assert_eq!(num, new_num);
        bytes
    }
    assert_eq!(to_bytes::<LEB128>(u16::MIN), vec![0]);
    assert_eq!(to_bytes::<LEB128>(u16::MAX), vec![255, 255, 3]);
    assert_eq!(to_bytes::<LEB128>(u32::MAX), vec![255, 255, 255, 255, 15]);
    to_bytes::<LEB128>(u64::MAX);
    to_bytes::<LEB128>(u128::MAX);
    to_bytes::<LEB128>(usize::MAX);

    // ------------------------------------

    macro_rules! test_zigzag {
        [$($rty:tt)*] => ($(
            let mut bytes = to_bytes::<LEB128>($rty::MAX);
            bytes[0] += 1;
            assert_eq!(to_bytes::<LEB128>($rty::MIN), bytes);
        )*);
    }
    test_zigzag!(i16 i32 i64 i128 isize);

    // ------------------------------------

    fn check_overflow<T>(num: impl Into<u128>)
    where
        T: for<'de> Decode<'de> + Encode + std::fmt::Debug,
    {
        let bytes = (num.into() + 1).to_bytes::<LEB128>();
        let err = T::from_bytes::<LEB128>(&bytes).unwrap_err();
        assert!(err.is::<IntegerOverflow>());
    }
    check_overflow::<u16>(u16::MAX);
    check_overflow::<u32>(u32::MAX);
    check_overflow::<u64>(u64::MAX);

    let mut bytes = vec![255; 18];
    bytes.push(0b111_u8);
    let err = u128::from_bytes::<LEB128>(&bytes).unwrap_err();
    assert!(err.is::<IntegerOverflow>());
}

macro_rules! assert_varint {
    [$len: expr, $expect: expr] => {
        let bytes = $len.to_bytes::<LE>();
        assert_eq!(bytes, $expect);
        assert_eq!($len, Decode::from_bytes::<LE>(&bytes).unwrap());
    };
}

#[test]
fn test_be_u15() {
    assert_varint!(LEU15(0), [0]);
    assert_varint!(LEU15(127), [127]);

    assert_varint!(LEU15(128), [128, 128]);
    assert_varint!(LEU15(32767), [255, 255]);
}

#[test]
fn test_be_u22() {
    assert_varint!(LEU22(0), [0]);
    assert_varint!(LEU22(127), [127]);

    assert_varint!(LEU22(128), [128, 128]);
    assert_varint!(LEU22(16383), [191, 255]);

    assert_varint!(LEU22(16384), [192, 64, 0]);
    assert_varint!(LEU22(4194303), [255, 255, 255]);
}

#[test]
fn test_be_u29() {
    assert_varint!(LEU29(0), [0]);
    assert_varint!(LEU29(127), [127]);

    assert_varint!(LEU29(128), [128, 128]);
    assert_varint!(LEU29(16383), [191, 255]);

    assert_varint!(LEU29(16384), [192, 64, 0]);
    assert_varint!(LEU29(2097151), [223, 255, 255]);

    assert_varint!(LEU29(2097152), [224, 32, 0, 0]);
    assert_varint!(LEU29(536870911), [255, 255, 255, 255]);
}

#[test]
fn test_be_u30() {
    assert_varint!(LEU30(0), [0]);
    assert_varint!(LEU30(63), [63]);

    assert_varint!(LEU30(64), [64, 64]);
    assert_varint!(LEU30(16383), [127, 255]);

    assert_varint!(LEU30(16384), [128, 64, 0]);
    assert_varint!(LEU30(4194303), [191, 255, 255]);

    assert_varint!(LEU30(4194304), [192, 64, 0, 0]);
    assert_varint!(LEU30(1073741823), [255, 255, 255, 255]);
}

#[test]
fn test_scaler_type() {
    for word in [0x_A5C11, 0x_C0DE, 0x_DEC0DE, 0x_ADDED, 0x_AB0DE, 0x_CAFE] {
        let bytes = word.to_bytes::<LE>();
        assert_eq!(word, u32::from_bytes::<LE>(&bytes).unwrap());
    }
    for word in [
        0x_DEAD_BEEF,
        0x_Faded_Face,
        0x_BAD_F00D,
        0x_C01D_C0FFEE,
        0x_C0CA_C01A,
    ] {
        let bytes = word.to_bytes::<LE>();
        assert_eq!(word, u64::from_bytes::<LE>(&bytes).unwrap());
    }
}
