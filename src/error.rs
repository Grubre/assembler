use owo_colors::OwoColorize;
use std::{
    error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    io,
    path::PathBuf,
};

use crate::lexer::LexerErr;

#[derive(Debug)]
pub struct InnerError<E> {
    error: E,
    info: SrcFileInfo,
}

impl From<LexerErr> for InnerError<LexerErr> {
    fn from(value: LexerErr) -> Self {
        match value {
            LexerErr::LabelParseError(_, loc)
            | LexerErr::NumberParseError(_, loc)
            | LexerErr::UnknownToken(_, loc) => InnerError {
                error: value,
                info: SrcFileInfo::new_with_loc(loc),
            },
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Io(InnerError<io::Error>),
    // Config(ConfigErr),
    Lexer(InnerError<LexerErr>),
}

impl Error {
    pub fn with_filename(self, pb: PathBuf) -> Self {
        match self {
            Error::Io(inner) => Error::Io(InnerError {
                error: inner.error,
                info: inner.info.with_filename(pb),
            }),
            Error::Lexer(inner) => Error::Lexer(InnerError {
                error: inner.error,
                info: inner.info.with_filename(pb),
            }),
        }
    }
}

fn space(info: &SrcFileInfo) -> &str {
    if info.file.is_some() || info.loc.is_some() {
        " "
    } else {
        ""
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        return match &self {
            Error::Io(inner) => {
                f.write_fmt(format_args!("{}{}{}: I/O error: {}", inner.info, space(&inner.info), "error".red().bold(), inner.error))
            },
            Error::Lexer(inner) => {
                f.write_fmt(format_args!("{}{}{}: {}", inner.info, space(&inner.info), "error".red().bold(), inner.error.white().bold()))
            },
        };
    }
}

impl error::Error for Error {}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct SrcFileLoc {
    line: usize,
    column: usize,
}

impl SrcFileLoc {
    pub fn at(line: usize, column: usize) -> Self {
        SrcFileLoc { line, column }
    }
}

impl Display for SrcFileLoc {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_fmt(format_args!("{}:{}", self.line, self.column))
    }
}

#[derive(Debug)]
struct SrcFileInfo {
    loc: Option<SrcFileLoc>,
    file: Option<PathBuf>,
}

impl SrcFileInfo {
    pub fn new_with_loc(loc: SrcFileLoc) -> Self {
        Self {
            loc: Some(loc),
            file: None,
        }
    }

    pub fn new(file: PathBuf, loc: SrcFileLoc) -> Self {
        Self {
            loc: Some(loc),
            file: Some(file),
        }
    }
}

impl SrcFileInfo {
    pub fn with_filename(self, file: PathBuf) -> Self {
        Self {
            loc: self.loc,
            file: Some(file),
        }
    }
}

impl Default for SrcFileInfo {
    fn default() -> Self {
        Self {
            loc: None,
            file: None,
        }
    }
}

impl Display for SrcFileInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(file) = &self.file {
            f.write_fmt(format_args!("{}:", file.display()))?;
        }

        if let Some(loc) = &self.loc {
            f.write_fmt(format_args!("{}:", loc))?;
        }

        Ok(())
    }
}

pub trait CustomizeResult {
    type Type;

    fn with_filename(self, path: PathBuf) -> Result<Self::Type, Error>;
}

impl<T> CustomizeResult for Result<T, Error> {
    type Type = T;

    fn with_filename(self, path: PathBuf) -> Self {
        if let Err(err) = self {
            Err(err.with_filename(path))
        } else {
            self
        }
    }
}

pub trait MapError {
    type Type;
    fn map_error(self) -> Result<Self::Type, Error>;
}

impl<T> MapError for Result<T, LexerErr> {
    type Type = T;

    fn map_error(self) -> Result<T, Error> {
        match self {
            Ok(t) => Ok(t),
            Err(err) => Err(Error::Lexer(err.into())),
        }
    }
}
