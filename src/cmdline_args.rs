use clap::Parser;
use std::{
    fs::File,
    io::{stdin, BufRead, BufReader, Write, BufWriter, stdout, self},
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input file name
    pub input_file: Option<PathBuf>,

    /// Output file name
    #[arg(short, long, value_name = "output")]
    pub output_file: Option<PathBuf>,
}

pub type ReadWriteResult = Result<(Box<dyn BufRead>, Box<dyn Write>),io::Error>;

pub fn get_read_write(args: &Args) -> ReadWriteResult {
    let input: Box<dyn BufRead> = match args.input_file.as_ref() {
        Some(name) => Box::new(BufReader::new(File::open(name)?)),
        None => Box::new(BufReader::new(stdin())),
    };

    let output: Box<dyn Write> = match args.output_file.as_ref() {
        Some(name) => Box::new(BufWriter::new(File::open(name)?)),
        None => Box::new(BufWriter::new(stdout())),
    };

    Ok((input, output))
}
