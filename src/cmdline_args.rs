use clap::Parser;
use std::{
    fs::File,
    io::{self, stdin, stdout, BufRead, BufReader, BufWriter, Write},
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

    /// Config file
    #[arg(short, long, value_name = "config")]
    pub config_file: Option<PathBuf>,

    /// Output to binary file
    #[arg(short, long, default_value_t = false)]
    pub text: bool,
}

pub type ReadWriteResult = Result<(Box<dyn BufRead>, Box<dyn Write>), io::Error>;

impl Args {
    pub fn get_read_write(args: &Args) -> ReadWriteResult {
        let input: Box<dyn BufRead> = match args.input_file.as_ref() {
            Some(name) => Box::new(BufReader::new(File::open(name)?)),
            None => Box::new(BufReader::new(stdin())),
        };

        let output: Box<dyn Write> = match args.output_file.as_ref() {
            Some(name) => Box::new(BufWriter::new(File::create(name)?)),
            None => Box::new(BufWriter::new(stdout())),
        };

        Ok((input, output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Read, Seek, Write};
    use tempfile::NamedTempFile;

    fn create_temp_file_with_content(content: &str) -> io::Result<NamedTempFile> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "{}", content)?;
        temp_file.seek(io::SeekFrom::Start(0))?;
        Ok(temp_file)
    }

    #[test]
    fn test_get_read_write() -> io::Result<()> {
        let input_content = "Sample content";
        let input_file = create_temp_file_with_content(input_content)?;
        let input_path = input_file.path().to_path_buf();

        let output_file = NamedTempFile::new()?;
        let output_path = output_file.path().to_path_buf();

        let args = Args {
            input_file: Some(input_path),
            output_file: Some(output_path),
            config_file: None,
            text: true,
        };

        let (mut input, _) = Args::get_read_write(&args).unwrap();
        let mut input_content_result = String::new();
        input.read_to_string(&mut input_content_result)?;

        assert_eq!(input_content, input_content_result.trim());

        Ok(())
    }
}
