use std::io::{Write, Read};

use console::style;
use anyhow::Result;
use vexv5_serial::Device;


pub fn download_file<S: Read+Write, U: Read+Write>(device: &mut Device<S, U>, file: String) -> Result<Vec<u8>> {
    
    // Convert the file name into a 24 byte long ASCII string
    let mut file_name_bytes: [u8; 24] = [0; 24];
    for (i, byte) in file.as_bytes().iter().enumerate() {
        if i + 1 > 24 {
            break;
        }
        file_name_bytes[i] = *byte as u8;
    }
    
    // Print that we are downloading a file
    println!("{} {}", style("Downloading File").bright(), style(file.clone()).cyan().bright());

    // Retrieve the file metadata
    let metadata = device.send_request(vexv5_serial::commands::GetFileMetadataByName(
        &file_name_bytes,
        vexv5_serial::file::FTVID::System,
        vexv5_serial::file::FTOptions::NONE,
    ));

    println!("{:?}", metadata);

    todo!();
}