use crate::*;

impl Encoder for bool {
    #[inline]
    fn encoder(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&[*self as u8])
    }
}

impl Decoder<'_> for bool {
    #[inline]
    fn decoder(c: &mut &[u8]) -> Result<Self> {
        u8::decoder(c).map(|byte| byte != 0)
    }
}

impl Encoder for char {
    #[inline]
    fn encoder(&self, c: &mut impl Write) -> io::Result<()> {
        u32::from(*self).encoder(c)
    }
}
impl Decoder<'_> for char {
    #[inline]
    fn decoder(c: &mut &[u8]) -> Result<Self> {
        let num = u32::decoder(c)?;
        char::from_u32(num).ok_or(invalid_data(format!("{num} is not a valid char")))
    }
}

// ----------------------------------------------------------------------------------------------

impl Encoder for u8 {
    #[inline]
    fn encoder(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&[*self])
    }
}

impl Decoder<'_> for u8 {
    #[inline]
    fn decoder(reader: &mut &[u8]) -> Result<Self> {
        if reader.len() > 0 {
            unsafe {
                let slice = reader.get_unchecked(0);
                *reader = reader.get_unchecked(1..);
                Ok(*slice)
            }
        } else {
            Err(invalid_data("Insufficient bytes"))
        }
    }
}

impl Encoder for i8 {
    #[inline]
    fn encoder(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&[*self as u8])
    }
}
impl Decoder<'_> for i8 {
    #[inline]
    fn decoder(c: &mut &[u8]) -> Result<Self> {
        u8::decoder(c).map(|byte| byte as i8)
    }
}
macro_rules! impl_data_type_for {
    [$($rty:ty)*] => ($(
        impl Encoder for $rty {
            #[inline] fn encoder(&self, writer: &mut impl Write) -> io::Result<()> {
                #[cfg(not(any(feature = "BE", feature = "NE")))]
                return writer.write_all(&self.to_le_bytes());
                #[cfg(feature = "BE")]
                return writer.write_all(&self.to_be_bytes());
                #[cfg(feature = "NE")]
                return writer.write_all(&self.to_ne_bytes());
            }
        }
        impl Decoder<'_> for $rty {
            #[inline] fn decoder(c: &mut &[u8]) -> Result<Self> {
                let arr = <&[u8; std::mem::size_of::<Self>()]>::decoder(c)?;
                #[cfg(not(any(feature = "BE", feature = "NE")))]
                return Ok(Self::from_le_bytes(*arr));
                #[cfg(feature = "BE")]
                return Ok(Self::from_be_bytes(*arr));
                #[cfg(feature = "NE")]
                return Ok(Self::from_ne_bytes(*arr));
            }
        }
    )*);
}

impl_data_type_for!(
    u16 u32 u64 u128
    i16 i32 i64 i128
    usize isize
    f32 f64
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaler_type() {
        for word in [0x_A5C11, 0x_C0DE, 0x_DEC0DE, 0x_ADDED, 0x_AB0DE, 0x_CAFE] {
            assert_eq!(word, u32::decode(&word.encode()).unwrap());
        }
        for word in [
            0x_DEAD_BEEF,
            0x_Faded_Face,
            0x_BAD_F00D,
            0x_C01D_C0FFEE,
            0x_C0CA_C01A,
        ] {
            assert_eq!(word, u64::decode(&word.encode()).unwrap());
        }
    }
}
