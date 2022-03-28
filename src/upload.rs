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



pub fn upload<T: Read + Write>(mut device: VexV5Device<T>, slot: u8, run: bool, upload_file: String) -> Result<()> {
    
    // Get the .v5 directory
    let v5_dir = dirs::home_dir().unwrap().join(".v5");

    // If it does not exist then error
    if !v5_dir.exists() {
        return Err(anyhow::anyhow!("No HOME/.v5 directory found. Please re-install cargo-v5 to fix this."));
    }
    
    // If the upload_file does not have an extension .bin then objcopy it
    // into a binary file
    let upload_file = if !upload_file.ends_with(".bin") {
        // If this is a windows system, then use the objcopy that comes
        // with vexcode.
        // If it is a unix system, then use the objcopy in the path.
        let command = if cfg!(windows) {
            let dir = v5_dir.join("libv5rt/toolchain/vexv5/win32/gcc/bin/arm-none-eabi-objcopy.exe");
            let str = dir.to_str();
            let str = str.unwrap_or("arm-none-eabi-objcoppy");
            str.to_string()
        } else {
           "arm-none-eabi-objcopy".to_string()
        };
        
        // Create the objcopy command
        let mut command = std::process::Command::new(command);
        command.arg("-O").arg("binary");

        // Add the upload file path
        command.arg(upload_file.clone());

        // Set the output file to the upload_file with the .bin extension
        let mut output_file = upload_file;
        output_file.push_str(".bin");
        command.arg(output_file.clone());

        // Run the command
        command.output()?;

        output_file
    } else {
        upload_file
    };
    
    
    // Try to find a Cargo.toml in the current directory
    let cargo = Path::new("./Cargo.toml");

    // If we can't find it, then we can't upload
    if !cargo.exists() {
        return Err(anyhow::anyhow!("Could not find Cargo.toml in the current directory"));
    }

    // Parse the toml file
    let f = std::fs::read_to_string(cargo)?;
    let parsed_toml = toml::from_str::<CargoToml>(&f)?;
    

    // Get the time
    let time = SystemTime::now();

    // Format it as ISO 8601
    let time_fmt = <DateTime<Utc>>::from(time).format("%+");

    // Create the ini file
    let mut ini = Vec::<String>::new();
    ini.push("[program]".to_string());
    ini.push(format!("name = \"{}\"", parsed_toml.package.name));
    ini.push(format!("version = \"{}\"", parsed_toml.package.version));
    ini.push(format!("description = \"{}\"", parsed_toml.package.description.unwrap_or_else(||{"".to_string()})));
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
    let filename = Path::new(&upload_file).file_name().unwrap().to_string_lossy().to_string();

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