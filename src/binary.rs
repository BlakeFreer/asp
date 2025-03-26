use std::{fmt::Display, fs::File, io::Read};

use crate::{
    op::{InvalidOpcode, Op},
    Program,
};

#[derive(Debug)]
enum BinaryFileError {
    ReadError,
    BinaryError(InvalidOpcode, usize),
}

impl Display for BinaryFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryFileError::ReadError => write!(f, "Failed to read file."),
            BinaryFileError::BinaryError(e, position) => {
                write!(f, "Error at 0x{position:04x}: {e}")
            }
        }
    }
}

pub fn parse_file(file: File) -> Option<Program> {
    fn inner(mut file: File) -> Result<Program, BinaryFileError> {
        use BinaryFileError::*;

        let mut contents = vec![];
        file.read_to_end(&mut contents).or(Err(ReadError))?;

        contents
            .iter()
            .enumerate()
            .map(|(idx, b)| (*b).try_into().map_err(|e| BinaryError(e, idx)))
            .collect::<Result<Vec<Op>, BinaryFileError>>()
            .map(|ops| Program { ops })
    }

    match inner(file) {
        Ok(program) => Some(program),
        Err(e) => {
            println!("{e}");
            None
        }
    }
}
