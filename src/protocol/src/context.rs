pub struct ProtocolContext {
    session_id: u8,
    started: bool,
    current_method: Option<u8>,
    path: Option<String>,
    file_size: Option<u32>,
    chunk_count: Option<u32>,
    current_chunk_id: Option<u32>,
    data_chunk: Option<Vec<u8>>,
}

impl ProtocolContext {
    pub fn new(session_id: u8) -> ProtocolContext {
        ProtocolContext {session_id, started: false, current_method: None, path: None, file_size: None,
            chunk_count: None, current_chunk_id: None, data_chunk: None}
    }

    pub fn get_session_id(&self) -> u8 { self.session_id }
    pub fn get_started(&self) -> bool { self.started }
    pub fn get_current_method(&self) -> Option<u8> { self.current_method }
    pub fn get_path(&self) -> Option<String> { self.path.clone() }
    pub fn get_file_size(&self) -> Option<u32> { self.file_size }
    pub fn get_chunk_count(&self) -> Option<u32> { self.chunk_count }
    pub fn get_current_chunk_id(&self) -> Option<u32> { self.current_chunk_id }
    pub fn get_data_chunk(&self) -> Option<Vec<u8>> { self.data_chunk.clone() }

    pub fn set_started(&mut self, started: bool) { self.started = started; }
    pub fn set_current_method(&mut self, method: u8) { self.current_method = Some(method); }
    pub fn set_path(&mut self, path: String) { self.path = Some(path); }
    pub fn set_file_size(&mut self, file_size: u32) { self.file_size = Some(file_size); }
    pub fn set_chunk_count(&mut self, chunk_count: u32) { self.chunk_count = Some(chunk_count); }
    pub fn set_current_chunk_id(&mut self, chunk_id: Option<u32>) { self.current_chunk_id = chunk_id; }
    pub fn set_data_chunk(&mut self, data_chunk: Vec<u8>) { self.data_chunk = Some(data_chunk); }
}