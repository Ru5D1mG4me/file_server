use std::io::Error;
use super::network::{Client};
use super::filesystem::{remove_file, FileChunkReader, FileChunkWriter};
use super::utils::ceil;
use protocol::context::ProtocolContext;
use protocol::enums::{FILE_CHUNK_SIZE, Action as ProtocolAction, NextAction as ProtocolNextAction};
use protocol::{proceed_error, proceed_request, proceed_retry};
use crc32fast::hash;
use super::cypher::Cypher;

enum Action {
    Continue,
    Break,
}

enum SessionState {
    None,
    Reading(FileChunkReader),
    Writing(FileChunkWriter),
}

pub struct Session {
    client: Client,
    cypher: Cypher,
    ctx: ProtocolContext,
    state: SessionState,

    new_request: bool,
    request: Vec<u8>,
}

impl Session {
    pub fn new(client: Client, session_id: u8, cypher: Cypher) -> Session {
        Session { client, cypher, ctx: ProtocolContext::new(session_id),
            state: SessionState::None, new_request: true, request: Vec::new() }
    }

    fn encrypted_response(&mut self) -> Vec<u8> {
        match self.cypher.encrypt(self.ctx.get_response()) {
            Ok(encrypted_data) => encrypted_data,
            Err(_) => panic!("Error encrypting response"),
        }
    }

    fn encrypted_response_with_crc(&mut self) -> Vec<u8> {
        let encrypted_data = &self.encrypted_response()[..];
        let crc = hash(encrypted_data);
        [&crc.to_be_bytes()[..], encrypted_data].concat()
    }

    fn send_response(&mut self) -> Action {
        let response = self.encrypted_response_with_crc();
        if let Err(error) = self.client.send(&response) {
            println!("Error while sending response: {}", Error::from(error).to_string());
            return Action::Break;
        }

        Action::Continue
    }

    fn handle_send_error(&mut self) -> Action {
        println!("Error: {}", self.ctx.get_err_msg());
        self.send_response()
    }
    fn handle_fileinfo_write(&mut self) -> Action {
        let writer = match FileChunkWriter::new(self.ctx.get_file_path()) {
            Ok(writer) => writer,
            Err(error) => {
                let err_msg = Error::from(error).to_string().as_str().to_owned();
                println!("Error: {}", err_msg);
                self.ctx.set_err_msg(err_msg);
                proceed_error(&mut self.ctx);
                return self.send_response();
            }
        };
        self.state = SessionState::Writing(writer);

        // self.ctx.increment_current_chunk_id();
        let chunk_count = ceil(self.ctx.get_file_size(), FILE_CHUNK_SIZE as u64);
        self.ctx.set_chunk_count(chunk_count);
        self.ctx.set_file_open(true);
        self.new_request = false;
        Action::Continue
    }

    fn handle_fileinfo_read(&mut self) -> Action {
        let reader = match FileChunkReader::new(self.ctx.get_file_path(), FILE_CHUNK_SIZE as usize) {
            Ok(reader) => reader,
            Err(error) => {
                let err_msg = Error::from(error).to_string().as_str().to_owned();
                println!("Error: {}", err_msg);
                self.ctx.set_err_msg(err_msg);
                proceed_error(&mut self.ctx);
                return self.send_response();
            }
        };

        let file_size = match reader.get_size() {
            Ok(file_size) => file_size,
            Err(error) => {
                let err_msg = Error::from(error).to_string().as_str().to_owned();
                println!("Error: {}", err_msg);
                self.ctx.set_err_msg(err_msg);
                proceed_error(&mut self.ctx);
                return self.send_response();
            }
        };
        self.state = SessionState::Reading(reader);

        self.ctx.set_file_size(file_size);
        let chunk_count = ceil(file_size, FILE_CHUNK_SIZE as u64);
        self.ctx.set_chunk_count(chunk_count);
        self.ctx.set_file_open(true);
        self.new_request = false;
        Action::Continue
    }

    fn handle_terminate(&mut self) -> Action {
        match self.send_response() {
            Action::Break => Action::Break,
            Action::Continue => Action::Break,
        }
    }

    fn handle_read_data(&mut self) -> Action {
        if let Action::Break = self.send_response() {
            return Action::Break;
        }

        if let SessionState::Reading(reader) = &mut self.state {
            if let Some(value) = reader.next() {
                match value {
                    Ok(chunk) => self.ctx.set_data_chunk(chunk),
                    Err(error) => {
                        let err_msg = Error::from(error).to_string().as_str().to_owned();
                        println!("Error: {}", err_msg);
                        self.ctx.set_err_msg(err_msg);
                        proceed_error(&mut self.ctx);
                        return self.send_response();
                    }
                }
            } else {
                let err_msg = "ChunkReader returned None".to_string();
                println!("Error: {}", err_msg);
                self.ctx.set_err_msg(err_msg);
                proceed_error(&mut self.ctx);
                return self.send_response();
            }
        }

        if !self.new_request && self.ctx.get_started() {
            self.new_request = true;
        }

        self.ctx.increment_current_chunk_id();
        Action::Continue
    }

    fn handle_write_data(&mut self) -> Action {
        if let Action::Break = self.send_response() {
            return Action::Break;
        }

        if let SessionState::Writing(writer) = &mut self.state {
            match writer.write_chunk(self.ctx.get_data_chunk()) {
                Ok(()) => (),
                Err(error) => {
                    let err_msg = Error::from(error).to_string().as_str().to_owned();
                    println!("Error: {}", err_msg);
                    self.ctx.set_err_msg(err_msg);
                    proceed_error(&mut self.ctx);
                    return self.send_response();
                }
            };
        }

        self.ctx.increment_current_chunk_id();
        Action::Continue
    }

    fn handle_end(&mut self) -> Action {
        if let Action::Break = self.send_response() {
            return Action::Break;
        }

        if let SessionState::Writing(writer) = &mut self.state {
            match writer.finish() {
                Ok(_) => (),
                Err(error) => {
                    let err_msg = Error::from(error).to_string().as_str().to_owned();
                    println!("Error: {}", err_msg);
                    self.ctx.set_err_msg(err_msg);
                    proceed_error(&mut self.ctx);
                    return self.send_response();
                }
            };
        }

        self.state = SessionState::None;
        self.ctx.reset();

        Action::Continue
    }

    fn handle_cancel(&mut self) -> Action {
        if let Action::Break = self.send_response() {
            return Action::Break;
        }

        let mut is_writer = false;
        if let SessionState::Writing(writer) = &mut self.state {
            match writer.finish() {
                Ok(_) => { is_writer = true; },
                Err(error) => {
                    let err_msg = Error::from(error).to_string().as_str().to_owned();
                    println!("Error: {}", err_msg);
                    self.ctx.set_err_msg(err_msg);
                    proceed_error(&mut self.ctx);
                    return self.send_response();
                }
            }
        };

        self.state = SessionState::None;
        if is_writer {
            match remove_file(self.ctx.get_file_path()) {
                Ok(_) => (),
                Err(error) => {
                    let err_msg = Error::from(error).to_string().as_str().to_owned();
                    println!("Error: {}", err_msg);
                    self.ctx.set_err_msg(err_msg);
                    proceed_error(&mut self.ctx);
                    return self.send_response();
                }
            }
        }

        self.ctx.reset();
        Action::Continue
    }

    fn handle_none(&mut self) -> Action {
        if let Action::Break = self.send_response() {
            return Action::Break;
        }

        if !self.new_request && self.ctx.get_started() {
            self.new_request = true;
        }

        Action::Continue
    }

    pub fn start(&mut self) {
        loop {
            if self.new_request {
                let mut buffer = [0u8; FILE_CHUNK_SIZE as usize + 1500];
                let size = match self.client.recv(&mut buffer) {
                    Ok(size) => size,
                    Err(error) => {
                        println!("Error while receiving response: {}", Error::from(error).to_string());
                        break;
                    }
                };

                let crc_bytes: &[u8; 4] = match <&[u8; 4]>::try_from(&buffer[..4]) {
                    Ok(crc) => crc,
                    Err(error) => panic!("Error while parsing CRC: {}", error),
                };
                let crc = u32::from_be_bytes(*crc_bytes);
                let excepted_crc = hash(&buffer[4..size]);
                if crc != excepted_crc {
                    println!("CRC mismatch");
                    proceed_retry(&mut self.ctx);
                    if let Action::Break = self.send_response() {
                        break;
                    };
                }

                let nonce: &[u8; 12] = match <&[u8; 12]>::try_from(&buffer[4..16]) {
                    Ok(nonce) => nonce,
                    Err(error) => panic!("Error while parsing nonce: {}", error),
                };

                self.request = match self.cypher.decrypt(nonce, &buffer[16..size]) {
                    Ok(data) => data,
                    Err(error) => panic!("Error: {}", Error::from(error).to_string()),
                };
            }
            
            let action = match proceed_request(&mut self.ctx, &self.request) {
                ProtocolAction::SendError => {
                    self.handle_send_error();
                    println!("Len: {}", self.request.len());
                    println!("{:?}", self.request);
                    break;
                }
                ProtocolAction::RequestFileInfoRead => { self.handle_fileinfo_read() },
                ProtocolAction::RequestFileInfoWrite => { self.handle_fileinfo_write() },
                ProtocolAction::SendResponse(response) => match response {
                    ProtocolNextAction::Terminate => { self.handle_terminate() },
                    ProtocolNextAction::ReadData => { self.handle_read_data() },
                    ProtocolNextAction::WriteData => { self.handle_write_data() },
                    ProtocolNextAction::End => { self.handle_end() },
                    ProtocolNextAction::Cancel => { self.handle_cancel() },
                    ProtocolNextAction::None => { self.handle_none() },
                },
            };

            match action {
                Action::Continue => continue,
                Action::Break => break,
            }
        }
    }
}