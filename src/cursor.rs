use super::*;

#[derive(Debug, Default)]
pub struct Cursor<T> {
    pub data: T,
    pub offset: usize,
}

impl<T: Bytes> Cursor<T> {
    /// Writes a slice into the data view.
    ///
    /// # Examples
    ///
    /// ```
    /// use bin_layout::{Cursor, ErrorKind};
    ///
    /// let mut view = Cursor::new([0; 3]);
    ///
    /// assert_eq!(view.write_slice([4, 2]), Ok(()));
    /// assert_eq!(view.write_slice([1, 2, 3]), Err(ErrorKind::InsufficientBytes));
    ///
    /// assert_eq!(view.data, [4, 2, 0]);
    /// assert_eq!(view.offset, 2);
    /// ```
    #[inline]
    pub fn write_slice(&mut self, slice: impl AsRef<[u8]>) {
        self.offset = self.data.write_slice_at(self.offset, slice);
    }
}


impl<'de> Cursor<&'de [u8]> {
    /// Returns remaining slice from the current offset.
    /// It doesn't change the offset.
    ///
    /// # Examples
    ///
    /// ```
    /// use bin_layout::Cursor;
    ///
    /// let mut view = Cursor::new([1, 2].as_ref());
    ///
    /// assert_eq!(view.remaining_slice(), &[1, 2]);
    /// view.offset = 42;
    /// assert!(view.remaining_slice().is_empty());
    /// ```
    #[inline]
    pub fn remaining_slice(&self) -> &'de [u8] {
        unsafe { self.data.get_unchecked(self.offset.min(self.data.len())..) }
    }

    /// Read slice from the current offset.
    ///
    /// # Example
    /// ```
    /// use bin_layout::Cursor;
    /// let mut view = Cursor::new([1, 2, 3].as_ref());
    ///
    /// assert_eq!(view.read_slice(2), Ok([1, 2].as_ref()));
    /// assert!(view.read_slice(3).is_err());
    /// ```
    #[inline]
    pub fn read_slice(&mut self, len: usize) -> Result<&'de [u8]> {
        let total_len = self.offset + len;
        let slice = self
            .data
            .get(self.offset..total_len)
            .ok_or(InsufficientBytes)?;

        self.offset = total_len;
        Ok(slice)
    }
}

impl<T> Cursor<T> {
    #[inline]
    pub const fn new(data: T) -> Self {
        Self { data, offset: 0 }
    }
}

impl<T> From<T> for Cursor<T> {
    #[inline]
    fn from(data: T) -> Self {
        Self::new(data)
    }
}
