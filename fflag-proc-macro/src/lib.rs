extern crate proc_macro;

mod api;
mod cache;
mod dump;
mod stream;

use dump::{FastVar, FastVarValue, FastVarValueType};
use proc_macro::TokenStream;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use quote::quote;
use syn::{Type, Ident};

// static fflag proc macro
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



    let mut tokens = quote! {};
    for (real_name, var_name) in input.flags {
        let flag = match flags.get(&real_name) {
            Some(flag) => flag,
            None => panic!("Failed to find flag {} in binary", real_name),
        };

        let var_name = Ident::new(&var_name, proc_macro2::Span::call_site());
        let token = match flag.value.clone() {
            FastVarValue::Invalid => panic!("Invalid FastVarValue"),
            FastVarValue::Uninit => panic!("Dynamic FastVar {} must be loaded via runtime macro", flag.name),

            FastVarValue::Flag(flag) => quote! { 
                pub const #var_name: bool = #flag;
            },
            FastVarValue::Int(int) => quote! {
                pub const #var_name: i32 = #int;
            },
            FastVarValue::Log(log) => quote! {
                pub const #var_name: u16 = #log;
            },
            FastVarValue::String(str) => quote! {
                pub const #var_name: &str = #str;
            }
        };

        tokens.extend(token);
    }

    Ok(TokenStream::from(tokens))
}

#[proc_macro]
pub fn include_fflags(item: TokenStream) -> TokenStream {
    match include_fflags_internal(item) {
        Ok(stream) => stream,
        Err(e) => panic!("{}", e),
    }
}


// type loader for runtime fflag loader
fn get_type_str_for_fast_var_value_type<'a>(value_type: FastVarValueType) -> &'a str {
    match value_type {
        FastVarValueType::Flag => "bool",
        FastVarValueType::Int => "i32",
        FastVarValueType::Log => "u16",
        FastVarValueType::String => "&str",

        FastVarValueType::Uninit => "",
        FastVarValueType::Invalid => ""
    }
}


fn generate_base_flag_for_type(
    token_type: Type, 
    flag: &FastVar,
    real_name: String, 
    var_name: String
) -> proc_macro2::TokenStream {
    let type_name = get_type_str_for_fast_var_value_type(flag.value_type);
    let token_name = Ident::new(&var_name, proc_macro2::Span::call_site());
    let token_prefix = flag.get_full_name();

    quote! {
        pub static ref #token_name: #token_type = {
            let flag = match FLAGS_INTERNAL_DO_NOT_USE.application_settings.get(#token_prefix) {
                Some(flag) => flag,
                None => match FLAGS_INTERNAL_DO_NOT_USE.application_settings.get(#real_name) {
                    Some(flag) => flag,
                    None => panic!("Failed to find FFlag {} from application settings", #real_name)
                }
            };

            match flag.parse::<#token_type>() {
                Ok(flag) => flag,
                Err(e) => panic!("Expected {} for Flag {} but got: {}", #real_name, #type_name, flag)
            }
        };
    }
}

fn include_fflags_runtime_internal(item: TokenStream) -> Result<TokenStream, Box<dyn Error>> {
    let str = format!("{{{}}}", item.to_string());
    let map: HashMap<String, String> = serde_json::from_str(str.as_str())?;

    // read fflags if cached
    let version = api::get_latest_version()?;
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

    let mut tokens = quote! {
        static ref FLAGS_INTERNAL_DO_NOT_USE: api::ClientSettings = {
            match api::get_dynamic_flags() {
                Ok(flags) => flags,
                Err(e) => panic!("Failed to load fflags from api: {}", e)
            } 
        };
    };

    for (real_name, var_name) in map {
        if real_name == "" { continue };

        let flag = match flags.get(&real_name) {
            Some(flag) => flag,
            None => panic!("FFlag {} not found in binary", real_name)
        };

        let token_type = match syn::parse_str(get_type_str_for_fast_var_value_type(flag.value_type)) {
            Ok(t) => t,
            Err(_) => panic!("Failed to parse syn token")
        };

        let token_name = Ident::new(&var_name, proc_macro2::Span::call_site());
        let token_prefix = flag.get_full_name();
        let token = if flag.value_type == FastVarValueType::String {
            quote! {
                pub static ref #token_name: &str = {
                    match FLAGS_INTERNAL_DO_NOT_USE.application_settings.get(&#token_prefix) {
                        Some(flag) => flag,
                        None => match FLAGS_INTERNAL_DO_NOT_USE.application_settings.get(&#real_name) {
                            Some(flag) => flag,
                            None => panic!("Failed to find FFlag {} from application settings", #real_name)
                        }
                    }.as_str()
                };
            }
        } else if flag.value_type != FastVarValueType::Invalid && flag.value_type != FastVarValueType::Uninit {
            generate_base_flag_for_type(token_type, &flag, real_name, var_name)
        } else {
            quote! {}
        };
        
        tokens.extend(token);
    }
   
    let code = quote! {
        use fflag_proc_macro::{api, lazy_static};
        use lazy_static::lazy_static;

        lazy_static! {
            #tokens
        }
    };

    Ok(TokenStream::from(code))
}

#[proc_macro]
pub fn include_fflags_runtime(item: TokenStream) -> TokenStream {
    match include_fflags_runtime_internal(item) {
        Ok(stream) => stream,
        Err(e) => panic!("{}", e),
    }
}