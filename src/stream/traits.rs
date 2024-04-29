use super::NetworkStream;
use std::error::Error;

pub trait Serialize<T> {
    fn write(&mut self, stream: &mut NetworkStream) -> Result<(), Box<dyn Error>>;
    fn read(stream: &mut NetworkStream) -> Result<T, Box<dyn Error>>;
}

pub trait Bytes {
    fn from_le_bytes(bytes: Vec<u8>) -> Self;
    fn from_be_bytes(bytes: Vec<u8>) -> Self;
    fn to_le_bytes(self) -> Vec<u8>;
    fn to_be_bytes(self) -> Vec<u8>;
}

macro_rules! impl_Bytes (( $($int:ident),* )  => {
    $(
        impl Bytes for $int {
            fn from_le_bytes( bytes: Vec<u8> ) -> Self {
                Self::from_le_bytes(
                    bytes
                        .as_slice()
                        .try_into()
                        .unwrap()
                )
            }
            fn from_be_bytes( bytes:  Vec<u8> ) -> Self {
                Self::from_be_bytes(
                    bytes
                        .as_slice()
                        .try_into()
                        .unwrap()
                )
            }
            fn to_le_bytes( self ) -> Vec<u8> {
                self.to_le_bytes().to_vec()
            }
            fn to_be_bytes( self ) -> Vec<u8> {
                self.to_le_bytes().to_vec()
            }
        }
    )*
});

impl_Bytes!(usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128);
