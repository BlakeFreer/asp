use crate::op::Op;
use core::fmt;
use std::fmt::Write;

pub struct Program {
    pub ops: Vec<Op>,
}

impl Program {
    pub fn as_binary(&self) -> Vec<u8> {
        self.ops.iter().map(|o| o.to_binary()).collect()
    }
    pub fn as_text(&self) -> String {
        self.ops
            .iter()
            .map(|o| o.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn as_mif(&self) -> Result<String, fmt::Error> {
        let width = 8;
        let depth = 256;
        let len = self.ops.len();

        if len > depth {
            panic!("Program is too long!");
        }

        let mut s = String::new();
        writeln!(s, "WIDTH={width};")?;
        writeln!(s, "DEPTH={depth};")?;
        writeln!(s, "")?;
        writeln!(s, "ADDRESS_RADIX=UNS;")?;
        writeln!(s, "DATA_RADIX=BIN;")?;
        writeln!(s, "")?;
        writeln!(s, "CONTENT BEGIN")?;

        for (n, op) in self.ops.iter().enumerate() {
            writeln!(s, "\t{n}\t:\t{:08b};", op.to_binary())?;
        }

        match len {
            x if x == depth => {}
            x if x == depth - 1 => writeln!(s, "\t255\t:\t{:08b}", 0)?,
            _ => writeln!(s, "\t[{}..{}]\t:\t{:08b};", self.ops.len(), depth - 1, 0)?,
        }
        writeln!(s, "END;")?;
        Ok(s)
    }
}
