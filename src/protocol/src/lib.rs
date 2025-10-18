use packet::{Packet, PacketField};
use bytel::u64_to_str;
use protocol_spec::{FieldStatus, FieldType, PacketMethod};

pub fn generate_error_response_packet(method: u8, err_msg: &str) -> Packet {
    let resp_fields = vec![
        PacketField::new(FieldType::Status as u8, 1, vec![FieldStatus::Error as u8]),
        PacketField::new(FieldType::ErrorMsg as u8, err_msg.len() as u16, err_msg.as_bytes().to_vec())
    ];

    Packet::new(method, resp_fields.len() as u8, resp_fields)
}

pub fn generate_status_ready_response_packet(method: u8, operation_id: u64, file_size: u64,
                                             chunk_size: u64, chunk_count: u64) -> Packet {
    let operation_id_str = u64_to_str(operation_id);
    let chunk_size_str = u64_to_str(chunk_size);
    let chunk_count_str = u64_to_str(chunk_count);
    if method == PacketMethod::Download as u8 {
        let file_size_str = u64_to_str(file_size);
        let resp_fields = vec![
            PacketField::new(FieldType::Status as u8, 1, vec![FieldStatus::Ready as u8]),
            PacketField::new(FieldType::OperationID as u8, operation_id_str.len() as u16,
                             operation_id_str.as_bytes().to_vec()),
            PacketField::new(FieldType::FileSize as u8, file_size_str.len() as u16,
                             file_size_str.as_bytes().to_vec()),
            PacketField::new(FieldType::ChunkSize as u8, chunk_size_str.len() as u16,
                             chunk_size_str.as_bytes().to_vec()),
            PacketField::new(FieldType::ChunksCount as u8, chunk_count_str.len() as u16,
                             chunk_count_str.as_bytes().to_vec())
        ];

        return Packet::new(method, resp_fields.len() as u8, resp_fields);
    }

    let resp_fields = vec![
        PacketField::new(FieldType::Status as u8, 1, vec![FieldStatus::Ready as u8]),
        PacketField::new(FieldType::OperationID as u8, operation_id_str.len() as u16,
                         operation_id_str.as_bytes().to_vec()),
        PacketField::new(FieldType::ChunkSize as u8, chunk_size_str.len() as u16,
                         chunk_size_str.as_bytes().to_vec()),
        PacketField::new(FieldType::ChunksCount as u8, chunk_count_str.len() as u16,
                         chunk_count_str.as_bytes().to_vec())
    ];

    Packet::new(method, resp_fields.len() as u8, resp_fields)
}

pub fn generate_status_sent_response_packet(method: u8, cur_chunk_id: u64,
                                            data_chunk: &Vec<u8>) -> Packet {
    let chunk_id = u64_to_str(cur_chunk_id);
    let resp_fields: Vec<PacketField> = vec![
        PacketField::new(FieldType::Status as u8, 1, vec![FieldStatus::Sent as u8]),
        PacketField::new(FieldType::ChunkID as u8, chunk_id.len() as u16, chunk_id.as_bytes().to_vec()),
        PacketField::new(FieldType::DataChunk as u8, data_chunk.len() as u16, data_chunk.clone())
    ];

    Packet::new(method, resp_fields.len() as u8, resp_fields)
}

pub fn generate_status_received_response_packet(method: u8) -> Packet {
    let resp_fields: Vec<PacketField> = vec![
        PacketField::new(FieldType::Status as u8, 1, vec![FieldStatus::Received as u8])
    ];

    Packet::new(method, resp_fields.len() as u8, resp_fields)
}

pub fn generate_status_ok_response_packet(method: u8) -> Packet {
    let mut resp_fields: Vec<PacketField> = Vec::with_capacity(1);
    resp_fields.push(PacketField::new(FieldType::Status as u8, 1,
                                      vec![FieldStatus::Ok as u8]));

    Packet::new(method, resp_fields.len() as u8, resp_fields)
}

pub fn generate_status_cancelled_response_packet(method: u8) -> Packet {
    let mut resp_fields: Vec<PacketField> = Vec::with_capacity(1);
    resp_fields.push(PacketField::new(FieldType::Status as u8, 1,
                                      vec![FieldStatus::Cancelled as u8]));

    Packet::new(method, resp_fields.len() as u8, resp_fields)
}