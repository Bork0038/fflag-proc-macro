use crate::dump::FastVar;
use crate::stream::NetworkStream;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

fn open_cache(truncate: bool) -> Result<File, Box<dyn Error>> {
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(truncate)
        .open("target/version")?;

    Ok(file)
}
pub fn get_fflags_if_version_cached(
    version: &String,
) -> Result<Option<HashMap<String, FastVar>>, Box<dyn Error>> {
    let mut file = open_cache(false)?;

    let mut vec = Vec::new();
    file.read_to_end(&mut vec)?;

    if vec.len() == 0 {
        return Ok(None);
    }

    let mut stream = NetworkStream::from(vec);
    let cached_version = stream.read_string_le::<u8>()?;

    if &cached_version != version {
        return Ok(None);
    }

    let num_flags: u16 = stream.read_le()?;
    let mut map = HashMap::new();

    for _ in 0..num_flags {
        let flag: FastVar = stream.read()?;

        map.insert(flag.name.clone(), flag);
    }

    Ok(Some(map))
}

pub fn write_flags_to_cache(
    version: &String,
    flags: &mut HashMap<String, FastVar>,
) -> Result<(), Box<dyn Error>> {
    let mut stream = NetworkStream::new();

    stream.write_string_le::<u8>(version)?;
    stream.write_le::<u16>(flags.len() as u16);

    for flag in flags {
        let flag = flag.1;

        stream.write(flag)?;
    }

    let mut file = open_cache(true)?;
    file.write_all(&stream.data)?;

    Ok(())
}
