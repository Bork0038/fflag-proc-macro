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

fn default_flags() -> HashMap<String, String> {
    HashMap::new()
}

#[derive(Deserialize)]
struct Input {
    #[serde(default = "default_version")]
    version: String,

    #[serde(default = "default_flags")]
    flags: HashMap<String, String>,

    #[serde(default = "default_flags")]
    dynamic_flags: HashMap<String, String>,
}

fn flag_to_token(
    flags: &mut HashMap<String, FastVar>, 
    name: &String,
    var_name: &String
) -> String {
    let flag = match flags.get(name) {
        Some(flag) => flag,
        None => panic!("Failed to find flag {} in binary", name),
    };

    match flag.value.clone() {
        FastVarValue::Invalid => panic!("Invalid FastVarValue"),
        FastVarValue::Uninit => panic!("FastVar {} not in initialized memory", flag.name),

        FastVarValue::Flag(flag) => format!("pub const {}: bool = {};", var_name, flag),
        FastVarValue::Int(int) => format!(
            "pub const {}: i32 = {};",
            var_name,
            Literal::i32_unsuffixed(int)
        ),
        FastVarValue::Log(log) =>
            format!("pub const {}: &str = {};", var_name, Literal::string(&log)),

        FastVarValue::String(str) =>
            format!("pub const {}: &str = {};", var_name, Literal::string(&str))
    }
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
    let mut flags = match cache::get_fflags_if_version_cached(&version)? {
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

    let mut dynamic_flags = api::get_dynamic_flags()?;

    let mut tokens = Vec::new();

    for (real_name, var_name) in input.flags {
        let token = flag_to_token(&mut flags, &real_name, &var_name); 
        tokens.push(token);
    }

    for (real_name, var_name) in input.dynamic_flags {
        let token = flag_to_token(&mut dynamic_flags, &real_name, &var_name);
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
