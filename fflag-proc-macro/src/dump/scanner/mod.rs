mod pattern;
pub use pattern::{CodePat, IDAPat, Pattern};

use super::binary::Sections;

fn scan_single_section<P: Pattern, S: Into<String>>(
    sections: &mut Sections,
    pattern: &P,
    section: S,
) -> Vec<usize> {
    let mut out = Vec::new();

    if let Some(section) = sections.get_section_by_name(section) {
        let data = &section.data;
        let len = data.len();

        for i in 0..len {
            let mut found = true;

            for j in 0..pattern.get_len() {
                if i + j >= len {
                    found = false;
                    break;
                }

                if !pattern.scan(data[i + j], j) {
                    found = false;
                    break;
                }
            }

            if found {
                out.push(i as usize);
            }
        }
    }

    out
}

fn scan_all_sections<P: Pattern>(sections: &mut Sections, pattern: &P) -> Vec<usize> {
    let mut out = Vec::new();

    for section in sections.data.clone() {
        let res = scan_single_section(sections, pattern, &section.get_name())
            .iter()
            .map(|d| d + section.header.pointer_to_raw_data.get(object::LittleEndian) as usize)
            .collect::<Vec<usize>>();

        out.extend_from_slice(&res);
    }

    out
}

pub fn scan<P: Pattern, S: Into<String>>(
    sections: &mut Sections,
    pattern: &P,
    section: Option<S>,
) -> Vec<usize> {
    match section {
        Some(section) => scan_single_section(sections, pattern, section),
        None => scan_all_sections(sections, pattern),
    }
}
