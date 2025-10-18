use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use packet::{Packet, PacketField};
use protocol_spec::{FieldType, PacketMethod, EOF};

#[derive(Debug)]
pub enum ParseError {
    NotValidHeaderLength,
    NotValidFieldLength,
    NotValidFieldDataLength,
    NotValidFieldsCount,
    NotValidMethod,
    NotValidFieldType,
    DuplicateFieldFound,
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Error {
        match error {
            ParseError::NotValidHeaderLength => Error::new(ErrorKind::Other, "Invalid header length"),
            ParseError::NotValidFieldLength => Error::new(ErrorKind::Other, "Invalid field length"),
            ParseError::NotValidFieldDataLength => Error::new(ErrorKind::Other, "Invalid field data length"),
            ParseError::NotValidFieldsCount => Error::new(ErrorKind::Other, "Invalid fields count"),
            ParseError::NotValidMethod => Error::new(ErrorKind::Other, "Invalid method"),
            ParseError::NotValidFieldType => Error::new(ErrorKind::Other, "Invalid field type"),
            ParseError::DuplicateFieldFound => Error::new(ErrorKind::Other, "Duplicate field found"),
        }
    }
}

pub struct Parser {}

impl Parser {
    pub fn parse(raw_data: &[u8]) -> Result<Packet, ParseError> {
        if raw_data.len() < 2 {
            return Err(ParseError::NotValidHeaderLength);
        }

        let mut i = 0;
        let method: u8 = raw_data[i]; i += 1;
        if PacketMethod::try_from(method).is_err() {
            return Err(ParseError::NotValidMethod);
        }

        let fields_count: u8 = raw_data[i]; i += 1;
        let mut fields: Vec<PacketField> = Vec::with_capacity(fields_count as usize);
        let mut seen_types: HashSet<u8> = HashSet::new();

        for _ in 0..fields_count {
            if i + 3 > raw_data.len() {
                return Err(ParseError::NotValidFieldLength);
            }

            let field_type: u8 = raw_data[i]; i += 1;
            if FieldType::try_from(field_type).is_err() {
                return Err(ParseError::NotValidFieldType);
            }

            if !seen_types.insert(field_type) {
                return Err(ParseError::NotValidFieldType);
            }

            let mut field_data_length = u16::from_be_bytes([raw_data[i], raw_data[i + 1]]); i += 2;
            if field_data_length < 2 {
                return Err(ParseError::NotValidFieldDataLength);
            }
            field_data_length -= 1;

            if i + field_data_length as usize > raw_data.len() {
                return Err(ParseError::NotValidFieldDataLength);
            }
            if raw_data[i + field_data_length as usize] != EOF {
                return Err(ParseError::NotValidFieldDataLength)
            }

            let field_data: Vec<u8> = raw_data[i..(i + field_data_length as usize)].to_vec();
            i += (field_data_length + 1) as usize;
            fields.push(PacketField::new(field_type, field_data_length, field_data));
        }

        if i < raw_data.len() {
            return Err(ParseError::NotValidFieldsCount);
        }

        Ok(Packet::new(method, fields_count, fields))
    }

    pub fn get_bytes(packet: &Packet) -> Vec<u8> {
        let mut size: usize = 2;
        for i in 0..packet.get_fields().len() {
            size += 3 + 1 + packet.get_fields()[i].get_field_data_length() as usize;
        }

        let mut bytes = Vec::with_capacity(size);
        bytes.push(packet.get_method());
        bytes.push(packet.get_fields().len() as u8);

        for i in 0..packet.get_fields().len() {
            bytes.push(packet.get_fields()[i].get_field_type());
            bytes.extend_from_slice(&(packet.get_fields()[i].get_field_data_length() + 1).to_be_bytes());
            bytes.extend_from_slice(&packet.get_fields()[i].get_field_data());
            bytes.push(EOF);
        }

        bytes
    }
}