use anyhow::Result;
use serialport::SerialPortInfo;

pub mod protocol;
pub mod device;

/// Finds and returns all vex V5 ports.
pub fn discover_v5_ports() -> Result<Vec<SerialPortInfo>> {

    // Get all serial ports
    let ports = serialport::available_ports()?;

    // Find all V5 devices
    let ports: Vec<SerialPortInfo> = ports.into_iter().filter(|port| {
        let info = match port.port_type.clone() {
            serialport::SerialPortType::UsbPort(i) => i,
            _ => {
                return false;
            }
        };
        
        // The VEX vendor ids are 0x2888 and 0x0501
        if info.vid == 0x2888 || info.vid == 0x0501 {
            return true;
        }

        // Get the product name
        let product_name = match info.product {
            Some(name) => name,
            None => "".to_string(),
        };
        // If the name has VEX or V5 in it, it is most likely a V5
        if product_name.contains("VEX") || product_name.contains("V5") {
            return true;
        }

        

        false
    }).collect();

    Ok(ports)
}

