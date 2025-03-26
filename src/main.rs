use std::process::ExitCode;
use std::{fs::File, io::Write};

use asp::{assembly, binary};
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
enum OutputFmt {
    ASM,
    HEX,
    MIF,
}
impl OutputFmt {
    fn ext(&self) -> &str {
        match self {
            OutputFmt::ASM => "s",
            OutputFmt::HEX => "hex",
            OutputFmt::MIF => "mif",
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Cli {
    file: String,

    #[arg(short, long="fmt", value_enum, default_value_t=OutputFmt::MIF, help="Output format.")]
    format: OutputFmt,

    #[arg(short, long, help = "Output filename, by default out.<fmt>")]
    output: Option<String>,

    #[arg(short = 'H', long, help = "Input file is machine code in a hex file.")]
    hex: bool,

    #[arg(short, long)]
    verbose: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let Ok(file) = File::open(&cli.file) else {
        println!("Failed to open {}", cli.file);
        return ExitCode::from(2);
    };

    let program = match cli.hex {
        false => assembly::parse_file(file),
        true => binary::parse_file(file),
    };

    let Some(program) = program else {
        println!("Exiting due to errors.");
        return ExitCode::from(1);
    };

    if cli.verbose {
        println!("---- Assembly ----");
        println!("{}", program.as_text());

        println!("---- Machine Code ----");
        for op in program.as_binary() {
            println!("{:08b}", op)
        }
    }

    let outfilename = cli.output.unwrap_or(format!("out.{}", cli.format.ext()));

    let mut outfile = File::create(&outfilename).expect("Failed to create output file.");
    let contents = match cli.format {
        OutputFmt::ASM => program.as_text().as_bytes().to_vec(),
        OutputFmt::HEX => program.as_binary().to_vec(),
        OutputFmt::MIF => program.as_mif().unwrap().as_bytes().to_vec(),
    };

    return match outfile.write(&contents) {
        Ok(_) => {
            println!("Output saved to {outfilename}");
            ExitCode::from(0)
        }
        Err(_) => {
            println!("Failed to save output.");
            ExitCode::from(1)
        }
    };
}
