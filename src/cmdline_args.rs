use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input file name
    pub input_file: Option<PathBuf>,

    /// Output file name
    #[arg(short, long, value_name = "output")]
    pub output_file: Option<PathBuf>,
}

pub fn get_io_files(args: &Args) -> (PathBuf, PathBuf) {
    let input_file = match args.input_file.clone() {
        Some(name) => name,
        // TODO: Error handling rather than panic
        None => panic!(),
    };

    let output_path = args.output_file.clone().unwrap_or(PathBuf::from("a.out"));

    (input_file, output_path)
}
