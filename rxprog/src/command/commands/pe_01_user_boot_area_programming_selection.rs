use super::*;
use std::io;

use super::reader::*;

#[derive(Debug)]
pub struct UserBootAreaProgrammingSelection {}

impl TransmitCommandData for UserBootAreaProgrammingSelection {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x42,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for UserBootAreaProgrammingSelection {
    type Response = ();
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader =
            ResponseReader::<_, SimpleResponse, NoError>::new(p, ResponseFirstByte::Byte(0x06));

        let _response = reader.read_response()?;

        Ok(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = UserBootAreaProgrammingSelection {};
        let command_bytes = [0x42];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = UserBootAreaProgrammingSelection {};
        let response_bytes = [0x06];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(()));
        assert!(all_read(&mut p));
    }
}