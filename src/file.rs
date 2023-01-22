use std::{io::{Write, Read}};

use console::style;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use vexv5_serial::{devices::device::Device, file::{FTInit, FTVID, FTTarget, FTFunction, FTOptions, FTExit, FTComplete, FTRead, FTWrite, FTInitResponse, FTType}};

use crate::device_commands::with_channel;


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

    

    // If the file size is larger than 16 KiB and we are connected to a controller, then prompt the user if they want
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

        // Create the progress bar
        let bar = ProgressBar::new(metadata.length.into());
        
        // Style the progress bar
        bar.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {binary_bytes_per_sec} {bar:40.cyan/blue} {percent}% {bytes:>7}/{total_bytes:7} {msg}")?
            .progress_chars("##-"));

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

            bar.inc(packet_size.into());
        }

        // Finalize the progress bar
        bar.finish_and_clear();

        // Return the data
        Ok(data)
    };

    // Begin timer
    let time = std::time::SystemTime::now();
    
    // Use a download channel
    let d = with_channel(device, vexv5_serial::remote::V5ControllerChannel::Download, |device| {
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
        device.send_request(FTExit(on_exit))?;

        // Return the context handler result
        r
    });

    match d {
        Ok(v) => {
            println!("\x1b[F\x1b[32m✔\x1b[0m {} {} {}", 
                style("Successfully downloaded file").bold(),
                style(file).cyan().bright(),
                style(format!("in {:.3} seconds", std::time::SystemTime::now().duration_since(time)?.as_secs_f32())).bold()
            );
            Ok(v)
        },
        Err(e) => {
            println!("\x1b[F\x1b[31m❌\x1b[0m {} {} {}", 
                style("Failed to download file").red().bold(),
                style(file).cyan().bright(),
                style(format!("in {:.3} seconds", std::time::SystemTime::now().duration_since(time)?.as_secs_f32())).bold()
            );
            Err(e)
        }
    }
}


pub fn upload_file<S: Read+Write, U: Read+Write>(device: &mut Device<S, U>, file: String, data: Vec<u8>, on_exit: FTComplete) -> Result<Vec<u8>> {
    
    // Convert the file name into a 24 byte long ASCII string
    let mut file_name_bytes: [u8; 24] = [0; 24];
    for (i, byte) in file.as_bytes().iter().enumerate() {
        if i + 1 > 24 {
            break;
        }
        file_name_bytes[i] = *byte as u8;
    }
    
    println!("{} {}", style("Uploading File").bright(), style(file.clone()).cyan().bright());
    
    // If the file size is larger than 16 KiB and we are connected to a controller, then prompt the user if they want
    // to continue
    if device.is_controller()? && data.len() > 16384 {
        let prompt = format!(
            "You are uploading a large ({}) file wirelessly. This is projected to take {} to complete. Are you sure you want to continue?",
            indicatif::HumanBytes(data.len() as u64),
            indicatif::HumanDuration(std::time::Duration::from_secs(data.len() as u64 / 1024)) // The average download speed at close range is ~1 KiB/s
        );
        if dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default()).with_prompt(prompt).interact()? {
            // Continue
        } else {
            // Abort
            return Err(anyhow::anyhow!("Aborted download due to user request"));
        }
    }

    

    // Retrieve the file metadata
    let metadata = device.send_request(vexv5_serial::commands::GetFileMetadataByName(
        &file_name_bytes,
        vexv5_serial::file::FTVID::User,
        vexv5_serial::file::FTOptions::NONE,
    ));

    // If the file does not exist, then set metadata to None
    let metadata = match metadata {
        Ok(v) => Some(v),
        Err(e) => None,
    };

    
    
    // Begin timer
    let time = std::time::SystemTime::now();

    // Create the vex CRC
    let v5crc = crc::Crc::<u32>::new(&vexv5_serial::VEX_CRC32);
    
    
    // Open the file
    let ftm = device.send_request(FTInit {
        function: FTFunction::Upload,
        target: FTTarget::Flash,
        vid: FTVID::User,
        options: FTOptions::OVERWRITE,
        file_type: FTType::Ini, // Default to ini because it works for some reason
        length: data.len() as u32,
        addr: match metadata {
            Some(v) => v.addr,
            None => 0
        },
        crc: v5crc.checksum(&data),
        timestamp: 0, // Just set to 0 for now
        version: 1,
        name: file_name_bytes,
    })?;
    println!("ok");
    // Inside of a file context handler
    let ctx = |ftm: FTInitResponse, device: &mut Device<S, U>| -> Result<Vec<u8>> {

        // Create the progress bar
        let bar = ProgressBar::new(ftm.file_size.into());
        
        // Style the progress bar
        bar.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {binary_bytes_per_sec} {bar:40.cyan/blue} {percent}% {bytes:>7}/{total_bytes:7} {msg}")?
            .progress_chars("##-"));

        // Save the max size so it is easier to access
        // We want it to be 3/4 size so we do not have issues with packet headers
        // going over the max size
        let max_size = ftm.max_packet_size / 
            2 + (ftm.max_packet_size / 4);
        
        // We will be using the length of the file in the metadata
        // that way we do not ever write more data than is expected.
        // However, if the vector is smaller than the file size
        // Then use the vector size.
        let size = if data.len() as u32 > ftm.file_size {
            ftm.file_size
        } else {
            data.len() as u32
        };



        // We will be incrementing this variable so we know how much we have written
        let mut how_much: usize = 0;

        // Iterate over the file's size in chunks of max_size
        for i in (0..size).step_by(max_size.into()) {
            // Determine the packet size. We do not want to write
            // max_size bytes if we are at the end of the file
            let packet_size = if size < max_size as u32 {
                size as u16
            } else if i as u32 + max_size as u32 > size {
                (size - i as u32) as u16
            } else {
                max_size
            };
        
            // Cut out packet_size bytes out of the provided buffer
            let payload = &data[i as usize..i as usize + packet_size as usize];
            
            // Get the addr
            let addr = match metadata {
                Some(v) => v.addr,
                None => 0x0,
            };

            // Write the payload to the file
            device.send_request(FTWrite(i + addr, payload))?;
        
            // Update the progress bar
            bar.inc(packet_size.into());
        
            // Increment how_much by packet data so we know how much we
            // have written to the file
            how_much += packet_size as usize;
        }

        // Finalize the progress bar
        bar.finish_and_clear();

        // Return the data
        Ok(data)
    };

    
    // Run the context handler
    let r = ctx(ftm, device);
    
    // Close the file
    device.send_request(FTExit(on_exit))?;
    
    // Return the context handler result
    match r {
        Ok(v) => {
            println!("\x1b[F\x1b[32m✔\x1b[0m {} {} {}", 
                style("Successfully uploaded file").bold(),
                style(file).cyan().bright(),
                style(format!("in {:.3} seconds", std::time::SystemTime::now().duration_since(time)?.as_secs_f32())).bold()
            );
            Ok(v)
        },
        Err(e) => {
            println!("\x1b[F\x1b[31m❌\x1b[0m {} {} {}", 
                style("Failed to upload file").red().bold(),
                style(file).cyan().bright(),
                style(format!("in {:.3} seconds", std::time::SystemTime::now().duration_since(time)?.as_secs_f32())).bold()
            );
            Err(e)
        }
    }
}