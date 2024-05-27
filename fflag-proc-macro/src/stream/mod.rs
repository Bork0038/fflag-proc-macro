use num_traits::NumCast;
use std::error::Error;

mod traits;
pub use traits::*;

#[derive(Clone)]
pub struct NetworkStream {
    pub data: Vec<u8>,
    pub read_pointer: usize,
}

impl NetworkStream {
    pub fn new() -> Self {
        NetworkStream {
            data: Vec::new(),
            read_pointer: 0,
        }
    }

    pub fn from<D: AsRef<[u8]>>(data: D) -> Self {
        NetworkStream {
            data: data.as_ref().to_vec(),
            read_pointer: 0,
        }
    }

    pub fn ignore_bytes(&mut self, num_bytes: usize) {
        self.read_pointer += num_bytes;
    }

    pub fn read_to_end(&mut self) -> &[u8] {
        &self.data[self.read_pointer as usize..self.data.len()]
    }

    pub fn write_bytes<D: AsRef<[u8]>>(&mut self, bytes: D) {
        self.data.append(&mut bytes.as_ref().to_vec());
    }

    pub fn read_bytes<'a>(&mut self, num_bytes: usize) -> Result<Vec<u8>, &'a str> {
        let slice = self
            .data
            .get(self.read_pointer..self.read_pointer + num_bytes)
            .map_or(Err("Out of bounds read_bytes attempted"), |slice| Ok(slice))?;

        self.read_pointer += num_bytes;
        Ok(slice.to_vec())
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.data.append(&mut vec![byte]);
    }

    pub fn read_byte<'a>(&mut self) -> Result<u8, &'a str> {
        let byte = self
            .data
            .get(self.read_pointer)
            .map_or(Err("Out of bounds read_bytes attempted"), |slice| Ok(slice))?;

        self.read_pointer += 1;
        Ok(*byte)
    }

    pub fn write_bool(&mut self, bool: bool) {
        self.write_byte(bool as u8);
    }

    pub fn read_bool<'a>(&mut self) -> Result<bool, &'a str> {
        Ok(self.read_byte()? == 0x01)
    }

    pub fn read_le<'a, T>(&mut self) -> Result<T, &'a str>
    where
        T: Bytes,
    {
        Ok(T::from_le_bytes(self.read_bytes(std::mem::size_of::<T>())?))
    }

    pub fn read_be<'a, T>(&mut self) -> Result<T, &'a str>
    where
        T: Bytes,
    {
        Ok(T::from_be_bytes(self.read_bytes(std::mem::size_of::<T>())?))
    }

    pub fn write_le<T>(&mut self, val: T)
    where
        T: Bytes,
    {
        self.write_bytes(T::to_le_bytes(val));
    }

    pub fn write_be<T>(&mut self, val: T)
    where
        T: Bytes,
    {
        self.write_bytes(T::to_be_bytes(val));
    }

    pub fn write_string_le<'a, T>(&mut self, s: &str) -> Result<(), &'a str>
    where
        T: Bytes + NumCast,
    {
        self.write_le::<T>(T::from(s.len()).map_or(Err("Failed to cast string usize"), |d| Ok(d))?);

        self.write_bytes(s.as_bytes().to_vec());
        Ok(())
    }

    pub fn read_string_le<'a, T>(&mut self) -> Result<String, Box<dyn Error>>
    where
        T: Bytes + NumCast,
    {
        let len = <usize as NumCast>::from(self.read_le::<T>()?)
            .map_or(Err("Failed to convert string size_t"), |d| Ok(d))?;

        Ok(String::from_utf8(self.read_bytes(len)?)?)
    }

    pub fn write_string_be<'a, T>(&mut self, s: &str) -> Result<(), &'a str>
    where
        T: Bytes + NumCast,
    {
        self.write_be::<T>(T::from(s.len()).map_or(Err("Failed to cast string usize"), |d| Ok(d))?);

        self.write_bytes(s.as_bytes().to_vec());
        Ok(())
    }

    pub fn read_string_be<'a, T>(&mut self) -> Result<String, Box<dyn Error>>
    where
        T: Bytes + NumCast,
    {
        let len = <usize as NumCast>::from(self.read_be::<T>()?)
            .map_or(Err("Failed to convert string size_t"), |d| Ok(d))?;

        Ok(String::from_utf8(self.read_bytes(len)?)?)
    }

    pub fn read_varint64<'a>(&mut self) -> Result<u64, &'a str> {
        let mut int: u64 = 0;
        let mut idx: u64 = 0;

        loop {
            let byte = self.read_byte()?;

            int |= ((byte & 0x7F) as u64) << idx;
            idx += 7;

            if byte & 0x80 == 0 {
                break;
            }
        }

        Ok(int)
    }

    pub fn read_signedvarint64<'a>(&mut self) -> Result<i64, &'a str> {
        let int = self.read_varint64()?;

        Ok(-((int & 1) as i64) ^ ((int >> 1) as i64))
    }

    pub fn read_varint32<'a>(&mut self) -> Result<u32, &'a str> {
        let mut int: u32 = 0;
        let mut idx: u32 = 0;

        loop {
            let byte = self.read_byte()?;

            int |= ((byte & 0x7F) as u32) << idx;
            idx += 7;

            if byte & 0x80 == 0 {
                break;
            }
        }

        Ok(int)
    }

    pub fn read_signedvarint32<'a>(&mut self) -> Result<i32, &'a str> {
        let int = self.read_varint32()?;

        Ok(-((int & 1) as i32) ^ ((int >> 1) as i32))
    }

    pub fn read_sized_string(&mut self, size: usize) -> Result<String, Box<dyn Error>> {
        Ok(String::from_utf8(self.read_bytes(size)?)?)
    }

    pub fn read_network_string64(&mut self) -> Result<String, Box<dyn Error>> {
        let int = self.read_varint64()?;

        Ok(self.read_sized_string(int as usize)?)
    }

    pub fn read_network_string32(&mut self) -> Result<String, Box<dyn Error>> {
        let int = self.read_varint32()?;

        Ok(self.read_sized_string(int as usize)?)
    }

    pub fn read_float32_be(&mut self) -> Result<f32, Box<dyn Error>> {
        Ok(f32::from_bits(self.read_be::<u32>()?))
    }

    pub fn read_float32_le(&mut self) -> Result<f32, Box<dyn Error>> {
        Ok(f32::from_bits(self.read_le::<u32>()?))
    }

    pub fn read_float64_be(&mut self) -> Result<f64, Box<dyn Error>> {
        Ok(f64::from_bits(self.read_be::<u64>()?))
    }

    pub fn read_float64_le(&mut self) -> Result<f64, Box<dyn Error>> {
        Ok(f64::from_bits(self.read_le::<u64>()?))
    }

    pub fn write<T>(&mut self, obj: &mut T) -> Result<(), Box<dyn Error>>
    where
        T: Serialize<T>,
    {
        obj.write(self)
    }

    pub fn read<T>(&mut self) -> Result<T, Box<dyn Error>>
    where
        T: Serialize<T>,
    {
        T::read(self)
    }
}
