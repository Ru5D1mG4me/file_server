pub struct PacketField {
    field_type: u8,
    field_data_length: u16,
    field_data: Vec<u8>,
}

impl PacketField {
    pub fn new(field_type: u8, field_data_length: u16, field_data: Vec<u8>) -> Self {
        PacketField{ field_type, field_data_length, field_data }
    }

    pub fn get_field_type(&self) -> u8 {
        self.field_type
    }
    pub fn get_field_data_length(&self) -> u16 {
        self.field_data_length
    }
    pub fn get_field_data(&self) -> &[u8] {
        &self.field_data[..]
    }
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

    pub fn get_method(&self) -> u8 { self.method }
    pub fn get_fields_count(&self) -> u8 { self.fields_count }
    pub fn get_fields(&self) -> &Vec<PacketField> { &self.fields }
}