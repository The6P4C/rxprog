use super::command_impl_prelude::*;

/// Requests the number of bytes in each programming unit
#[derive(Debug)]
pub struct ProgrammingSizeInquiry {}

impl TransmitCommandData for ProgrammingSizeInquiry {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x27,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for ProgrammingSizeInquiry {
    type Response = u16;

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u8>, NoError>::new(p, ResponseFirstByte::Byte(0x37));

        let data = reader.read_response()?.data;

        let mut programming_size_bytes = [0u8; 2];
        programming_size_bytes.copy_from_slice(&data);

        let programming_size = u16::from_be_bytes(programming_size_bytes);

        Ok(programming_size)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> Result<()> {
        let cmd = ProgrammingSizeInquiry {};
        let command_bytes = [0x27];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = ProgrammingSizeInquiry {};
        let response_bytes = [0x37, 0x02, 0x12, 0x34, 0x81];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(0x1234));
        assert!(is_script_complete(&mut p));
    }
}
