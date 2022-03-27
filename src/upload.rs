use vex_v5_serial::v5::device::{VexV5Device};
use vex_v5_serial::v5::protocol::VexFiletransferFinished;
use std::path::Path;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::io::{Read, Write};
use std::time::SystemTime;
use chrono::prelude::{DateTime, Utc};
use ascii::{AsAsciiStr};

#[derive(Serialize, Deserialize, Debug)]
struct Package {
    name: String,
    version: String,
    description: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct CargoToml {
    package: Package,
}



pub fn upload<T: Read + Write>(mut device: VexV5Device<T>, slot: u8, run: bool) -> Result<()> {
    // Try to find a Cargo.toml in the current directory
    let cargo = Path::new("./Cargo.toml");

    // If we can't find it, then we can't upload
    if !cargo.exists() {
        return Err(anyhow::anyhow!("Could not find Cargo.toml in the current directory"));
    }

    // Parse the toml file
    let f = std::fs::read_to_string(cargo)?;
    let parsed_toml = toml::from_str::<CargoToml>(&f)?;
    


    // Get the environment variable for the upload file
    let upload_file = std::env::var("VEX_UPLOAD_FILE");

    // If it is none, we can not upload
    if upload_file.is_err() {
        return Err(anyhow::anyhow!("Could not find VEX_UPLOAD_FILE environment variable"));
    }

    let upload_file = upload_file.unwrap();

    // Get the time
    let time = SystemTime::now();

    // Format it as ISO 8601
    let time_fmt = <DateTime<Utc>>::from(time).format("%+");

    // Create the ini file
    let mut ini = Vec::<String>::new();
    ini.push(format!("[program]"));
    ini.push(format!("name = \"{}\"", parsed_toml.package.name));
    ini.push(format!("version = \"{}\"", parsed_toml.package.version));
    ini.push(format!("description = \"{}\"", parsed_toml.package.description.unwrap_or("".to_string())));
    ini.push(format!("slot = {}", slot - 1));
    ini.push(format!("date = {}", time_fmt));

    // Lets use the vex X logo for now
    ini.push("icon = USER001x.bmp".to_string());

    // Join into a single string
    let ini = ini.join("\n");
    
    // Convert to an ascii string
    let ini = ini.as_ascii_str()?;
    // Convert to bytes
    let ini: Vec<u8> = ini.as_bytes().to_vec();

    // Upload the file
    device.upload_file(
        format!("slot_{}.ini", slot), 
        ini, 
        VexFiletransferFinished::DoNothing
    )?;
    
    // Open the file and read it's contents into a vec
    let mut file = std::fs::File::open(upload_file.clone())?;
    let mut contents = Vec::<u8>::new();
    file.read_to_end(&mut contents)?;
    
    // Get the upload_file's filename
    let filename = Path::new(&upload_file.clone()).file_name().unwrap().to_string_lossy().to_string();

    // Upload the file
    device.upload_file(
        filename, 
        contents, 
        if run {
            VexFiletransferFinished::RunProgram
        } else {
            VexFiletransferFinished::DoNothing
        }
    )?;

    Ok(())
}