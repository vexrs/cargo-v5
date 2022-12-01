use std::{io::{Write, Read}};

use console::style;
use anyhow::Result;
use vexv5_serial::{Device, file::{FTInit, FTVID, FTTarget, FTFunction, FTOptions, FTExit, FTComplete, FTRead}};


pub fn download_file<S: Read+Write, U: Read+Write>(device: &mut Device<S, U>, file: String, on_exit: FTComplete) -> Result<Vec<u8>> {
    
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
        vexv5_serial::file::FTVID::User,
        vexv5_serial::file::FTOptions::NONE,
    ))?;

    // If the file size is larger than 16 KiB and twe are connected to a controller, then prompt the user if they want
    // to continue
    if device.is_controller()? && metadata.length > 16384 {
        let prompt = format!(
            "You are downloading a large ({}) file wirelessly. This is projected to take {} to complete. Are you sure you want to continue?",
            indicatif::HumanBytes(metadata.length as u64),
            indicatif::HumanDuration(std::time::Duration::from_secs(metadata.length as u64 / 1024)) // The average download speed at close range is ~1 KiB/s
        );
        if dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default()).with_prompt(prompt).interact()? {
            // Continue
        } else {
            // Abort
            return Err(anyhow::anyhow!("Aborted download due to user request"));
        }
    }

    // Inside of a file context handler
    let ctx = |ftm, device: &mut Device<S, U>| -> Result<Vec<u8>> {
        // Create a buffer to store data in
        let mut data = Vec::<u8>::new();

        // Get the max packet size
        let max_size: u16 = 512;

        // Get the size of the file
        let length = metadata.length;

        // Iterate over the file's size in chunks of max_size
        for i in (0..length).step_by(max_size.into()) {


            // Find the packet size that we want to read in
            let packet_size = if i + <u32>::from(max_size) > length {
                <u16>::try_from(length - i)?
            } else {
                max_size
            };

            // Find where to read the data from
            let read_location = i + metadata.addr;

            // Find how much data to read, padding it to 4 bytes
            let read_size = (packet_size + 3) & !3;

            // Read the data from the brain
            let read_data = device.send_request(FTRead(read_location, read_size))?;

            // Truncate to requested data (Ignore the integer sent in the first four bytes)
            let read_data = read_data[3..3 + read_size as usize][0..packet_size as usize].to_vec();

            data.extend(read_data);
        }

        // Return the data
        Ok(data)
    };

    // Open the file
    let ftm = device.send_request(FTInit {
        function: FTFunction::Download,
        target: FTTarget::Flash,
        vid: FTVID::User,
        options: FTOptions::NONE,
        file_type: metadata.file_type,
        length: metadata.length,
        addr: metadata.addr,
        crc: metadata.crc,
        timestamp: metadata.timestamp,
        version: metadata.version,
        name: file_name_bytes,
    })?;

    // Run the context handler
    let r = ctx(ftm, device);

    // Close the file
    device.send_request(FTExit(on_exit));

    // Return the context handler result
    r
}