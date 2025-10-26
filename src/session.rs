use std::fs::{metadata, remove_file};
use std::io::Error;
use std::path::Path;
use super::network::{Client, NetworkError};
use super::filesystem::{FileChunkReader, FileChunkWriter};
use protocol::context::ProtocolContext;
use protocol::Action;
use protocol::enums::*;

pub struct Session {
    client: Client,
    ctx: ProtocolContext,

    chunk_reader: Option<FileChunkReader>,
    chunk_writer: Option<FileChunkWriter>,
}

impl Session {
    pub fn new(client: Client, session_id: u8) -> Session {
        Session { client, ctx: ProtocolContext::new(session_id), chunk_reader: None, chunk_writer: None }
    }

    fn safe_receive(&mut self, size: usize) -> Option<Vec<u8>> {
        match self.client.receive(size as u32) {
            Ok(data) => Some(data),
            Err(error) => match error {
                NetwError::ConnectionAborted => {
                    println!("Connection aborted");
                    None
                },
                _ => {
                    println!("Error: {:?}", error);
                    Some(Vec::new())
                },
            }
        }
    }

    fn safe_send(&mut self, data: &[u8]) -> Action {
        match self.client.send(data) {
            Ok(_) => Action::Continue,
            Err(error) => match error {
                NetwError::ConnectionAborted => {
                    println!("Connection aborted");
                    Action::Break
                },
                _ => Action::Error(Error::from(error).to_string()),
            }
        }
    }

    fn safe_close(&mut self) -> Action {
        match self.client.close() {
            Ok(_) => Action::Break,
            Err(error) => Action::Error(Error::from(error).to_string()),
        }
    }

    fn send_error(&mut self, msg: &str) -> Action {
        println!("sending error message: {}", msg);
        // let response = generate_error_response_packet(self.method, msg);
        self.safe_send(&*Parser::get_bytes(&response))
    }

    pub fn start(&mut self) {
        loop {
            let size = if self.method == PacketMethod::Upload as u8 { 262144 } else { 65535 };
            let request_raw = match self.safe_receive(size) {
                Some(data) if !data.is_empty() => data,
                Some(_) => continue,
                None => break,
            };
        }
    }
}