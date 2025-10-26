use std::io::{Error, ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::result::Result;

#[derive(Debug)]
pub enum NetworkError {
    BindFailed,
    ConnectionFailed,
    ConnectionAborted,
    SendFailed,
    ReceiveFailed,
    CloseFailed,
    InvalidPacket,
}

impl From<NetworkError> for Error {
    fn from(error: NetworkError) -> Error {
        match error {
            NetworkError::BindFailed => Error::new(ErrorKind::Other, "Failed to bind to address"),
            NetworkError::ConnectionFailed => Error::new(ErrorKind::Other, "Connection failed"),
            NetworkError::ConnectionAborted => Error::new(ErrorKind::ConnectionAborted, "Connection aborted"),
            NetworkError::SendFailed => Error::new(ErrorKind::Other, "Send failed"),
            NetworkError::ReceiveFailed => Error::new(ErrorKind::Other, "Receive failed"),
            NetworkError::CloseFailed => Error::new(ErrorKind::Other, "Close failed"),
            NetworkError::InvalidPacket => Error::new(ErrorKind::Other, "Invalid packet")
        }
    }
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(port: &str) -> Result<Self, NetworkError> {
        let addr = "0.0.0.0:".to_string() + port;
        let listener = TcpListener::bind(addr).map_err(|_| NetworkError::BindFailed)?;
        Ok(Server { listener })
    }

    pub fn accept(&self) -> Result<Client, NetworkError> {
        let (stream, _) = self.listener.accept().map_err(|_| NetworkError::ConnectionFailed)?;
        Ok(Client::new(stream))
    }
}

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn send(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        match self.stream.write_all(data) {
            Ok(_) => Ok(()),
            Err(error) => match error.kind() {
                ErrorKind::ConnectionAborted => Err(NetworkError::ConnectionAborted),
                _ => Err(NetworkError::ReceiveFailed),
            }
        }
    }

    pub fn receive(&mut self, size: u32) -> Result<Vec<u8>, NetworkError> {
        let mut buffer = vec![0u8; size as usize]; // TODO best idea with sizing
        let n = match self.stream.read(&mut buffer) {
            Ok(value) => value,
            Err(error) => return match error.kind() {
                ErrorKind::ConnectionAborted => Err(NetworkError::ConnectionAborted),
                _ => Err(NetworkError::ReceiveFailed),
            }
        };
        buffer.truncate(n);
        Ok(buffer)
    }

    pub fn close(&mut self) -> Result<(), NetworkError> {
        self.stream.shutdown(Shutdown::Both).map_err(|_| NetworkError::CloseFailed)
    }
}