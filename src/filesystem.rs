use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std::path::Path;
use std::result::Result;

#[derive(Debug)]
pub enum FSError {
    FileNotFound,
    PathNotExists,
    FileCreationFailed,
    DirCreationFailed,
    FileReadFailed,
    FileWriteFailed,
    FileOpenFailed,
    NotADirectory,
    NotHasParent,
    NotAFile,
    ReadDirFailed,
    UnpackFailed,
    FileAlreadyExists,
    DirectoryFound,
    MetadataFailed,
    RemovingFailed,
    FlushFailed,
    SyncFailed,
}

impl From<FSError> for Error {
    fn from(e: FSError) -> Error {
        match e {
            FSError::FileNotFound => Error::new(ErrorKind::Other, "File not found"),
            FSError::PathNotExists => Error::new(ErrorKind::Other, "Path not found"),
            FSError::FileCreationFailed => Error::new(ErrorKind::Other, "File creation failed"),
            FSError::DirCreationFailed => Error::new(ErrorKind::Other, "Dir creation failed"),
            FSError::FileReadFailed => Error::new(ErrorKind::Other, "File read failed"),
            FSError::FileWriteFailed => Error::new(ErrorKind::Other, "File write failed"),
            FSError::FileOpenFailed => Error::new(ErrorKind::Other, "File open failed"),
            FSError::NotADirectory => Error::new(ErrorKind::Other, "Not a directory"),
            FSError::NotHasParent => Error::new(ErrorKind::Other, "Not has a parent directory"),
            FSError::NotAFile => Error::new(ErrorKind::Other, "Not a file"),
            FSError::ReadDirFailed => Error::new(ErrorKind::Other, "Read dir failed"),
            FSError::UnpackFailed => Error::new(ErrorKind::Other, "Unpack failed"),
            FSError::FileAlreadyExists => Error::new(ErrorKind::Other, "File already exists"),
            FSError::DirectoryFound => Error::new(ErrorKind::Other, "Directory found"),
            FSError::MetadataFailed => Error::new(ErrorKind::Other, "Metadata failed"),
            FSError::RemovingFailed => Error::new(ErrorKind::Other, "Removing failed"),
            FSError::FlushFailed => Error::new(ErrorKind::Other, "Flush failed"),
            FSError::SyncFailed => Error::new(ErrorKind::Other, "Sync failed"),
        }
    }
}

pub fn remove_file(path_str: &str) -> Result<(), FSError> {
    let path = Path::new(path_str);
    match fs::remove_file(path) {
        Ok(_) => Ok(()),
        Err(_) => Err(FSError::RemovingFailed),
    }
}

pub struct FileChunkReader {
    reader: BufReader<File>,
    chunk_size: usize
}

impl FileChunkReader {
    pub fn new(path_str: &str, chunk_size: usize) -> Result<Self, FSError> {
        let path = Path::new(&path_str);

        if !path.is_file() {
            return Err(FSError::FileNotFound);
        }

        let file = match File::open(path) {
            Ok(v) => v,
            Err(_) => return Err(FSError::FileOpenFailed),
        };

        Ok(FileChunkReader { reader: BufReader::new(file), chunk_size })
    }

    pub fn get_size(&self) -> Result<u64, FSError> {
        match self.reader.get_ref().metadata() {
            Ok(v) => Ok(v.len()),
            Err(_) => Err(FSError::MetadataFailed),
        }
    }
}

impl Iterator for FileChunkReader {
    type Item = Result<Vec<u8>, FSError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = vec![0u8; self.chunk_size];
        match self.reader.read(&mut buffer) {
            Ok(n) => {
                buffer.truncate(n);
                Some(Ok(buffer))
            },
            Err(error) if error.kind() == ErrorKind::Interrupted => None,
            Err(_) => Some(Err(FSError::FileReadFailed)),
        }
    }
}

pub struct FileChunkWriter { // TODO optimum
    writer: BufWriter<File>,
}

impl FileChunkWriter {
    pub fn new(path_str: &str) -> Result<Self, FSError> { // TODO Rewrite error handling
        let path = Path::new(&path_str);
        if path.is_file() {
            return Err(FSError::FileAlreadyExists);
        }

        if path.is_dir() {
            return Err(FSError::DirectoryFound);
        }

        // if path.extension().is_none() {
        //     return Err(FSError::NotAFile);
        // }

        let parent = match path.parent() {
            Some(p) => p,
            None => return Err(FSError::NotHasParent)
        };

        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|_| FSError::DirCreationFailed)?;
        }

        let file = match File::create(path) {
            Ok(f) => f,
            Err(_) => return Err(FSError::FileCreationFailed)
        };
        Ok(FileChunkWriter { writer: BufWriter::new(file) })
    }

    pub fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), FSError> {
        Ok(self.writer.write_all(chunk).map_err(|_| FSError::FileWriteFailed)?)
    }

    pub fn finish(&mut self) -> Result<(), FSError> {
        self.writer.flush().map_err(|_| FSError::FlushFailed)?;
        Ok(self.writer.get_ref().sync_all().map_err(|_| FSError::SyncFailed)?)
    }
}

pub fn get_fs_entries(path_str: &String) -> Result<Vec<OsString>, FSError> {
    let path = Path::new(&path_str);
    if path.exists() {
        let mut entries: Vec<OsString> = Vec::new();
        if path.is_dir() {
            let read_dir = fs::read_dir(path).map_err(|_| FSError::ReadDirFailed)?;
            for entry_result in read_dir {
                let entry = entry_result.map_err(|_| FSError::UnpackFailed)?;
                let path = entry.path();
                let filename_osstr = match path.file_name() {
                    Some(osstr) => osstr,
                    None => return Err(FSError::UnpackFailed),
                };

                entries.push(filename_osstr.to_os_string());
            }

            return Ok(entries);
        }

        return Err(FSError::NotADirectory)
    }

    Err(FSError::PathNotExists)
}