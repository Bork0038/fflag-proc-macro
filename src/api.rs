use serde::Deserialize;
use std::error::Error;
use std::collections::HashMap;


#[derive(Deserialize)]
pub struct ClientSettings {
    #[serde(rename = "applicationSettings")]
    pub application_settings: HashMap<String, String>
}

#[derive(Deserialize)]
struct VersionData {
    #[serde(rename = "clientVersionUpload")]
    client_version_upload: String,
}


const DYNAMIC_FLAG_API: &str =
    "https://clientsettingscdn.roblox.com/v2/settings/application/PCStudioApp";

const VERSION_API: &str =
    "https://clientsettings.roblox.com/v2/client-version/WindowsStudio64/channel/LIVE";



pub fn get_latest_version<'a>() -> Result<String, Box<dyn Error>> {
    let res = attohttpc::get(VERSION_API).send()?;
    let data: VersionData = serde_json::from_str(&res.text()?)?;

    Ok(data.client_version_upload)
}

pub fn get_dynamic_flags() -> Result<ClientSettings, Box<dyn Error>> {
    let res = attohttpc::get(DYNAMIC_FLAG_API).send()?;
    Ok(serde_json::from_str(&res.text()?)?)
}