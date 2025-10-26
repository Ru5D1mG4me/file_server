use std::io::Error;
use super::network::{Client};
use super::filesystem::{remove_file, FileChunkReader, FileChunkWriter};
use super::utils::ceil;
use protocol::context::ProtocolContext;
use protocol::enums::{FILE_CHUNK_SIZE, Action as ProtocolAction, NextAction as ProtocolNextAction};
use protocol::{proceed_error, proceed_request};

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
    ctx: ProtocolContext,
    state: SessionState,

    new_request: bool,
    request: Vec<u8>,
}

impl Session {
    pub fn new(client: Client, session_id: u8) -> Session {
        Session { client, ctx: ProtocolContext::new(session_id), state: SessionState::None, new_request: true,
            request: Vec::new() }
    }

    fn handle_send_error(&mut self) -> Action {
        println!("Error: {}", self.ctx.get_err_msg());
        if let Err(error) = self.client.send(self.ctx.get_response()) {
            println!("Error while sending response: {}", Error::from(error).to_string());
            return Action::Break;
        }

        Action::Continue
    }
    fn handle_fileinfo_write(&mut self) -> Action {
        let writer = match FileChunkWriter::new(self.ctx.get_file_path()) {
            Ok(writer) => writer,
            Err(error) => {
                let err_msg = Error::from(error).to_string().as_str().to_owned();
                println!("Error: {}", err_msg);
                self.ctx.set_err_msg(err_msg);
                proceed_error(&mut self.ctx);

                if let Err(error) = self.client.send(self.ctx.get_response()) {
                    println!("Error while sending response: {}", Error::from(error).to_string());
                    return Action::Break;
                };

                return Action::Continue;
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

                if let Err(error) = self.client.send(self.ctx.get_response()) {
                    println!("Error while sending response: {}", Error::from(error).to_string());
                    return Action::Break;
                };

                return Action::Continue;
            }
        };

        let file_size = match reader.get_size() {
            Ok(file_size) => file_size,
            Err(error) => {
                let err_msg = Error::from(error).to_string().as_str().to_owned();
                println!("Error: {}", err_msg);
                self.ctx.set_err_msg(err_msg);
                proceed_error(&mut self.ctx);

                if let Err(error) = self.client.send(self.ctx.get_response()) {
                    println!("Error while sending response: {}", Error::from(error).to_string());
                    return Action::Break;
                };

                return Action::Continue;
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
        if let Err(error) = self.client.send(self.ctx.get_response()) {
            println!("Error while sending response: {}", Error::from(error).to_string());
        };

        // Drop client

        Action::Break
    }

    fn handle_read_data(&mut self) -> Action {
        if let Err(error) = self.client.send(self.ctx.get_response()) {
            println!("Error while sending response: {}", Error::from(error).to_string());
            return Action::Break;
        };

        if let SessionState::Reading(reader) = &mut self.state {
            if let Some(value) = reader.next() {
                match value {
                    Ok(chunk) => self.ctx.set_data_chunk(chunk),
                    Err(error) => {
                        let err_msg = Error::from(error).to_string().as_str().to_owned();
                        println!("Error: {}", err_msg);
                        self.ctx.set_err_msg(err_msg);
                        proceed_error(&mut self.ctx);

                        if let Err(error) = self.client.send(self.ctx.get_response()) {
                            println!("Error while sending response: {}", Error::from(error).to_string());
                            return Action::Break;
                        };

                        return Action::Continue;
                    }
                }
            } else {
                let err_msg = "ChunkReader returned None".to_string();
                println!("Error: {}", err_msg);
                self.ctx.set_err_msg(err_msg);
                proceed_error(&mut self.ctx);

                if let Err(error) = self.client.send(self.ctx.get_response()) {
                    println!("Error while sending response: {}", Error::from(error).to_string());
                    return Action::Break;
                };

                return Action::Continue;
            }
        }

        if !self.new_request && self.ctx.get_started() {
            self.new_request = true;
        }

        self.ctx.increment_current_chunk_id();
        Action::Continue
    }

    fn handle_write_data(&mut self) -> Action {
        if let Err(error) = self.client.send(self.ctx.get_response()) {
            println!("Error while sending response: {}", Error::from(error).to_string());
            return Action::Break;
        }

        if let SessionState::Writing(writer) = &mut self.state {
            match writer.write_chunk(self.ctx.get_data_chunk(false)) {
                Ok(()) => (),
                Err(error) => {
                    let err_msg = Error::from(error).to_string().as_str().to_owned();
                    println!("Error: {}", err_msg);
                    self.ctx.set_err_msg(err_msg);
                    proceed_error(&mut self.ctx);

                    if let Err(error) = self.client.send(self.ctx.get_response()) {
                        println!("Error while sending response: {}", Error::from(error).to_string());
                        return Action::Break;
                    };

                    return Action::Continue;
                }
            };
        }

        self.ctx.increment_current_chunk_id();
        Action::Continue
    }

    fn handle_end(&mut self) -> Action {
        if let Err(error) = self.client.send(self.ctx.get_response()) {
            println!("Error while sending response: {}", Error::from(error).to_string());
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

                    if let Err(error) = self.client.send(self.ctx.get_response()) {
                        println!("Error while sending response: {}", Error::from(error).to_string());
                        return Action::Break;
                    };

                    return Action::Continue;
                }
            };
        }

        self.state = SessionState::None;
        self.ctx.reset();

        Action::Continue
    }

    fn handle_cancel(&mut self) -> Action {
        if let Err(error) = self.client.send(self.ctx.get_response()) {
            println!("Error while sending response: {}", Error::from(error).to_string());
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

                    if let Err(error) = self.client.send(self.ctx.get_response()) {
                        println!("Error while sending response: {}", Error::from(error).to_string());
                        return Action::Break;
                    };

                    return Action::Continue;
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

                    if let Err(error) = self.client.send(self.ctx.get_response()) {
                        println!("Error while sending response: {}", Error::from(error).to_string());
                        return Action::Break;
                    };

                    return Action::Continue;
                }
            }
        }

        self.ctx.reset();
        Action::Continue
    }

    fn handle_none(&mut self) -> Action {
        if let Err(error) = self.client.send(self.ctx.get_response()) {
            println!("Error while sending response: {}", Error::from(error).to_string());
            return Action::Break;
        };

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

                self.request = buffer[..size].to_vec();
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