use crate::{
    imm::{Imm, ImmType},
    op::Op,
    program::Program,
    reg::Reg,
};
use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
};

struct Line {
    string: String,
    pub lineno: usize,
}
struct LinePreprocessed {
    string: String,
    pub lineno: usize,
}

struct Tokenized<'a> {
    mnenomic: &'a str,
    tokens: Vec<&'a str>,
}

impl Line {
    fn preprocess(self) -> Option<LinePreprocessed> {
        // remove comment
        let cut = self.string.find(';').unwrap_or(self.string.len());
        let string = self.string[0..cut].trim().to_owned();

        if string.is_empty() {
            None
        } else {
            Some(LinePreprocessed {
                string,
                lineno: self.lineno,
            })
        }
    }
}

impl LinePreprocessed {
    fn tokenize<'a>(&'a self) -> Tokenized<'a> {
        let mut parts = self.string.splitn(2, ' ');
        let mnenomic = parts.next().expect("Shouldn't be empty after trim.");
        let tokens = parts
            .next()
            .map(|t| {
                t.split([',', ' '])
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .collect()
            })
            .unwrap_or_else(|| vec![]);
        Tokenized::<'a> { mnenomic, tokens }
    }
}

#[derive(Debug, PartialEq)]
enum AsmError {
    InvalidMnenomic(String),
    MissingImmediate,
    InvalidImmediate(String),
    ImmediateOutOfRange(i32),
    MissingRegister,
    InvalidRegister(String),
    ExtraToken(String),
}

impl Display for AsmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsmError::InvalidMnenomic(x) => write!(f, "Invalid mnenomic \"{x}\"."),
            AsmError::MissingImmediate => write!(f, "Missing an immediate."),
            AsmError::InvalidImmediate(x) => write!(f, "Invalid immediate \"{x}\"."),
            AsmError::ImmediateOutOfRange(x) => write!(f, "Immediate {x} is out of range."),
            AsmError::MissingRegister => write!(f, "Missing a register."),
            AsmError::InvalidRegister(x) => write!(f, "Invalid register \"{x}\"."),
            AsmError::ExtraToken(x) => write!(f, "Unexpected token \"{x}\"."),
        }
    }
}

impl AsmError {
    fn on_line(self, line: usize) -> AsmLineError {
        AsmLineError(self, line)
    }
}

struct AsmLineError(AsmError, usize);

impl Display for AsmLineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}: {}", self.1, self.0)
    }
}

fn parse_line(line: &LinePreprocessed) -> Result<Op, AsmError> {
    use AsmError::*;

    fn get_imm<'a, T, const N: u8>(
        tokens: &mut impl Iterator<Item = &'a str>,
    ) -> Result<Imm<T, N>, AsmError>
    where
        T: ImmType<N> + Copy,
    {
        let imm = tokens.next().ok_or(MissingImmediate)?;
        let imm = imm.strip_prefix('#').unwrap_or(imm);

        // Two steps are required since .parse<T> doesn't distinguish
        // between invalid and out of range.
        let val: i32 = imm.parse().or(Err(InvalidImmediate(imm.to_string())))?;
        let v: T = val.try_into().or(Err(ImmediateOutOfRange(val)))?;
        v.try_into().or(Err(ImmediateOutOfRange(val)))
    }

    fn get_reg<'a>(tokens: &mut impl Iterator<Item = &'a str>) -> Result<Reg, AsmError> {
        let reg = tokens.next().ok_or(MissingRegister)?;
        reg.try_into().or(Err(InvalidRegister(reg.to_string())))
    }

    let tokenized = line.tokenize();
    let mut tokens = tokenized.tokens.into_iter();

    let op = match tokenized.mnenomic {
        "BR" => Op::BR(get_imm(&mut tokens)?),
        "BRZ" => Op::BRZ(get_imm(&mut tokens)?),
        "ADDI" => Op::ADDI(get_reg(&mut tokens)?, get_imm(&mut tokens)?),
        "SUBI" => Op::SUBI(get_reg(&mut tokens)?, get_imm(&mut tokens)?),
        "SR0" => Op::SR0(get_imm(&mut tokens)?),
        "SRH0" => Op::SRH0(get_imm(&mut tokens)?),
        "CLR" => Op::CLR(get_reg(&mut tokens)?),
        "MOVA" => Op::MOVA(get_reg(&mut tokens)?),
        "MOVR" => Op::MOVR(get_reg(&mut tokens)?),
        "MOVRHS" => Op::MOVRHS(get_reg(&mut tokens)?),
        "MOV" => Op::MOV(get_reg(&mut tokens)?, get_reg(&mut tokens)?),
        "PAUSE" => Op::PAUSE,
        x => return Err(InvalidMnenomic(x.to_string())),
    };

    if let Some(t) = tokens.next() {
        return Err(ExtraToken(t.to_string()));
    }

    Ok(op)
}

pub fn parse_file(file: File) -> Option<Program> {
    fn inner(file: File) -> Result<Program, Vec<AsmLineError>> {
        let reader = BufReader::new(file);

        let preprocessed = reader
            .lines()
            .filter_map(|l| l.ok())
            .enumerate()
            .map(|(n, l)| Line {
                string: l,
                lineno: n + 1, // file lineno start at 1
            })
            .filter_map(|l| l.preprocess());

        let mut errors: Vec<AsmLineError> = vec![];

        // convert to Ops and record all errors along the way
        let ops = preprocessed
            .filter_map(|l| {
                parse_line(&l)
                    .map_err(|e| errors.push(e.on_line(l.lineno)))
                    .ok()
            })
            .collect();

        // Only create a program if there are no errors
        if errors.is_empty() {
            Ok(Program { ops })
        } else {
            Err(errors)
        }
    }

    match inner(file) {
        Ok(program) => Some(program),
        Err(errs) => {
            for e in errs {
                println!("{}", e);
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::imm::{I5, U3, U4};

    use super::*;

    impl PartialEq for LinePreprocessed {
        fn eq(&self, other: &Self) -> bool {
            self.string == other.string
        }
    }

    #[test]
    fn test_preprocess() {
        let cases = [
            ("PAUSE", Some("PAUSE")),
            ("BR ; remove comment", Some("BR")),
            ("  ADDI; trim", Some("ADDI")),
            ("  ; empty", None),
        ];
        for (test, expected) in cases {
            let l = Line {
                string: test.to_owned(),
                lineno: 0,
            }
            .preprocess();

            if let Some(l) = l {
                assert!(expected.is_some());
                assert_eq!(l.string, expected.unwrap());
            } else {
                assert!(l.is_none());
            }
        }
    }

    #[test]
    fn test_asm() {
        use AsmError::*;
        use Reg::*;
        // don't need to test empty strings since they should be filtered out
        let cases: Vec<(&str, Result<Op, AsmError>)> = vec![
            ("PAUSE", Ok(Op::PAUSE)),
            ("ADDI r3, 7", Ok(Op::ADDI(R3, U3::new(7).unwrap()))),
            ("ADDI r3, 8", Err(ImmediateOutOfRange(8))),
            ("BR -14", Ok(Op::BR(I5::new(-14).unwrap()))),
            ("BRZ 2", Ok(Op::BRZ(I5::new(2).unwrap()))),
            ("MOV r3r2", Err(InvalidRegister("r3r2".to_string()))),
            ("MOV r3,    r2", Ok(Op::MOV(R3, R2))),
            ("SRH0", Err(MissingImmediate)),
            ("SRH0 1", Ok(Op::SRH0(U4::new(1).unwrap()))),
            ("SRH0 #1", Ok(Op::SRH0(U4::new(1).unwrap()))),
            ("CLR r0, extra", Err(ExtraToken("extra".to_string()))),
            ("SR0 numbers", Err(InvalidImmediate("numbers".to_string()))),
            ("SBI", Err(InvalidMnenomic("SBI".to_string()))),
            ("CLR", Err(MissingRegister)),
        ];

        for (line, result) in cases {
            let l = Line {
                string: line.to_string(),
                lineno: 0,
            }
            .preprocess()
            .unwrap();
            assert_eq!(parse_line(&l), result);
        }
    }
}
