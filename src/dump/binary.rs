use object::{
    coff::CoffHeader,
    pe::{
        ImageDataDirectory, ImageDosHeader, ImageFileHeader, ImageNtHeaders64,
        ImageOptionalHeader64, ImageSectionHeader,
    },
    read::pe::{ImageNtHeaders, ImageOptionalHeader, ImportTable},
    LittleEndian,
};
use std::error::Error;

#[derive(Clone)]
pub struct Section {
    pub header: ImageSectionHeader,
    pub data: Vec<u8>,
}

impl Section {
    pub fn get_name(&self) -> String {
        String::from_utf8(self.header.name.to_vec())
            .map_or(String::new(), |s| String::from(s.trim_end_matches("\0")))
    }
}

pub struct Sections {
    pub data: Vec<Section>,
}

impl Sections {
    pub fn new() -> Self {
        Sections { data: Vec::new() }
    }

    pub fn get_section_by_name<S: Into<String>>(&mut self, name: S) -> Option<Section> {
        let name: String = name.into();

        for section in self.data.iter() {
            if section.get_name() == name {
                return Some(section.clone());
            }
        }

        None
    }
}

pub fn get_sections_from_binary(binary: Vec<u8>) -> Result<Sections, Box<dyn Error>> {
    let binary: &[u8] = binary.as_ref();
    let mut sections = Sections::new();

    let dos_header = *ImageDosHeader::parse(binary)?;
    let mut offset = dos_header.nt_headers_offset().into();

    let (nt_headers, _data_directories) = ImageNtHeaders64::parse(binary, &mut offset)?;
    let file_header = nt_headers.file_header();

    for section in file_header.sections(binary, offset)?.iter() {
        sections.data.push(Section {
            header: *section,
            data: section.pe_data(binary)?.to_vec(),
        })
    }

    Ok(sections)
}
