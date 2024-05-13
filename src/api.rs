use serde::Deserialize;
use std::error::Error;
use std::io::{Cursor, Read};
use std::collections::HashMap;
use zip::ZipArchive;

use crate::dump::{
    FastVar, 
    FastVarType, 
    FastVarValue, 
    FastVarValueType
};

#[derive(Deserialize)]
struct VersionData {
    #[serde(rename = "clientVersionUpload")]
    pub client_version_upload: String,
}

#[derive(Deserialize)]
struct ClientSettings {
    #[serde(rename = "applicationSettings")]
    pub application_settings: HashMap<String, String>
}

const VERSION_API: &str =
    "https://clientsettings.roblox.com/v2/client-version/WindowsStudio64/channel/LIVE";

const DYNAMIC_FLAG_API: &str =
    "https://clientsettingscdn.roblox.com/v2/settings/application/PCStudioApp";

    
pub fn get_latest_version<'a>() -> Result<String, Box<dyn Error>> {
    let res = attohttpc::get(VERSION_API).send()?;
    let data: VersionData = serde_json::from_str(&res.text()?)?;

    Ok(data.client_version_upload)
}

pub fn get_dynamic_flags() -> Result<HashMap<String, FastVar>, Box<dyn Error>> {
    let mut flags = HashMap::new();

    let client_settings: ClientSettings = {
        let res = attohttpc::get(DYNAMIC_FLAG_API).send()?;
        serde_json::from_str(&res.text()?)?
    };

    for (flag, value) in client_settings.application_settings {
        let mut words = Vec::new();
        let mut last_idx = 0;

        for (idx, char) in flag.char_indices().skip(1) {
            if char.is_uppercase() {
                words.push(&flag[last_idx..idx]);
                last_idx = idx;
            }
        }
        words.push(&flag[last_idx..]);
    
        if words[0] != "D" && words[0] != "F" { continue };

        let is_dynamic = words[0] == "D";

        let flag_type = words[if is_dynamic { 2 } else { 1 }];
        let flag_name = &words[if is_dynamic { 3 } else { 2 }..].join("");

        let value = value.split(";")
            .next()
            .map_or(
                Err("Failed to find fflag value"),
                | d | Ok(d)
            )?;

        let (flag_value_type, flag_value) = match flag_type {
            "String" => (
                FastVarValueType::String,
                FastVarValue::String(value.to_string())
            ),
            "Flag" => (
                FastVarValueType::Flag,
                FastVarValue::Flag(value == "True")
            ),
            "Int" => {
                if let Ok(int) = value.parse() {
                    (
                        FastVarValueType::Int,
                        FastVarValue::Int(int)
                    )
                } else {
                    (
                        FastVarValueType::Invalid,
                        FastVarValue::Uninit
                    )
                }
            },
            "Log" => (
                FastVarValueType::Log,
                FastVarValue::Log(value.to_string())
            ),

            _ => (
                FastVarValueType::Invalid,
                FastVarValue::Uninit
            ),
        };

        let flag = FastVar {
            name: flag_name.to_string(),
            var_type: FastVarType::Dynamic,
            value_type: flag_value_type,
            value: flag_value
        };

        flags.insert(flag_name.to_string(), flag);
    }

    Ok(flags)
}


fn unzip_binary(zip: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut vec = Vec::new();
    let mut archive = ZipArchive::new(Cursor::new(zip))?;

    let mut file = archive.by_name("RobloxStudioBeta.exe")?;
    file.read_to_end(&mut vec)?;

    Ok(vec)
}

pub fn get_binary(version: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let url = format!("http://setup.rbxcdn.com/{}-RobloxStudio.zip", version);
    let zip = attohttpc::get(url).send()?.bytes()?;

    Ok(unzip_binary(zip)?)
}
