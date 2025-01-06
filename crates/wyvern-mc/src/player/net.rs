use std::{io::{ErrorKind, Read}, net::{IpAddr, TcpStream}};

#[derive(Debug)]
pub struct RawConnection {
    pub(crate) stream: TcpStream,
    #[allow(unused)]
    pub(crate) addr: IpAddr,
    pub(crate) removed: bool
}

impl RawConnection {
    pub fn event_loop(&mut self) {
        self.handle_incoming_packets();
        self.write_outgoing_packets();
    }

    pub fn handle_incoming_packets(&mut self) {
        let mut buf = [0; 256];
        let bytes_read = self.stream.read(&mut buf);
        match bytes_read {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    self.removed = true;
                    return;
                }
                println!("recv: {:?}", &buf[0..bytes_read])
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {

            }
            Err(_e) => {

            }
        }
    }

    pub fn write_outgoing_packets(&mut self) {

    }
}