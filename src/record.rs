use super::*;
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use std::fmt;

pub trait LenType: TryFrom<usize> + Encoder + for<'de> Decoder<'de> {}
impl LenType for u8 {}
impl LenType for u16 {}
impl LenType for u32 {}
impl LenType for u64 {}
impl LenType for usize {}

/// `Record` can be used to represent fixed-size integer to represent the length of a record.
///
/// It accepts fixed-length unsigned interger type of `N` (`u8`, `u32`, `usize`, etc..) and a generic type of `T` (`Vec<T>`, `String` etc..)
/// ### Example
///
/// ```rust
/// use bin_layout::{Record, Encoder};
///
/// let record: Record<u8, &str> = "HelloWorld".into();
/// assert_eq!(record.len(), 10);
///
/// let bytes = record.encode();
/// assert_eq!(bytes.len(), 11);
/// ```
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Record<L: LenType, T> {
    pub data: T,
    _marker: PhantomData<L>,
}

// ---------------------------------------------------------------------------------

macro_rules! impls {
    [Encoder for $($ty:ty),*] => {$(
        impl<L: LenType> Encoder for Record<L, $ty>
        where
            L::Error: fmt::Debug,
        {
            #[inline] fn size_hint(&self) -> usize {
                let bytes: &[u8] = self.data.as_ref();
                size_of::<L>() + bytes.len()
            }
            #[inline] fn encoder(&self, c: &mut impl Write) -> Result<()> {
                let len: L = self.data.len().try_into().unwrap();
                len.encoder(c)?;
                c.write_all(&self.data.as_ref())
            }
        }
    )*};
}
impls!(Encoder for &[u8], &str, String);

impl<'de, L: LenType> Decoder<'de> for Record<L, &'de [u8]>
where
    usize: TryFrom<L>,
    <usize as TryFrom<L>>::Error: fmt::Debug,
{
    fn decoder(c: &mut &'de [u8]) -> Result<Self> {
        let len: usize = L::decoder(c)?.try_into().unwrap();
        get_slice(c, len).map(Record::new)
    }
}

impl<'de, L: LenType> Decoder<'de> for Record<L, &'de str>
where
    usize: TryFrom<L>,
    <usize as TryFrom<L>>::Error: fmt::Debug,
{
    fn decoder(c: &mut &'de [u8]) -> Result<Self> {
        let bytes = <Record<L, &[u8]>>::decoder(c)?;
        core::str::from_utf8(bytes.data)
            .map_err(invalid_data)
            .map(Record::new)
    }
}
impl<L: LenType> Decoder<'_> for Record<L, String>
where
    usize: TryFrom<L>,
    <usize as TryFrom<L>>::Error: fmt::Debug,
{
    fn decoder(c: &mut &[u8]) -> Result<Self> {
        let bytes = <Record<L, &[u8]>>::decoder(c)?;
        String::from_utf8(bytes.data.to_vec())
            .map_err(invalid_data)
            .map(Record::new)
    }
}

impl<L, T: Encoder> Encoder for Record<L, Vec<T>>
where
    L: LenType,
    L::Error: fmt::Debug,
{
    #[inline]
    fn size_hint(&self) -> usize {
        size_of::<L>() + self.iter().map(T::size_hint).sum::<usize>()
    }

    #[inline]
    fn encoder(&self, c: &mut impl Write) -> Result<()> {
        let len: L = self.data.len().try_into().unwrap();
        len.encoder(c)?;

        for record in &self.data {
            record.encoder(c)?;
        }
        Ok(())
    }
}

impl<'de, L: LenType, T> Decoder<'de> for Record<L, Vec<T>>
where
    T: Decoder<'de>,
    usize: TryFrom<L>,
    <usize as TryFrom<L>>::Error: fmt::Debug,
{
    #[inline]
    fn decoder(c: &mut &'de [u8]) -> Result<Self> {
        let len: usize = L::decoder(c)?.try_into().unwrap();

        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::decoder(c)?);
        }
        Ok(Record::new(vec))
    }
}

impl<L: LenType, T> Record<L, T> {
    pub const fn new(data: T) -> Self {
        Self {
            data,
            _marker: PhantomData,
        }
    }
}
impl<L: LenType, T> From<T> for Record<L, T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}
impl<L: LenType, T: fmt::Debug> fmt::Debug for Record<L, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}
impl<L: LenType, T> Deref for Record<L, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<L: LenType, T> DerefMut for Record<L, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
