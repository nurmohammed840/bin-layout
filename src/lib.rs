#![cfg_attr(feature = "nightly", feature(array_try_map))]
#![doc = include_str!("../README.md")]

mod record;
mod types;
mod view;

use core::convert::TryInto;
use ErrorKind::*;

pub use derive::DataType;
pub use record::Record;
pub use view::DataView;
// pub use view::C;

/// Shortcut for `Result<T, bin_layout::ErrorKind>`
pub type Result<T> = core::result::Result<T, ErrorKind>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    InsufficientBytes,
    InvalidLength,
    InvalidInput,
    Unsupported,
    InvalidType,
    InvalidData,
    InvalidChar,
    InvalidUtf8,
    Other,
}

/// A trait for serialize and deserialize data for binary format.
///
/// All [primitive types](https://doc.rust-lang.org/stable/rust-by-example/primitives.html) implement this trait.
///
/// And For collection types, `Vec` and `String` are supported. They are encoded with their length `u32` value first, Following by each entry of the collection.
pub trait DataType<'de>: Sized {
    /// Serialize the data to binary format.
    fn serialize(self, view: &mut DataView<impl AsMut<[u8]>>);

    /// Deserialize the data from binary format.
    fn deserialize(view: &mut DataView<&'de [u8]>) -> Result<Self>;

    /// Shortcut for `DataType::serialize(self, &mut DataView::new(bytes.as_mut()))`
    /// 
    /// ### Example
    /// 
    /// ```
    /// use bin_layout::DataType;
    /// 
    /// #[derive(DataType)]
    /// struct FooBar {
    ///     foo: u8,
    ///     bar: [u8; 2],
    /// }
    /// 
    /// let mut bytes = [0; 3];
    /// FooBar { foo: 1, bar: [2, 3] }.encode(&mut bytes);
    /// assert_eq!(bytes, [1, 2, 3]);
    /// ```
    #[inline]
    fn encode(self, data: impl AsMut<[u8]>) {
        self.serialize(&mut DataView::new(data));
    }

    /// Shortcut for `DataType::deserialize(&mut DataView::new(bytes.as_ref()))`
    /// 
    /// ### Example
    /// 
    /// ```
    /// use bin_layout::DataType;
    /// 
    /// #[derive(DataType, PartialEq, Debug)]
    /// struct FooBar {
    ///     foo: u8,
    ///     bar: [u8; 2],
    /// }
    /// 
    /// let foobar = FooBar::decode(&[1, 2, 3]).unwrap();
    /// assert_eq!(foobar, FooBar { foo: 1, bar: [2, 3] });
    /// ```
    #[inline]
    fn decode(data: &'de [u8]) -> Result<Self> {
        Self::deserialize(&mut DataView::new(data))
    }
}


pub struct Cursor<T> {
    pub data: T,
    pub offset: usize,
}

impl<T> From<T> for Cursor<T> {
    #[inline]
    fn from(data: T) -> Self {
        Self { data, offset: 0 }
    }
}
