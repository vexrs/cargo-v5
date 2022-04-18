use std::{io::{Read, Write}, time::Duration};

use console::style;
use indicatif::ProgressBar;
use vexv5_serial::device::VexDevice;
use anyhow::Result;

use crate::util;

/// Writes data to a file on the V5 brain.
pub fn upload_file<T: Read + Write>(device: &mut VexDevice<T>, file_name: String, data: Vec<u8>) -> Result<()> {

    // Begin timer
    let time = std::time::SystemTime::now();

    println!("{} {}", style("Uploading File").bright(), style(file_name.clone()).cyan().bright());

    // Write to the slot_1.ini file on the brain
    let mut fh = device.open(file_name.clone(), Some(vexv5_serial::device::VexInitialFileMetadata {
        function: vexv5_serial::device::VexFileMode::Upload(vexv5_serial::device::VexFileTarget::FLASH, true),
        vid: vexv5_serial::device::VexVID::USER,
        options: 0,
        length: data.len() as u32,
        addr: 0x3800000,
        crc: crc::Crc::<u32>::new(&vexv5_serial::protocol::VEX_CRC32).checksum(&data),
        r#type: *b"bin\0",
        timestamp: 0,
        version: 0x01000000,
        linked_name: None,
    }))?;

    

    // Write data
    util::write_file_progress(&mut fh, data)?;
    
    // We are doing a file transfer, so it may take some time for the final response.
    // Just increase the timeout here
    device.set_timeout(Some(Duration::new(15, 0)));

    

    // We will also setup a spinner so the user knows that the application has not frozen.
    //let sp = Spinner::with_timer(Spinners::Dots, "Closing file handle".to_string());
    let sp = ProgressBar::new_spinner();
    sp.set_message("Closing file handle");
    sp.enable_steady_tick(100);

    // Close file
    fh.close(vexv5_serial::device::VexFiletransferFinished::ShowRunScreen)?;
    
    // And stop the spinner
    sp.finish_and_clear();
    println!("\x1b[F\x1b[32m✔\x1b[0m Finished closing file handle in {:.3} seconds", std::time::SystemTime::now().duration_since(time)?.as_secs_f32());


    // Reset the timeout to default
    device.set_timeout(None);


    // Log that the file has been successfully uploaded
    println!("\x1b[F\x1b[32m✔\x1b[0m {} {} {}", 
        style("Successfully uploaded file").bold(),
        style(file_name).cyan().bright(),
        style(format!("in {:.3} seconds", std::time::SystemTime::now().duration_since(time)?.as_secs_f32())).bold()
    );

    Ok(())
}


/// Reads a file from the V5 brain.
pub fn download_file<T: Read + Write>(device: &mut VexDevice<T>, file_name: String) -> Result<Vec<u8>> {

    // Begin timer
    let time = std::time::SystemTime::now();

    println!("{} {}", style("Downloading File").bright(), style(file_name.clone()).cyan().bright());

    // Retrieve the file metadata
    let metadata = device.file_metadata_from_name(file_name.clone(), None, None)?;


    // Write to the slot_1.ini file on the brain
    let mut fh = device.open(file_name.clone(), Some(vexv5_serial::device::VexInitialFileMetadata {
        function: vexv5_serial::device::VexFileMode::Download(vexv5_serial::device::VexFileTarget::FLASH, true),
        vid: num::FromPrimitive::from_u8(metadata.linked_vid).unwrap_or(vexv5_serial::device::VexVID::USER),
        options: 0,
        length: metadata.size,
        addr: metadata.addr,
        crc: 0,
        r#type: *b"bin\0",
        timestamp: 0,
        version: metadata.version,
        linked_name: Some(metadata.linked_filename),
    }))?;

    

    // Read data
    let data = util::read_file_progress(&mut fh)?;
    
    // We are doing a file transfer, so it may take some time for the final response.
    // Just increase the timeout here
    device.set_timeout(Some(Duration::new(15, 0)));


    // We will also setup a spinner so the user knows that the application has not frozen.
    //let sp = Spinner::with_timer(Spinners::Dots, "Closing file handle".to_string());
    let sp = ProgressBar::new_spinner();
    sp.set_message("Closing file handle");
    sp.enable_steady_tick(100);

    // Close file
    fh.close(vexv5_serial::device::VexFiletransferFinished::ShowRunScreen)?;
    
    // And stop the spinner
    sp.finish_and_clear();
    println!("\x1b[F\x1b[32m✔\x1b[0m Finished closing file handle in {:.3} seconds", std::time::SystemTime::now().duration_since(time)?.as_secs_f32());


    // Reset the timeout to default
    device.set_timeout(None);


    // Log that the file has been successfully uploaded
    println!("\x1b[F\x1b[32m✔\x1b[0m {} {} {}", 
        style("Successfully downloaded file").bold(),
        style(file_name).cyan().bright(),
        style(format!("in {:.3} seconds", std::time::SystemTime::now().duration_since(time)?.as_secs_f32())).bold()
    );

    Ok(data)
}