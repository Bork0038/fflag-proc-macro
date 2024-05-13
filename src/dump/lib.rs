use crate::stream::{NetworkStream, Serialize};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use std::error::Error;

#[repr(u32)]
#[derive(FromPrimitive, ToPrimitive, Clone, Debug, PartialEq, Copy)]
pub enum FastVarType {
    Invalid = 0x00,
    Static = 0x01,
    Dynamic = 0x02,
    Sync = 0x04,
    ABNewUsers = 0x05,
    ABNewStudioUsers = 0x10,
    ABAllUsers = 0x20,
    LocalLocked = 0x40,
    Any = 0x7F,
}

impl Serialize<FastVarType> for FastVarType {
    fn read(stream: &mut NetworkStream) -> Result<FastVarType, Box<dyn Error>> {
        let var_type = FastVarType::from_u32(stream.read_le()?).map_or(FastVarType::Invalid, |t| t);

        Ok(var_type)
    }

    fn write(&mut self, stream: &mut NetworkStream) -> Result<(), Box<dyn Error>> {
        stream.write_le(self.clone() as u32);

        Ok(())
    }
}

#[repr(u32)]
#[derive(FromPrimitive, ToPrimitive, Clone, Debug, PartialEq, Copy)]
pub enum FastVarValueType {
    Invalid = 0x00,
    Uninit = 0xFFFFFFFF,

    Log = 0x01,
    String = 0x02,
    Int = 0x03,
    Flag = 0x04,
}

impl Serialize<FastVarValueType> for FastVarValueType {
    fn read(stream: &mut NetworkStream) -> Result<FastVarValueType, Box<dyn Error>> {
        let value_type =
            FastVarValueType::from_u32(stream.read_le()?).map_or(FastVarValueType::Invalid, |t| t);

        Ok(value_type)
    }

    fn write(&mut self, stream: &mut NetworkStream) -> Result<(), Box<dyn Error>> {
        stream.write_le(self.clone() as u32);

        Ok(())
    }
}

#[derive(Clone)]
pub enum FastVarValue {
    Invalid,
    Log(String),
    String(String),
    Int(i32),
    Flag(bool),
    Uninit,
}

impl ToString for FastVarValue {
    fn to_string(&self) -> String {
        match self {
            FastVarValue::Int(val) => val.to_string(),
            FastVarValue::Log(val) => val.to_string(),
            FastVarValue::Flag(val) => val.to_string(),
            FastVarValue::String(val) => val.to_string(),

            FastVarValue::Invalid => String::from("INVALID VALUE TYPE"),
            FastVarValue::Uninit => String::from("VALUE NOT IN INITIALIZED MEMORY"),
        }
    }
}

impl Serialize<FastVarValue> for FastVarValue {
    fn read(stream: &mut NetworkStream) -> Result<FastVarValue, Box<dyn Error>> {
        let value_type: FastVarValueType = stream.read()?;

        let value = match value_type {
            FastVarValueType::Invalid => FastVarValue::Invalid,
            FastVarValueType::Uninit => FastVarValue::Uninit,
            FastVarValueType::Int => FastVarValue::Int(stream.read_le()?),
            FastVarValueType::Flag => FastVarValue::Flag(stream.read_bool()?),
            FastVarValueType::String => FastVarValue::String(stream.read_string_le::<u32>()?),
            FastVarValueType::Log => FastVarValue::Log(stream.read_string_le::<u32>()?),
        };

        Ok(value)
    }

    fn write(&mut self, stream: &mut NetworkStream) -> Result<(), Box<dyn Error>> {
        match self.clone() {
            FastVarValue::Invalid => stream.write(&mut FastVarValueType::Invalid)?,
            FastVarValue::Uninit => stream.write(&mut FastVarValueType::Uninit)?,
            FastVarValue::Int(val) => {
                stream.write(&mut FastVarValueType::Int)?;
                stream.write_le(val);
            }
            FastVarValue::Flag(flag) => {
                stream.write(&mut FastVarValueType::Flag)?;
                stream.write_bool(flag);
            }
            FastVarValue::String(str) => {
                stream.write(&mut FastVarValueType::String)?;
                stream.write_string_le::<u32>(&str)?;
            }
            FastVarValue::Log(log) => {
                stream.write(&mut FastVarValueType::Log)?;
                stream.write_string_le::<u32>(&log)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct FastVar {
    pub name: String,
    pub var_type: FastVarType,
    pub value_type: FastVarValueType,
    pub value: FastVarValue,
}

impl Serialize<FastVar> for FastVar {
    fn read(stream: &mut NetworkStream) -> Result<FastVar, Box<dyn Error>> {
        let var = FastVar {
            name: stream.read_string_le::<u32>()?,
            var_type: stream.read()?,
            value_type: stream.read()?,
            value: stream.read()?,
        };

        Ok(var)
    }

    fn write(&mut self, stream: &mut NetworkStream) -> Result<(), Box<dyn Error>> {
        stream.write_string_le::<u32>(&self.name)?;
        stream.write(&mut self.var_type)?;
        stream.write(&mut self.value_type)?;
        stream.write(&mut self.value)?;

        Ok(())
    }
}
