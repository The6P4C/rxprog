use std::io;

use super::command::*;
use super::data::MemoryArea;
use super::reader::*;

/// Reads a number of bytes from a specified memory location
#[derive(Debug)]
pub struct MemoryRead {
    /// The memory area to read from
    pub area: MemoryArea,
    /// Address of the first byte to read
    pub start_address: u32,
    /// Number of bytes to read
    pub size: u32,
}

impl TransmitCommandData for MemoryRead {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x52,
            has_size_field: true,
            payload: {
                let mut payload = vec![];
                payload.push(match self.area {
                    MemoryArea::UserBootArea => 0x00,
                    MemoryArea::UserArea => 0x01,
                });
                payload.extend(&self.start_address.to_be_bytes());
                payload.extend(&self.size.to_be_bytes());
                payload
            },
        }
    }
}

/// Error preventing a successful memory read
#[derive(Debug, PartialEq)]
pub enum MemoryReadError {
    /// Command checksum validation failed
    Checksum,
    /// Invalid address (not in selected area, or invalid area)
    Address,
    /// Invalid data size (zero, too large, or calculated end out of bounds)
    DataSize,
}

impl Receive for MemoryRead {
    type Response = Vec<u8>;
    type Error = MemoryReadError;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader = ResponseReader::<_, SizedResponse<u32>, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x52),
            ErrorFirstByte(0xD2),
        );

        let response = reader.read_response()?;

        Ok(match response {
            Ok(SizedResponse { data, .. }) => Ok(data),
            Err(error_code) => Err(match error_code {
                0x11 => MemoryReadError::Checksum,
                0x2A => MemoryReadError::Address,
                0x2B => MemoryReadError::DataSize,
                _ => panic!("Unknown error code"),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = MemoryRead {
            area: MemoryArea::UserArea,
            start_address: 0x12345678,
            size: 0x0A,
        };
        let command_bytes = [
            0x52, 0x09, 0x01, 0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x0A, 0x86,
        ];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx_success() {
        let cmd = MemoryRead {
            area: MemoryArea::UserArea,
            start_address: 0x12345678,
            size: 0x0A,
        };
        let response_bytes = [
            0x52, 0x00, 0x00, 0x00, 0x0A, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
            0x0A, 0x6D,
        ];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(vec![
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A
            ])
        );
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = MemoryRead {
            area: MemoryArea::UserArea,
            start_address: 0x12345678,
            size: 0x10,
        };
        let response_bytes = [0xD2, 0x2A];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Err(MemoryReadError::Address));
        assert!(is_script_complete(&mut p));
    }
}
