use super::command_impl_prelude::*;

/// Select a clock mode
#[derive(Debug)]
pub struct ClockModeSelection {
    /// The clock mode to select
    pub mode: u8,
}

impl TransmitCommandData for ClockModeSelection {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x11,
            has_size_field: true,
            payload: vec![self.mode],
        }
    }
}

impl Receive for ClockModeSelection {
    type Response = ();

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorFirstByte(0x91),
        );

        reader
            .read_response()?
            .map(|_| ())
            .map_err(|error_code| match error_code {
                0x11 => CommandError::Checksum.into(),
                // From "RX210 Group User's Manual: Hardware", pg. 1416
                // (4) Clock Mode Selection
                //     ...
                //     Error (1 byte): Error code
                //                     ...
                //                     21h: Clock mode error
                //
                // That's wrong. It's 0x22 - which is (at least sort of
                // confirmed) by the table of error codes on pg. 1423.
                0x22 => CommandError::ClockMode.into(),
                _ => panic!("Unknown error code"),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> Result<()> {
        let cmd = ClockModeSelection { mode: 0xAB };
        let command_bytes = [0x11, 0x01, 0xAB, 0x43];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx_success() {
        let cmd = ClockModeSelection { mode: 0xAB };
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(()));
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = ClockModeSelection { mode: 0xAB };
        let response_bytes = [0x91, 0x22];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Err(CommandError::ClockMode.into()));
        assert!(is_script_complete(&mut p));
    }
}
