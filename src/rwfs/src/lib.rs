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
        }
    }
}

pub struct FileChunkReader {
    reader: BufReader<File>,
}

impl FileChunkReader {
    pub fn new(path_str: &String) -> Result<Self, FSError> {
        let path = Path::new(&path_str);

        if !path.is_file() {
            return Err(FSError::FileNotFound);
        }

        let file = match File::open(path) {
            Ok(v) => v,
            Err(_) => return Err(FSError::FileOpenFailed),
        };

        Ok(FileChunkReader { reader: BufReader::new(file) })
    }
}

impl Iterator for FileChunkReader {
    type Item = Result<Vec<u8>, FSError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = vec![0u8; 65534];
        match self.reader.read(&mut buffer) {
            Ok(0) => None,
            Ok(n) => {
                buffer.truncate(n);
                Some(Ok(buffer))
            },
            Err(_) => Some(Err(FSError::FileReadFailed)),
        }
    }
}

pub struct FileChunkWriter {
    writer: BufWriter<File>,
}

impl FileChunkWriter {
    pub fn new(path_str: &String) -> Result<Self, FSError> { // Rewrite error handling
        let path = Path::new(&path_str);
        if path.extension().is_none() {
            return Err(FSError::NotAFile);
        }

        if path.is_file() {
            return Err(FSError::FileAlreadyExists);
        }

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
        self.writer.flush().map_err(|_| FSError::FileWriteFailed)?;
        Ok(self.writer.get_ref().sync_all().map_err(|_| FSError::FileWriteFailed)?)
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