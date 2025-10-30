struct SessionMeta {
    session_id: u8,
    started: bool,
    current_method: u8,
}

impl SessionMeta {
    fn new(session_id: u8) -> SessionMeta {
        SessionMeta { session_id, started: false, current_method: 0 }
    }

    fn reset(&mut self) {
        self.started = false;
        self.current_method = 0;
    }
}

struct FileState {
    is_open: bool,

    path: String,
    size: u64,
    chunk_count: u32,
    current_chunk_id: u32,
    data_chunk: Vec<u8>,
}

impl FileState {
    fn new() -> FileState {
        FileState { is_open: false, path: String::new(), size: 0, chunk_count: 0, current_chunk_id: 0,
            data_chunk: Vec::new() }
    }

    fn reset(&mut self) {
        self.is_open = false;
        self.path = String::new();
        self.size = 0;
        self.chunk_count = 0;
        self.current_chunk_id = 0;
        self.data_chunk = Vec::new();
    }
}

pub struct ProtocolContext {
    meta: SessionMeta,
    file: FileState,
    response: Vec<u8>,
    err_msg: String,
}

impl ProtocolContext {
    pub fn new(session_id: u8) -> ProtocolContext {
        ProtocolContext { meta: SessionMeta::new(session_id), file: FileState::new(),
            response: Vec::new(), err_msg: String::new() }
    }

    pub fn reset(&mut self) {
        self.meta.reset();
        self.file.reset();
        self.response.clear();
        self.err_msg.clear();
    }

    pub fn get_session_id(&self) -> u8 { self.meta.session_id }
    pub fn get_started(&self) -> bool { self.meta.started }
    pub fn get_current_method(&self) -> u8 { self.meta.current_method }
    pub fn get_response(&self) -> &[u8] { &self.response }
    pub fn get_err_msg(&self) -> &str { &self.err_msg }
    pub fn get_file_open(&self) -> bool { self.file.is_open }
    pub fn get_file_path(&self) -> &str { &self.file.path }
    pub fn get_file_size(&self) -> u64 { self.file.size }
    pub fn get_chunk_count(&self) -> u32 { self.file.chunk_count }
    pub fn get_current_chunk_id(&self) -> u32 { self.file.current_chunk_id }
    pub fn get_data_chunk(&self) -> &[u8] { &self.file.data_chunk }

    pub fn set_started(&mut self, started: bool) { self.meta.started = started; }
    pub fn set_current_method(&mut self, method: u8) { self.meta.current_method = method; }
    pub fn set_response(&mut self, response: Vec<u8>) { self.response = response; }
    pub fn set_err_msg(&mut self, err_msg: String) { self.err_msg = err_msg; }
    pub fn set_file_open(&mut self, open: bool) { self.file.is_open = open; }
    pub fn set_file_path(&mut self, path: String) { self.file.path = path; }
    pub fn set_file_size(&mut self, file_size: u64) { self.file.size = file_size; }
    pub fn set_chunk_count(&mut self, chunk_count: u32) { self.file.chunk_count = chunk_count; }
    pub fn increment_current_chunk_id(&mut self) { self.file.current_chunk_id += 1; }
    pub fn set_data_chunk(&mut self, data_chunk: Vec<u8>) { self.file.data_chunk = data_chunk; }
}