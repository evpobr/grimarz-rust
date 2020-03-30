#![allow(dead_code)]

use std::io;
use std::io::prelude::*;
use std::io::{Cursor, SeekFrom};
use std::result;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub enum Error {
    UnsupportedFormat,
    InvalidStringIndex(usize),
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Default)]
struct Header {
    pub id: u16,
    pub version: u16,
    pub record_table_start: u32,
    pub record_table_size: u32,
    pub record_table_entry_count: u32,
    pub string_table_start: u32,
    pub string_table_size: u32,
}

#[derive(Debug, Default)]
pub struct Entry {
    pub path: String,
    pub record_type: String,
    pub offset: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub file_time: u64,
}

#[derive(Debug)]
pub struct Reader<R: Read + Seek> {
    reader: R,
    entries: Vec<Entry>,
}

impl<R: Read + Seek> Reader<R> {
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn new(reader: R) -> Result<Reader<R>> {
        let mut rdr = Reader {
            reader,
            entries: Vec::new(),
        };
        let entries = rdr.parse_header()?;
        rdr.entries = entries;
        Ok(rdr)
    }

    fn read_header(&mut self) -> Result<Header> {
        let mut header_bytes = vec![0; 6 * 4];
        self.reader.read_exact(&mut header_bytes)?;
        let mut cursor = Cursor::new(header_bytes);
        let mut header = Header::default();
        header.id = cursor.read_u16::<LittleEndian>()?;
        header.version = cursor.read_u16::<LittleEndian>()?;
        if header.id != 0x02 || header.version != 0x03 {
            return Err(Error::UnsupportedFormat);
        }
        header.record_table_start = cursor.read_u32::<LittleEndian>()?;
        header.record_table_size = cursor.read_u32::<LittleEndian>()?;
        header.record_table_entry_count = cursor.read_u32::<LittleEndian>()?;
        header.string_table_start = cursor.read_u32::<LittleEndian>()?;
        header.string_table_size = cursor.read_u32::<LittleEndian>()?;
        Ok(header)
    }

    fn read_strings(&mut self, header: &Header) -> Result<Vec<String>> {
        self.reader
            .seek(SeekFrom::Start(header.string_table_start as u64))? as usize;
        let strings_count = self.reader.read_u32::<LittleEndian>()? as usize;
        let mut strings: Vec<String> = Vec::with_capacity(strings_count);
        for _ in 0..strings_count {
            let string_size = self.reader.read_u32::<LittleEndian>()? as usize;
            let mut string_bytes = vec![0; string_size];
            self.reader.read_exact(&mut string_bytes)?;
            let s = String::from_utf8_lossy(&mut string_bytes);
            strings.push(s.to_string());
        }
        Ok(strings)
    }

    fn read_entries(&mut self, header: Header, strings: &[String]) -> Result<Vec<Entry>> {
        self.reader
            .seek(SeekFrom::Start(header.record_table_start as u64))? as usize;
        let mut entries = Vec::with_capacity(header.record_table_entry_count as usize);
        for _ in 0..header.record_table_entry_count {
            let mut entry = Entry::default();
            let path_index = self.reader.read_u32::<LittleEndian>()? as usize;
            if path_index > strings.len() {
                return Err(Error::InvalidStringIndex(path_index));
            }
            entry.path = strings[path_index].to_string();
            let record_type_len = self.reader.read_u32::<LittleEndian>()? as usize;
            let mut record_type_bytes = vec![0; record_type_len];
            self.reader.read_exact(&mut record_type_bytes)?;
            entry.record_type = String::from_utf8_lossy(&record_type_bytes).to_string();
            entry.offset = self.reader.read_u32::<LittleEndian>()?;
            entry.compressed_size = self.reader.read_u32::<LittleEndian>()?;
            entry.uncompressed_size = self.reader.read_u32::<LittleEndian>()?;
            entry.file_time = self.reader.read_u64::<LittleEndian>()?;
            entries.push(entry);
        }
        Ok(entries)
    }

    fn parse_header(&mut self) -> Result<Vec<Entry>> {
        self.reader.seek(SeekFrom::Start(0))?;
        let header = self.read_header()?;
        let strings = self.read_strings(&header)?;
        let entries = self.read_entries(header, &strings)?;

        Ok(entries)
    }
}
