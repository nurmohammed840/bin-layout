use crate::*;

macro_rules! encode_len {
    [$c: expr, $len: expr] => {
        let len = $len.try_into().unwrap();
        Len::new(len)
            .ok_or(Error::new(ErrorKind::InvalidInput, format!("Max payload length: {}, But got {len}", Len::MAX)))?
            .encoder($c)?;
    }
}

macro_rules! impls {
    [Encoder for $($ty:ty),*] => {$(
        impl Encoder for $ty {
            #[inline] fn size_hint(&self) -> usize {
                let bytes: &[u8] = self.as_ref();
                Len::SIZE + bytes.len()
            }
            #[inline] fn encoder(&self, c: &mut impl Write) -> Result<()> {
                encode_len!(c, self.len());
                c.write_all(self.as_ref())
            }
    })*};
}
impls!(Encoder for &[u8], &str, String);

impl<'de> Decoder<'de> for &'de [u8] {
    #[inline]
    fn decoder(c: &mut &'de [u8]) -> Result<Self> {
        let len = Len::decoder(c)?.into_inner();
        get_slice(c, len as usize)
    }
}

impl<'de> Decoder<'de> for &'de str {
    #[inline]
    fn decoder(c: &mut &'de [u8]) -> Result<Self> {
        std::str::from_utf8(Decoder::decoder(c)?).map_err(invalid_data)
    }
}
impl Decoder<'_> for String {
    #[inline]
    fn decoder(c: &mut &[u8]) -> Result<Self> {
        String::from_utf8(<&[u8]>::decoder(c)?.to_vec()).map_err(invalid_data)
    }
}

impl<T: Encoder> Encoder for Vec<T> {
    #[inline]
    fn size_hint(&self) -> usize {
        Len::SIZE + self.iter().map(T::size_hint).sum::<usize>()
    }

    #[inline]
    fn encoder(&self, c: &mut impl Write) -> Result<()> {
        encode_len!(c, self.len());

        for item in self {
            item.encoder(c)?;
        }
        Ok(())
    }
}

impl<'de, T: Decoder<'de>> Decoder<'de> for Vec<T> {
    #[inline]
    fn decoder(c: &mut &'de [u8]) -> Result<Self> {
        let len = Len::decoder(c)?.into_inner();
        let mut vec = Vec::with_capacity(len as usize);
        for _ in 0..len {
            vec.push(T::decoder(c)?);
        }
        Ok(vec)
    }
}

// ---------------------------------------------------------------------

impl<T: Encoder> Encoder for Box<T> {
    const SIZE: usize = size_of::<T>();
    #[inline]
    fn encoder(&self, c: &mut impl Write) -> Result<()> {
        T::encoder(self, c)
    }
}

impl<'de, T: Decoder<'de>> Decoder<'de> for Box<T> {
    #[inline]
    fn decoder(c: &mut &'de [u8]) -> Result<Self> {
        T::decoder(c).map(|v| Box::new(v))
    }
}
