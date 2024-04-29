// use anyhow::{anyhow, Result};
mod binary;
mod lib;
mod scanner;

pub use lib::*;

use std::{collections::HashMap, error::Error, num::Wrapping};

use crate::stream::NetworkStream;
use binary::Sections;
use scanner::IDAPat;

const DYN_INIT_PAT: &str =
    "41 B8 ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? E9 ?? ?? ?? ??";
const STR_INIT_PAT: &str = "48 83 EC ?? B9 ?? ?? ?? ?? E8 ?? ?? ?? ?? 0F 10 05 ?? ?? ?? ?? 48 C7 05 ?? ?? ?? ?? ?? ?? ?? ??";
const DYN_INIT_SIZE: usize = 25;
const STR_INIT_SIZE: usize = 32;

macro_rules! read_object {
    ($expr:expr) => {
        $expr.get(object::LittleEndian) as usize
    };
}

fn calc_instruction_offset(
    stream: &mut NetworkStream,
    addr: usize,
    base_section: usize,
    new_section: usize,
) -> Result<usize, Box<dyn Error>> {
    let offset: i32 = stream.read_le()?;

    let rva = base_section as usize + addr + stream.read_pointer as usize;
    let real_rva = Wrapping(rva) + Wrapping(offset as usize);

    Ok(real_rva.0 - new_section)
}

fn read_fvar_at_addr(data: &Vec<u8>, addr: usize) -> NetworkStream {
    NetworkStream::from(data[addr..addr + DYN_INIT_SIZE].to_vec())
}

fn read_cstyle_string(data: &Vec<u8>, addr: usize) -> String {
    let mut out = String::new();

    let mut idx = 0;
    loop {
        let byte = data[addr + idx];
        if byte == 0x00 {
            break;
        }

        out.push(byte as char);
        idx += 1;
    }

    out
}

pub fn load_fvar_strings(
    sections: &mut Sections,
    text_data: &Vec<u8>,
    rdata_data: &Vec<u8>,
    text_rva: usize,
    rdata_rva: usize,
    data_rva: usize,
) -> Result<HashMap<usize, String>, Box<dyn Error>> {
    let mut map = HashMap::new();

    let matches =
        scanner::scan::<IDAPat, &str>(sections, &IDAPat::new(STR_INIT_PAT), Some(".text"));

    for addr in matches {
        let mut stream = NetworkStream::from(text_data[addr..addr + STR_INIT_SIZE].to_vec());

        stream.ignore_bytes(17);
        let str_rva = calc_instruction_offset(&mut stream, addr, text_rva, rdata_rva)?;

        stream.ignore_bytes(3);
        let fvar_rva = calc_instruction_offset(&mut stream, addr, text_rva, data_rva + 12)?; // this should be a + 16 but its not ???

        let str_size: u32 = stream.read_le()?;

        let s = String::from_utf8(rdata_data[str_rva..str_rva + str_size as usize].to_vec())
            .map_or(String::new(), |s| s);

        map.insert(fvar_rva, s);
    }

    Ok(map)
}

pub fn get_fflags(binary: Vec<u8>) -> Result<Vec<FastVar>, Box<dyn Error>> {
    let mut sections = binary::get_sections_from_binary(binary)?;
    let mut vec = Vec::new();

    let text_section = sections
        .get_section_by_name(".text")
        .map_or(Err("failed to find .text"), |t| Ok(t))?;

    let rdata_section = sections
        .get_section_by_name(".rdata")
        .map_or(Err("failed to find .rdata"), |t| Ok(t))?;

    let data_section = sections
        .get_section_by_name(".data")
        .map_or(Err("failed to find .data"), |t| Ok(t))?;

    let text_data = text_section.data;
    let rdata_data = rdata_section.data;
    let data_data = data_section.data;

    let text_rva = read_object!(text_section.header.virtual_address);
    let rdata_rva = read_object!(rdata_section.header.virtual_address);
    let data_rva = read_object!(data_section.header.virtual_address);

    let data_size = data_data.len();

    let strings = load_fvar_strings(
        &mut sections,
        &text_data,
        &rdata_data,
        text_rva,
        rdata_rva,
        data_rva,
    )?;

    let matches =
        scanner::scan::<IDAPat, &str>(&mut sections, &IDAPat::new(DYN_INIT_PAT), Some(".text"));

    for addr in matches {
        let mut stream = read_fvar_at_addr(&text_data, addr);

        stream.ignore_bytes(2);
        let fvar_type: FastVarType = stream.read()?;

        stream.ignore_bytes(3);
        let val_rva = calc_instruction_offset(&mut stream, addr, text_rva, data_rva)?;

        let fvar_name = {
            stream.ignore_bytes(3);

            read_cstyle_string(
                &rdata_data,
                calc_instruction_offset(&mut stream, addr, text_rva, rdata_rva)?,
            )
        };

        let fvar_val_type = {
            stream.ignore_bytes(1);

            let jmp_rva = calc_instruction_offset(&mut stream, addr, text_rva, text_rva)?;
            let mut jmp_data = NetworkStream::from(text_data[jmp_rva..jmp_rva + 0x3D].to_vec());

            let inst: u16 = jmp_data.read_be()?;
            let offset = if inst == 0x40_53 { 35 } else { 15 };

            jmp_data.ignore_bytes(offset);
            let sub_jmp_rva = calc_instruction_offset(&mut jmp_data, jmp_rva, text_rva, text_rva)?;

            let mut sub_jmp_stream =
                NetworkStream::from(text_data[sub_jmp_rva..sub_jmp_rva + 12].to_vec());

            sub_jmp_stream.ignore_bytes(8);
            sub_jmp_stream.read::<FastVarValueType>()?
        };

        let fvar_value = match fvar_val_type {
            FastVarValueType::Int => {
                if val_rva + 4 > data_size {
                    FastVarValue::Uninit
                } else {
                    let value =
                        u32::from_le_bytes((&data_data[val_rva..val_rva + 4]).try_into().unwrap());

                    FastVarValue::Int(value)
                }
            }

            FastVarValueType::Log => {
                if val_rva + 2 > data_size {
                    FastVarValue::Uninit
                } else {
                    let value =
                        u16::from_le_bytes((&data_data[val_rva..val_rva + 2]).try_into().unwrap());

                    FastVarValue::Log(value)
                }
            }

            FastVarValueType::Flag => {
                if val_rva > data_size {
                    FastVarValue::Uninit
                } else {
                    FastVarValue::Flag(data_data[val_rva] == 0x01)
                }
            }

            FastVarValueType::String => {
                if let Some(value) = strings.get(&val_rva) {
                    FastVarValue::String(value.clone())
                } else {
                    FastVarValue::Uninit
                }
            }

            _ => FastVarValue::Invalid,
        };

        vec.push(FastVar {
            name: fvar_name,
            value: fvar_value,
            value_type: fvar_val_type,
            var_type: fvar_type,
        });
    }

    Ok(vec)
}
