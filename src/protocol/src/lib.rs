mod packet;
mod errors;
pub mod enums;
mod utils;
pub mod context;

use std::io::Error;
use packet::*;
use utils::*;
use enums::*;
use context::*;

pub fn proceed_error(ctx: &mut ProtocolContext) {
    let response = generate_error_response_packet(ctx);
    ctx.set_response(response);
}

pub fn proceed_request(ctx: &mut ProtocolContext, request_raw: &[u8]) -> Action {
    let request = match Packet::parse(&request_raw) {
        Ok(packet) => packet,
        Err(error) => {
            ctx.set_err_msg(Error::from(error).to_string());
            let response = generate_error_response_packet(ctx);
            ctx.set_response(response);
            return Action::SendError;
        }
    };

    let method = request.get_method();
    if !ctx.get_started() { ctx.set_current_method(method); }

    if !ctx.get_started() && method == PacketMethod::Close as u8 {
        let response = generate_status_ok_response_packet(ctx);
        ctx.set_response(response);
        return Action::SendResponse(NextAction::Terminate);
    }

    if method != request.get_method() {
        ctx.set_err_msg(String::from("Method isn\'t match"));
        let response = generate_error_response_packet(ctx);
        ctx.set_response(response);
        return Action::SendError;
    }

    if request.get_fields()[0].get_field_type() != FieldType::Command as u8 {
        ctx.set_err_msg(String::from("First field should be command"));
        let response = generate_error_response_packet(ctx);
        ctx.set_response(response);
        return Action::SendError;
    }
    let command = request.get_fields()[0].get_field_data()[0];

    if FieldCommand::try_from(command).is_err() {
        ctx.set_err_msg(String::from("Invalid command"));
        let response = generate_error_response_packet(ctx);
        ctx.set_response(response);
        return Action::SendError;
    }

    if !ctx.get_started() && method == PacketMethod::Download as u8 && command == FieldCommand::Start as u8 {
        if !ctx.get_file_open() {
            if request.get_fields_count() != 2 {
                ctx.set_err_msg(String::from("Not valid count of fields for download method"));
                let response = generate_error_response_packet(ctx);
                ctx.set_response(response);
                return Action::SendError;
            }

            if request.get_fields()[1].get_field_type() != FieldType::Path as u8 {
                ctx.set_err_msg(String::from("Second field should be Path"));
                let response = generate_error_response_packet(ctx);
                ctx.set_response(response);
                return Action::SendError;
            }

            let path_str = match parse_str(request.get_fields()[1].get_field_data()) {
                Ok(value) => value,
                Err(error) => {
                    ctx.set_err_msg(Error::from(error).to_string());
                    let response = generate_error_response_packet(ctx);
                    ctx.set_response(response);
                    return Action::SendError;
                }
            };
            ctx.set_file_path(path_str);
            return Action::RequestFileInfoRead;
        }

        if ctx.get_data_chunk(false).is_empty() {
            let response = generate_status_ready_response_packet(ctx);
            ctx.set_response(response);

            if ctx.get_chunk_count() > 0 {
                return Action::SendResponse(NextAction::ReadData);
            }

            ctx.set_started(true);
            return Action::SendResponse(NextAction::None);
        }

        let response = generate_status_sent_response_packet(ctx, false);
        ctx.set_response(response);
        ctx.set_started(true);
        return Action::SendResponse(NextAction::ReadData);
    }

    if !ctx.get_started() && method == PacketMethod::Upload as u8 && command == FieldCommand::Start as u8 {
        if !ctx.get_file_open() {
            if request.get_fields_count() != 3 {
                ctx.set_err_msg(String::from("Not valid count of fields for upload method"));
                let response = generate_error_response_packet(ctx);
                ctx.set_response(response);
                return Action::SendError;
            }

            if request.get_fields()[1].get_field_type() != FieldType::Path as u8 {
                ctx.set_err_msg(String::from("Second field should be Path"));
                let response = generate_error_response_packet(ctx);
                ctx.set_response(response);
                return Action::SendError;
            }

            let path_str = match parse_str(request.get_fields()[1].get_field_data()) {
                Ok(value) => value,
                Err(error) => {
                    ctx.set_err_msg(Error::from(error).to_string());
                    let response = generate_error_response_packet(ctx);
                    ctx.set_response(response);
                    return Action::SendError;
                }
            };
            ctx.set_file_path(path_str);

            if request.get_fields()[2].get_field_type() != FieldType::FileSize as u8 {
                ctx.set_err_msg(String::from("Third field should be FileSize"));
                let response = generate_error_response_packet(ctx);
                ctx.set_response(response);
                return Action::SendError;
            }

            let file_size = match parse_u64(request.get_fields()[2].get_field_data()) {
                Ok(value) => value,
                Err(error) => {
                    ctx.set_err_msg(Error::from(error).to_string());
                    let response = generate_error_response_packet(ctx);
                    ctx.set_response(response);
                    return Action::SendError;
                }
            };

            ctx.set_file_size(file_size);
            return Action::RequestFileInfoWrite;
        }

        let response = generate_status_ready_response_packet(ctx);
        ctx.set_response(response);
        ctx.set_started(true);
        return Action::SendResponse(NextAction::None);
    }

    if ctx.get_started() && method == PacketMethod::Download as u8 && command == FieldCommand::Next as u8 {
        if request.get_fields_count() != 1 {
            ctx.set_err_msg(String::from("Not valid count of fields"));
            let response = generate_error_response_packet(ctx);
            ctx.set_response(response);
            return Action::SendError;
        }

        if ctx.get_current_chunk_id() > ctx.get_chunk_count() {
            ctx.set_err_msg(String::from("File chunk id out of range"));
            let response = generate_error_response_packet(ctx);
            ctx.set_response(response);
            return Action::SendError;
        }

        let response = generate_status_sent_response_packet(ctx, false);
        ctx.set_response(response);

        if ctx.get_current_chunk_id() < ctx.get_chunk_count() {
            return Action::SendResponse(NextAction::ReadData);
        }
        return Action::SendResponse(NextAction::None)
    }

    if ctx.get_started() && method == PacketMethod::Upload as u8 && command == FieldCommand::Send as u8 {
        if request.get_fields_count() != 3 {
            ctx.set_err_msg(String::from("Not valid count of fields"));
            let response = generate_error_response_packet(ctx);
            ctx.set_response(response);
            return Action::SendError;
        }

        if ctx.get_current_chunk_id() + 1 > ctx.get_chunk_count() {
            ctx.set_err_msg(String::from("File chunk id out of range"));
            let response = generate_error_response_packet(ctx);
            ctx.set_response(response);
            return Action::SendError;
        }

        if request.get_fields()[1].get_field_type() != FieldType::ChunkID as u8 {
            ctx.set_err_msg(String::from("Second field should be ChunkID"));
            let response = generate_error_response_packet(ctx);
            ctx.set_response(response);
            return Action::SendError;
        }

        if request.get_fields()[2].get_field_type() != FieldType::DataChunk as u8 {
            ctx.set_err_msg(String::from("Third field should be DataChunk"));
            let response = generate_error_response_packet(ctx);
            ctx.set_response(response);
            return Action::SendError;
        }

        let chunk_id = match parse_u64(request.get_fields()[1].get_field_data()) {
            Ok(chunk_id) => chunk_id,
            Err(error) => {
                ctx.set_err_msg(Error::from(error).to_string());
                let response = generate_error_response_packet(ctx);
                ctx.set_response(response);
                return Action::SendError;
            }
        };

        if ctx.get_current_chunk_id() as u64 + 1 != chunk_id {
            let err_msg = "Excepted ".to_owned() + u64_to_str(ctx.get_current_chunk_id() as u64)
                .to_string().as_str() + " in chunk_id, but found " + u64_to_str(chunk_id).to_string().as_str();
            ctx.set_err_msg(err_msg);
            let response = generate_error_response_packet(ctx);
            ctx.set_response(response);
            return Action::SendError;
        }

        ctx.set_data_chunk(Vec::from(request.get_fields()[2].get_field_data()));
        let response = generate_status_received_response_packet(ctx);
        ctx.set_response(response);
        return Action::SendResponse(NextAction::WriteData);
    }

    if ctx.get_started() && method == PacketMethod::Download as u8 && command == FieldCommand::Retry as u8 {
        if request.get_fields_count() != 1 {
            ctx.set_err_msg(String::from("Not valid count of fields"));
            let response = generate_error_response_packet(ctx);
            ctx.set_response(response);
            return Action::SendError;
        }

        let response = generate_status_sent_response_packet(ctx, true);
        ctx.set_response(response);
        return Action::SendResponse(NextAction::None);
    }

    if ctx.get_started() && command == FieldCommand::End as u8 {
        if request.get_fields_count() != 1 {
            ctx.set_err_msg(String::from("Not valid count of fields"));
            let response = generate_error_response_packet(ctx);
            ctx.set_response(response);
            return Action::SendError;
        }

        if ctx.get_current_chunk_id() != ctx.get_chunk_count() {
            ctx.set_err_msg(String::from("File chunks not ended"));
            let response = generate_error_response_packet(ctx);
            ctx.set_response(response);
            return Action::SendError;
        }

        ctx.set_started(false);
        let response = generate_status_ok_response_packet(ctx);
        ctx.set_response(response);
        return Action::SendResponse(NextAction::End);
    }

    if ctx.get_started() && command == FieldCommand::Cancel as u8 {
        if request.get_fields_count() != 1 {
            ctx.set_err_msg(String::from("Not valid count of fields"));
            let response = generate_error_response_packet(ctx);
            ctx.set_response(response);
            return Action::SendError;
        }

        ctx.set_started(false);
        let response = generate_status_ok_response_packet(ctx);
        ctx.set_response(response);
        return Action::SendResponse(NextAction::Cancel);
    }

    ctx.set_err_msg(String::from("Not valid request or method not started"));
    let response = generate_error_response_packet(ctx);
    ctx.set_response(response);
    Action::SendError
}

fn generate_error_response_packet(ctx: &ProtocolContext) -> Vec<u8> {
    let resp_fields = vec![
        PacketField::new(FieldType::Status as u8, 1, vec![FieldStatus::Error as u8]),
        PacketField::new(FieldType::ErrorMsg as u8, ctx.get_err_msg().len() as u16,
                         ctx.get_err_msg().as_bytes().to_vec())
    ];

    Packet::new(ctx.get_current_method(), resp_fields.len() as u8, resp_fields).get_bytes()
}

fn generate_status_ready_response_packet(ctx: &ProtocolContext) -> Vec<u8> {
    let session_id_str = u64_to_u8_vec(ctx.get_session_id() as u64);
    let chunk_size_str = u64_to_u8_vec(FILE_CHUNK_SIZE as u64);
    let chunk_count_str = u64_to_u8_vec(ctx.get_chunk_count() as u64);
    if ctx.get_current_method() == PacketMethod::Download as u8 {
        let file_size_str = u64_to_u8_vec(ctx.get_file_size());
        let resp_fields = vec![
            PacketField::new(FieldType::Status as u8, 1, vec![FieldStatus::Ready as u8]),
            PacketField::new(FieldType::SessionID as u8, session_id_str.len() as u16, session_id_str),
            PacketField::new(FieldType::FileSize as u8, file_size_str.len() as u16, file_size_str),
            PacketField::new(FieldType::ChunkSize as u8, chunk_size_str.len() as u16, chunk_size_str),
            PacketField::new(FieldType::ChunksCount as u8, chunk_count_str.len() as u16, chunk_count_str)
        ];

        return Packet::new(ctx.get_current_method(), resp_fields.len() as u8, resp_fields).get_bytes();
    }

    let resp_fields = vec![
        PacketField::new(FieldType::Status as u8, 1, vec![FieldStatus::Ready as u8]),
        PacketField::new(FieldType::SessionID as u8, session_id_str.len() as u16, session_id_str),
        PacketField::new(FieldType::ChunkSize as u8, chunk_size_str.len() as u16, chunk_size_str),
        PacketField::new(FieldType::ChunksCount as u8, chunk_count_str.len() as u16, chunk_count_str)
    ];

    Packet::new(ctx.get_current_method(), resp_fields.len() as u8, resp_fields).get_bytes()
}

fn generate_status_sent_response_packet(ctx: &ProtocolContext, is_previous: bool) -> Vec<u8> {
    let chunk_id = u64_to_u8_vec(ctx.get_current_chunk_id() as u64);
    let data_chunk: &[u8] = ctx.get_data_chunk(is_previous);

    let resp_fields: Vec<PacketField> = vec![
        PacketField::new(FieldType::Status as u8, 1, vec![FieldStatus::Sent as u8]),
        PacketField::new(FieldType::ChunkID as u8, chunk_id.len() as u16, chunk_id),
        PacketField::new(FieldType::DataChunk as u8, data_chunk.len() as u16, data_chunk.to_vec()),
    ];

    Packet::new(ctx.get_current_method(), resp_fields.len() as u8, resp_fields).get_bytes()
}

fn generate_status_received_response_packet(ctx: &ProtocolContext) -> Vec<u8> {
    let resp_fields: Vec<PacketField> = vec![
        PacketField::new(FieldType::Status as u8, 1, vec![FieldStatus::Received as u8])
    ];

    Packet::new(ctx.get_current_method(), resp_fields.len() as u8, resp_fields).get_bytes()
}

fn generate_status_ok_response_packet(ctx: &ProtocolContext) -> Vec<u8> {
    let resp_fields: Vec<PacketField> = vec![
        PacketField::new(FieldType::Status as u8, 1, vec![FieldStatus::Ok as u8])
    ];

    Packet::new(ctx.get_current_method(), resp_fields.len() as u8, resp_fields).get_bytes()
}