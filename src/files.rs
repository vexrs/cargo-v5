use std::{io::{Read, Write}, time::Duration};

use console::style;
use vexv5_serial::device::VexDevice;
use anyhow::Result;
use spinners::{Spinner, Spinners};

use crate::util;


pub fn upload_file<T: Read + Write>(device: &mut VexDevice<T>, file_name: String, data: Vec<u8>) -> Result<()> {

    // Write to the slot_1.ini file on the brain
    let mut fh = device.open(file_name.to_string(), Some(vexv5_serial::device::VexInitialFileMetadata {
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
    let sp = Spinner::with_timer(Spinners::Dots, "Closing File Handle".to_string());

    // Close file
    fh.close(vexv5_serial::device::VexFiletransferFinished::ShowRunScreen)?;
    
    // And stop the spinner
    sp.stop_with_symbol("\x1b[32m✔\x1b[0m");
    


    // Reset the timeout to default
    device.set_timeout(None);


    Ok(())
}