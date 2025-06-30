use std::{fs::File, io::{self, stdin, Read, Seek, SeekFrom, Stdin}, path::{Path, PathBuf}};

#[derive(Debug, Clone, PartialEq)]
pub enum SmartPath {
    FilePath(PathBuf),
    Url(url::Url),
}


struct StdinReader {
    inner: Stdin,
    buffer: Vec<u8>,
    pos: u64,
}
impl StdinReader {
    fn new() -> StdinReader {
        StdinReader {
            inner: stdin(),
            buffer: Vec::new(),
            pos: 0,
        }
    }
}
impl Read for StdinReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let buffer_len = self.buffer.len() as u64;
        if self.pos < buffer_len {
            let bytes_to_read = std::cmp::min(buf.len() as u64, buffer_len - self.pos) as usize;
            buf[..bytes_to_read].copy_from_slice(
                &self.buffer[self.pos as usize..self.pos as usize + bytes_to_read],
            );
            self.pos += bytes_to_read as u64;
            return Ok(bytes_to_read);
        }

        let bytes_read = self.inner.read(buf)?;
        self.buffer.extend_from_slice(&buf[..bytes_read]);
        self.pos += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl Seek for StdinReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(offset) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "SeekFrom::End not supported",
                ));
            }
            SeekFrom::Current(offset) => {
                let new_pos = self.pos as i64 + offset;
                if new_pos < 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Seek before start of stream",
                    ));
                }
                new_pos as u64
            }
        };

        if new_pos > self.buffer.len() as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot seek beyond buffered data",
            ));
        }

        self.pos = new_pos;
        Ok(self.pos)
    }
}


pub fn parse_path(s: &str) -> Result<SmartPath, String> {
    url::Url::parse(s)
        .map(SmartPath::Url)
        .or_else(|_| {
            let path = PathBuf::from(s);
            if path.exists() {
                Ok(SmartPath::FilePath(path))
            } else {
                Err(format!("`{s}` is not a valid URL or file path"))
            }
        })
}

pub(crate) enum SmartReader {
    Stdin(StdinReader),
    File(File),
    Url(reqwest::blocking::Response),
}

impl Read for SmartReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            SmartReader::Stdin(reader) => reader.read(buf),
            SmartReader::File(reader) => reader.read(buf),
            SmartReader::Url(reader) => reader.read(buf),
        }
    }
}



impl TryFrom<Option<&SmartPath>> for SmartReader  {
    fn try_from(value: Option<&SmartPath>) -> Result<Self, Self::Error> {
        match value {
            Some(SmartPath::FilePath(path))=>{
                File::open(path).map(SmartReader::File)
            },
            Some(SmartPath::Url(url))=>{
                reqwest::blocking::get(url.clone()).map(|resp| SmartReader::Url(resp)).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            },
            None=>{Ok(SmartReader::Stdin(StdinReader::new())) }
        }
    }
    
    type Error=io::Error;
    
}