extern crate proc_macro;

mod api;
mod cache;
mod dump;
mod stream;

use dump::{FastVar, FastVarValue};
use proc_macro::{Literal, TokenStream};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

fn default_version() -> String {
    "latest".into()
}

#[derive(Deserialize)]
struct Input {
    #[serde(default = "default_version")]
    version: String,

    flags: HashMap<String, String>,
}

fn include_fflags_internal(item: TokenStream) -> Result<TokenStream, Box<dyn Error>> {
    let str = format!("{{{}}}", item.to_string());
    let input: Input = serde_json::from_str(str.as_str())?;

    let version = if input.version == "latest" {
        api::get_latest_version()?
    } else {
        input.version
    };

    // read fflags if cached
    let flags = match cache::get_fflags_if_version_cached(&version)? {
        Some(flags) => flags,
        None => {
            let binary = api::get_binary(version.clone())?;
            let mut flags: HashMap<String, FastVar> = dump::get_fflags(binary)?
                .iter()
                .map(|flag| (flag.name.clone(), flag.clone()))
                .collect();

            cache::write_flags_to_cache(&version, &mut flags)?;

            flags
        }
    };

    let mut tokens = Vec::new();
    for (real_name, var_name) in input.flags {
        let flag = match flags.get(&real_name) {
            Some(flag) => flag,
            None => panic!("Failed to find flag {} in binary", real_name),
        };

        let token = match flag.value.clone() {
            FastVarValue::Invalid => panic!("Invalid FastVarValue"),
            FastVarValue::Uninit => panic!("FastVar {} not in initialized memory", flag.name),

            FastVarValue::Flag(flag) => format!("const {}: bool = {};", var_name, flag),
            FastVarValue::Int(int) => format!(
                "const {}: u32 = {};",
                var_name,
                Literal::u32_unsuffixed(int)
            ),
            FastVarValue::Log(log) => format!(
                "const {}: u16 = {};",
                var_name,
                Literal::u16_unsuffixed(log)
            ),
            FastVarValue::String(str) => {
                format!("const {}: &str = {};", var_name, Literal::string(&str))
            }
        };

        tokens.push(token);
    }

    Ok(tokens.join("\n").parse()?)
}

#[proc_macro]
pub fn include_fflags(item: TokenStream) -> TokenStream {
    match include_fflags_internal(item) {
        Ok(stream) => stream,
        Err(e) => panic!("{}", e),
    }
}
