use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::result::Result;
// use std::time::Duration;

#[derive(Debug)]
pub enum NetworkError {
    BindFailed,
    ConnectionFailed,
    ConnectionAborted,
    SendFailed,
    NotAllDataSent,
    ReceiveFailed,
    CloseFailed,
    InvalidPacket,
    SetTimeoutFailed,
    CloneFailed,
}

impl From<NetworkError> for Error {
    fn from(error: NetworkError) -> Error {
        match error {
            NetworkError::BindFailed => Error::new(ErrorKind::Other, "Failed to bind to address"),
            NetworkError::ConnectionFailed => Error::new(ErrorKind::Other, "Connection failed"),
            NetworkError::ConnectionAborted => Error::new(ErrorKind::ConnectionAborted, "Connection aborted"),
            NetworkError::SendFailed => Error::new(ErrorKind::Other, "Send failed"),
            NetworkError::NotAllDataSent => Error::new(ErrorKind::Other, "Not all data sent"),
            NetworkError::ReceiveFailed => Error::new(ErrorKind::Other, "Receive failed"),
            NetworkError::CloseFailed => Error::new(ErrorKind::Other, "Close failed"),
            NetworkError::InvalidPacket => Error::new(ErrorKind::Other, "Invalid packet"),
            NetworkError::SetTimeoutFailed => Error::new(ErrorKind::Other, "Set timeout failed"),
            NetworkError::CloneFailed => Error::new(ErrorKind::Other, "Clone failed"),
        }
    }
}

pub struct Server {
    socket: UdpSocket,
}

impl Server {
    pub fn new(port: &str) -> Result<Self, NetworkError> {
        let addr = "0.0.0.0:".to_string() + port;
        let socket = UdpSocket::bind(addr).map_err(|_| NetworkError::BindFailed)?;
        // socket.set_read_timeout(Some(Duration::from_secs(5))).map_err(|_| NetworkError::SetTimeoutFailed)?;
        Ok(Server { socket })
    }

    pub fn accept(&self) -> Result<Client, NetworkError> {
        let mut buf = [0u8; 1];
        let (_, addr) = self.socket.recv_from(&mut buf).map_err(|_| NetworkError::ReceiveFailed)?;
        let client = Client::new(self.socket.try_clone().
            map_err(|_| NetworkError::CloneFailed)?, addr)?;
        Ok(client)
    }

}

pub struct Client {
    socket: UdpSocket,
    peer_addr: SocketAddr,
}

impl Client {
    fn new(socket: UdpSocket, peer_addr: SocketAddr) -> Result<Self, NetworkError> {
        socket.connect(peer_addr).map_err(|_| NetworkError::ConnectionFailed)?;
        Ok(Client { socket, peer_addr })
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize, NetworkError> {
        self.socket.send(&data).map_err(|_| NetworkError::SendFailed)
    }

    pub fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, NetworkError> {
        self.socket.recv(buffer).map_err(|_| NetworkError::ReceiveFailed)
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }
}