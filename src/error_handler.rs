use std::path::Path;

pub struct Error<'a> {
    pub input_file: &'a Path,
    pub line_nr: usize,
    pub error_string: &'a str,
}

use owo_colors::OwoColorize;

pub fn error(err: Error) {
    eprintln!("{}:{}:?: {} {}", err.input_file.to_str().unwrap().bold(), err.line_nr.bold(), "error:".red().bold(), err.error_string);
    panic!()
}

