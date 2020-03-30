mod reader;

use reader::Reader;

use std::error;
use std::fmt;
use std::fs::File;

use clap::{App, Arg};

pub enum Error {
    InvalidInput,
    Io,
    InvalidHeader,
}

static ERROR_INVALID_ARGUMENT: &str = "Missing argument for .arz file path! Cannot continue.";
static ERROR_IO: &str = "Failed to open the given file for reading.";
static ERROR_INVALID_HEADER: &str =
    "Invalid file header, cannot read the given file as an ARZ database!";

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidInput => write!(f, "{}", ERROR_INVALID_ARGUMENT),
            Error::Io => write!(f, "{}", ERROR_IO),
            Error::InvalidHeader => write!(f, "{}", ERROR_INVALID_HEADER),
        }
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidInput => write!(f, "{}", ERROR_INVALID_ARGUMENT),
            Error::Io => write!(f, "{}", ERROR_IO),
            Error::InvalidHeader => write!(f, "{}", ERROR_INVALID_HEADER),
        }
    }
}

fn main() -> Result<(), Error> {
    let matches = App::new("grimarz")
        .version("0.1.0")
        .about("Grim Dawn Database File Extractor (Rust edition)")
        .author("evpobr <evpobr@gmail.com>")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();
    let input = matches.value_of("INPUT").ok_or(Error::InvalidInput)?;
    let file = File::open(input).map_err(|_| Error::Io)?;
    let _reader = Reader::new(file).map_err(|_| Error::InvalidHeader)?;
    Ok(())
}
