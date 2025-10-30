use std::collections::HashSet;
use super::errors::ParseError;
use super::enums::{PacketMethod, FieldType, EOF};

pub struct PacketField {
    field_type: u8,
    field_data_length: u16,
    field_data: Vec<u8>,
}

impl PacketField {
    pub fn new(field_type: u8, field_data_length: u16, field_data: Vec<u8>) -> Self {
        PacketField{ field_type, field_data_length, field_data }
    }

    pub fn get_field_type(&self) -> u8 { self.field_type }
    pub fn get_field_data_length(&self) -> u16 { self.field_data_length }
    pub fn get_field_data(&self) -> &[u8] { &self.field_data }
}

pub struct Packet {
    method: u8,
    fields_count: u8,
    fields: Vec<PacketField>,
}

impl Packet {
    pub fn new(method: u8, fields_count: u8, fields: Vec<PacketField>) -> Self {
        Packet { method, fields_count, fields}
    }

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
        let mut seen_types: HashSet<u8> = HashSet::with_capacity(fields_count as usize);

        for _ in 0..fields_count {
            if i + 3 > raw_data.len() {
                return Err(ParseError::NotValidFieldLength);
            }

            let field_type: u8 = raw_data[i]; i += 1;
            if FieldType::try_from(field_type).is_err() {
                return Err(ParseError::NotValidFieldType);
            }

            if !seen_types.insert(field_type) {
                return Err(ParseError::DuplicateFieldFound);
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

        if (i + 1) < raw_data.len() {
            return Err(ParseError::NotValidFieldsCount);
        }

        Ok(Packet::new(method, fields_count, fields))
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut size: usize = 2;
        for i in 0..self.get_fields().len() {
            size += 3 + 1 + self.get_fields()[i].get_field_data_length() as usize;
        }

        let mut bytes = Vec::with_capacity(size);
        bytes.push(self.get_method());
        bytes.push(self.get_fields().len() as u8);

        for i in 0..self.get_fields().len() {
            bytes.push(self.get_fields()[i].get_field_type());
            bytes.extend_from_slice(&(self.get_fields()[i].get_field_data_length() + 1).to_be_bytes());
            bytes.extend_from_slice(&self.get_fields()[i].get_field_data());
            bytes.push(EOF);
        }

        bytes
    }

    pub fn get_method(&self) -> u8 { self.method }
    pub fn get_fields_count(&self) -> u8 { self.fields_count }
    pub fn get_fields(&self) -> &Vec<PacketField> { &self.fields }
}