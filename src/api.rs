use serde::Deserialize;
use std::error::Error;
use std::io::{self, Cursor, Read};
use zip::ZipArchive;

#[derive(Deserialize)]
struct VersionData {
    #[serde(rename = "clientVersionUpload")]
    client_version_upload: String,
}

const VERSION_API: &str =
    "https://clientsettings.roblox.com/v2/client-version/WindowsStudio64/channel/LIVE";

pub fn get_latest_version<'a>() -> Result<String, Box<dyn Error>> {
    let res = attohttpc::get(VERSION_API).send()?;
    let data: VersionData = serde_json::from_str(&res.text()?)?;

    Ok(data.client_version_upload)
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
