use std::io::{Error, ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::result::Result;

#[derive(Debug)]
pub enum NetwError {
    BindFailed,
    ConnectionFailed,
    SendFailed,
    ReceiveFailed,
    CloseFailed,
    InvalidPacket,
}

impl From<NetwError> for Error {
    fn from(error: NetwError) -> Error {
        match error {
            NetwError::BindFailed => Error::new(ErrorKind::Other, "Failed to bind to address"),
            NetwError::ConnectionFailed => Error::new(ErrorKind::Other, "Connection failed"),
            NetwError::SendFailed => Error::new(ErrorKind::Other, "Send failed"),
            NetwError::ReceiveFailed => Error::new(ErrorKind::Other, "Receive failed"),
            NetwError::CloseFailed => Error::new(ErrorKind::Other, "Close failed"),
            NetwError::InvalidPacket => Error::new(ErrorKind::Other, "Invalid packet")
        }
    }
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(port: &str) -> Result<Self, NetwError> {
        let addr = "0.0.0.0:".to_string() + port;
        let listener = TcpListener::bind(addr).map_err(|_| NetwError::BindFailed)?;
        Ok(Server { listener })
    }

    pub fn accept(&self) -> Result<Client, NetwError> {
        let (stream, _) = self.listener.accept().map_err(|_| NetwError::ConnectionFailed)?;
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

    pub fn send(&mut self, data: &[u8]) -> Result<(), NetwError> {
        self.stream.write_all(data).map_err(|_| NetwError::SendFailed)
    }

    pub fn receive(&mut self, size: u32) -> Result<Vec<u8>, NetwError> {
        let mut buffer = vec![0u8; size as usize];
        let n = self.stream.read(&mut buffer).map_err(|_| NetwError::ReceiveFailed)?;
        buffer.truncate(n);
        Ok(buffer)
    }

    pub fn close(&mut self) -> Result<(), NetwError> {
        self.stream.shutdown(Shutdown::Both).map_err(|_| NetwError::CloseFailed)
    }
}