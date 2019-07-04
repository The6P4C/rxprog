use super::*;
use std::io;

struct SupportedDeviceInquiry {}

#[derive(Debug, PartialEq)]
struct SupportedDevice {
    device_code: u32,
    series_name: String,
}

#[derive(Debug, PartialEq)]
struct SupportedDeviceInquiryResponse {
    devices: Vec<SupportedDevice>,
}

impl Transmit for SupportedDeviceInquiry {
    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x20,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }

    fn tx<T: io::Write>(&self, p: &mut T) {
        p.write(&self.bytes());
        p.flush();
    }
}

impl Receive for SupportedDeviceInquiry {
    type Response = SupportedDeviceInquiryResponse;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        let mut b1 = [0u8; 1];
        p.read(&mut b1);
        let b1 = b1[0];

        assert_eq!(b1, 0x30);

        let mut _size = [0u8; 1];
        p.read(&mut _size);

        let mut device_count = [0u8; 1];
        p.read(&mut device_count);
        let device_count = device_count[0];

        let mut use_le = true;
        let mut devices: Vec<SupportedDevice> = vec![];
        for _ in 0..device_count {
            let mut character_count = [0u8; 1];
            p.read(&mut character_count);
            let character_count = character_count[0];

            let series_name_character_count = character_count - 4;

            let mut device_code_bytes = [0u8; 4];
            p.read(&mut device_code_bytes);

            let mut series_name_bytes = vec![0u8; series_name_character_count as usize];
            p.read(&mut series_name_bytes);

            let device_code = if use_le {
                u32::from_le_bytes(device_code_bytes)
            } else {
                u32::from_be_bytes(device_code_bytes)
            };

            use_le = !use_le;

            devices.push(SupportedDevice {
                device_code: device_code,
                series_name: str::from_utf8(&series_name_bytes)
                    .expect("Could not decode series name")
                    .to_string(),
            });
        }

        Ok(SupportedDeviceInquiryResponse { devices: devices })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() {
        let cmd = SupportedDeviceInquiry {};

        let bytes = cmd.bytes();

        assert_eq!(bytes, vec![0x20]);
    }

    #[test]
    fn test_rx() {
        let cmd = SupportedDeviceInquiry {};
        let response_bytes = vec![
            0x30, 0x13, 0x02, // Header
            0x08, 0x78, 0x56, 0x34, 0x12, 0x41, 0x42, 0x43, 0x44, // Device 1
            0x09, 0x89, 0xAB, 0xCD, 0xEF, 0x56, 0x57, 0x58, 0x59, 0x5A, // Device 2
        ];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(
            response,
            Ok(SupportedDeviceInquiryResponse {
                devices: vec![
                    SupportedDevice {
                        device_code: 0x12345678,
                        series_name: "ABCD".to_string(),
                    },
                    SupportedDevice {
                        device_code: 0x89ABCDEF,
                        series_name: "VWXYZ".to_string(),
                    },
                ],
            })
        );
        assert!(all_read(&mut p));
    }
}
