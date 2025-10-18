use bytel::{parse_str, parse_u64, u64_to_str};
use netw::{Client, NetwError, Server};
use parser::Parser;
use protocol::{generate_error_response_packet, generate_status_cancelled_response_packet,
               generate_status_ok_response_packet, generate_status_ready_response_packet,
               generate_status_received_response_packet, generate_status_sent_response_packet};
use protocol_spec::{FieldCommand, FieldType, PacketMethod};
use rwfs::{self, FileChunkReader, FileChunkWriter};
use std::fs::{metadata, remove_file};
use std::io::{Error, Result};
use std::path::Path;

const FILE_CHUNK_SIZE: u16 = 65535;

fn ceil(num1: u64, num2: u64) -> u32 {
    if num1 % num2 != 0 {
        return (num1 / num2 + 1) as u32;
    }

    (num1 / num2) as u32
}

fn send_error(client: &mut Client, method: u8, msg: &str) -> std::result::Result<(), NetwError> {
    println!("sending error message: {}", msg);
    let response = generate_error_response_packet(method, msg);
    client.send(&*Parser::get_bytes(&response))
}

pub fn main() -> Result<()> {
    let server = Server::new("1998")?;
    loop {
        // Session struct
        // Session starts own loop, in which user will be handled
        let mut client = server.accept()?;

        let mut started = false;
        let mut method = 0;
        let mut path_str = String::new();

        let mut operation_id: u8 = 0;
        let mut file_size: u64 = 0;
        let mut chunk_count: u32 = 0;
        let mut cur_chunk_id: u32 = 0;

        let mut chunk_reader: Option<FileChunkReader> = None;
        let mut chunk_writer: Option<FileChunkWriter> = None;
        let mut data_chunk: Vec<u8> = Vec::new();
        loop {
            let request_raw: Vec<u8>;
            // TODO maybe bad realization
            if method == PacketMethod::Upload as u8 {
                request_raw = client.receive(262144)?;
            } else {
                request_raw = client.receive(65535)?;
            }

            let request = match Parser::parse(&request_raw) {
                Ok(packet) => packet,
                Err(error) => {
                    println!("error parsing request: {:?}", error);
                    client.send(Error::from(error).to_string().as_ref())?;
                    continue;
                }
            };

            if !started { method = request.get_method(); }

            if !started && method == PacketMethod::Close as u8 {
                client.close()?;
                break;
            }

            if method != request.get_method() {
                send_error(&mut client, method, &"Method isn\'t match".to_string())?;
                continue;
            }

            if request.get_fields()[0].get_field_type() != FieldType::Command as u8 {
                send_error(&mut client, method, &"First field should be command".to_string())?;
                continue;
            }
            let command = request.get_fields()[0].get_field_data()[0];

            if !started && (method == PacketMethod::Download as u8 ||
                method == PacketMethod::Upload as u8) && command == FieldCommand::Start as u8 {
                if (method == PacketMethod::Download as u8 && request.get_fields_count() != 2) ||
                    (method == PacketMethod::Upload as u8 && request.get_fields_count() != 3) {
                    let mut err_msg = "Not valid count of fields for download method";
                    if method == PacketMethod::Upload as u8 {
                        err_msg = "Not valid count of fields for upload method"
                    }

                    send_error(&mut client, method, &err_msg)?;
                    continue;
                }

                if request.get_fields()[1].get_field_type() != FieldType::Path as u8 {
                    send_error(&mut client, method, &"Second field should be path")?;
                    continue;
                }

                path_str = match parse_str(request.get_fields()[1].get_field_data()) {
                    Ok(value) => value,
                    Err(error) => {
                        send_error(&mut client, method, Error::from(error).to_string().as_ref())?;
                        continue;
                    }
                };

                let path = Path::new(path_str.as_str());
                if method == PacketMethod::Download as u8 {
                    if !path.is_file() {
                        send_error(&mut client, method, &"File does not exist")?;
                        continue;
                    }
                }

                if method == PacketMethod::Upload as u8 {
                    if path.extension().is_none() {
                        send_error(&mut client, method, &"Not a file")?;
                        continue;
                    }

                    if path.is_file() {
                        send_error(&mut client, method, &"File already exists")?;
                        continue;
                    }

                    if request.get_fields()[2].get_field_type() != FieldType::FileSize as u8 {
                        send_error(&mut client, method, &"Third field should be file_size")?;
                        continue;
                    }

                    file_size = match parse_u64(request.get_fields()[2].get_field_data()) {
                        Ok(value) => value,
                        Err(error) => {
                            send_error(&mut client, method, Error::from(error).to_string().as_ref())?;
                            continue;
                        }
                    };
                }

                operation_id = 1;
                if file_size == 0 {
                    file_size = match metadata(path) {
                        Ok(value) => value.len(),
                        Err(_) => {
                            send_error(&mut client, method, &"File does not exist")?;
                            continue;
                        }
                    };
                }

                chunk_count = ceil(file_size, FILE_CHUNK_SIZE as u64);
                cur_chunk_id = 1;

                let response = generate_status_ready_response_packet(method, operation_id as u64,
                                                                     file_size, FILE_CHUNK_SIZE as u64,
                                                                     chunk_count as u64);
                client.send(&*Parser::get_bytes(&response))?;

                if method == PacketMethod::Download as u8 {
                    chunk_reader = match FileChunkReader::new(&path_str) {
                        Ok(reader) => Some(reader),
                        Err(error) => {
                            send_error(&mut client, method, Error::from(error).to_string().as_ref())?;
                            continue;
                        }
                    };

                    // TODO remove duplicate(1)
                    if let Some(ref mut reader) = chunk_reader {
                        data_chunk = match reader.next() {
                            Some(result) => match result {
                                Ok(chunk) => chunk,
                                Err(error) => {
                                    send_error(&mut client, method, Error::from(error).to_string().as_ref())?;
                                    continue;
                                }
                            },
                            None => {
                                send_error(&mut client, method, "Iterator ended up")?;
                                continue;
                            }
                        };
                    } else {
                        send_error(&mut client, method, "File chunk reader is none")?;
                        continue;
                    }

                    let response = generate_status_sent_response_packet(method,
                                                                        cur_chunk_id as u64,
                                                                        &data_chunk);
                    client.send(&*Parser::get_bytes(&response))?;

                    cur_chunk_id += 1;
                }

                if method == PacketMethod::Upload as u8 {
                    chunk_writer = match FileChunkWriter::new(&path_str) {
                        Ok(writer) => Some(writer),
                        Err(error) => {
                            send_error(&mut client, method, Error::from(error).to_string().as_ref())?;
                            continue;
                        }
                    };
                }

                started = true;
                continue;
            }

            if started && method == PacketMethod::Download as u8 && command == FieldCommand::Next as u8 {
                if request.get_fields_count() != 1 {
                    send_error(&mut client, method, &"Not valid count of fields")?;
                    continue;
                }

                if cur_chunk_id > chunk_count {
                    send_error(&mut client, method, &"File chunk id out of range")?;
                    continue;
                }

                // TODO remove duplicate(1)
                if let Some(ref mut reader) = chunk_reader {
                    data_chunk = match reader.next() {
                        Some(result) => match result {
                            Ok(chunk) => chunk,
                            Err(error) => {
                                send_error(&mut client, method, Error::from(error).to_string().as_ref())?;
                                continue;
                            }
                        },
                        None => {
                            send_error(&mut client, method, "Iterator ended up")?;
                            continue;
                        }
                    };
                } else {
                    send_error(&mut client, method, "File chunk reader is none")?;
                    continue;
                }

                let response = generate_status_sent_response_packet(method,
                                                                    cur_chunk_id as u64,
                                                                    &data_chunk);
                client.send(&*Parser::get_bytes(&response))?;

                cur_chunk_id += 1;
                continue;
            }

            if started && method == PacketMethod::Upload as u8 && command == FieldCommand::Send as u8 {
                if request.get_fields_count() != 3 {
                    send_error(&mut client, method, &"Not valid count of fields")?;
                    continue;
                }

                if cur_chunk_id > chunk_count {
                    send_error(&mut client, method, &"File chunk id out of range")?;
                    continue;
                }

                if request.get_fields()[1].get_field_type() != FieldType::ChunkID as u8 {
                    send_error(&mut client, method, &"Second field should be chunk_id")?;
                    continue;
                }

                if request.get_fields()[2].get_field_type() != FieldType::DataChunk as u8 {
                    send_error(&mut client, method, &"Third field should be data chunk")?;
                    continue;
                }

                let chunk_id = match parse_u64(request.get_fields()[1].get_field_data()) {
                    Ok(chunk_id) => chunk_id,
                    Err(error) => {
                        send_error(&mut client, method, Error::from(error).to_string().as_ref())?;
                        continue;
                    }
                };

                if cur_chunk_id as u64 != chunk_id {
                    let err_msg = "Excepted ".to_owned() + u64_to_str(cur_chunk_id as u64).to_string().as_str()
                        + " in chunk_id, but found " + u64_to_str(chunk_id).to_string().as_str();
                    send_error(&mut client, method, err_msg.as_str())?;
                    continue;
                }

                data_chunk = Vec::from(request.get_fields()[2].get_field_data());

                if let Some(ref mut writer) = chunk_writer {
                    match writer.write_chunk(&*data_chunk) {
                        Ok(_) => {
                            let response = generate_status_received_response_packet(method);
                            client.send(&*Parser::get_bytes(&response))?;

                            cur_chunk_id += 1;
                            continue;
                        },
                        Err(error) => {
                            send_error(&mut client, method, Error::from(error).to_string().as_ref())?;
                            continue;
                        }
                    }
                } else {
                    send_error(&mut client, method, "File chunk writer is none")?;
                    continue;
                }
            }

            if started && method == PacketMethod::Download as u8 && command == FieldCommand::Retry as u8 {
                if request.get_fields_count() != 1 {
                    send_error(&mut client, method, &"Not valid count of fields")?;
                    continue;
                }

                let response = generate_status_sent_response_packet(method,
                                                                    (cur_chunk_id - 1) as u64,
                                                                    &data_chunk);
                client.send(&*Parser::get_bytes(&response))?;
                continue;
            }

            if started && command == FieldCommand::End as u8 {
                if request.get_fields_count() != 1 {
                    send_error(&mut client, method, &"Not valid count of fields")?;
                    continue;
                }

                if cur_chunk_id != chunk_count + 1 {
                    send_error(&mut client, method, &"This chunk is not a last")?;
                    continue;
                }

                started = false;
                if !chunk_writer.is_none() {
                    // TODO remove duplicate(2)
                    if let Some(ref mut writer) = chunk_writer {
                        match writer.finish() {
                            Ok(_) => (),
                            Err(error) => {
                                send_error(&mut client, method, Error::from(error).to_string().as_ref())?;
                                continue;
                            }
                        }
                    }
                }
                chunk_writer = None;
                chunk_reader = None;

                let response = generate_status_ok_response_packet(method);
                client.send(&*Parser::get_bytes(&response))?;
                continue;
            }

            if started && command == FieldCommand::Cancel as u8 {
                if request.get_fields_count() != 1 {
                    send_error(&mut client, method, &"Not valid count of fields")?;
                    continue;
                }

                started = false;
                if !chunk_writer.is_none() {
                    // TODO remove duplicate(2)
                    if let Some(ref mut writer) = chunk_writer {
                        match writer.finish() {
                            Ok(_) => (),
                            Err(error) => {
                                send_error(&mut client, method, Error::from(error).to_string().as_ref())?;
                                continue;
                            }
                        }
                    }

                    match remove_file(Path::new(&path_str)) {
                        Ok(_) => (),
                        Err(error) => {
                            send_error(&mut client, method, Error::from(error).to_string().as_ref())?;
                            continue;
                        }
                    };
                }
                chunk_writer = None;
                chunk_reader = None;

                let response = generate_status_cancelled_response_packet(method);
                client.send(&*Parser::get_bytes(&response))?;
                continue;
            }

            if command != FieldCommand::Start as u8 || command == FieldCommand::Next as u8 ||
                command != FieldCommand::Send as u8 || command != FieldCommand::Retry as u8 ||
                command != FieldCommand::End as u8 || command != FieldCommand::Cancel as u8 {
                send_error(&mut client, method, &"Invalid command")?;
                continue;
            }

            send_error(&mut client, method, &"Not valid request or method not started")?;
            continue;
        }

        // break;
    }

    // Ok(())
}