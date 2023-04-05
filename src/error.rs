use std::{fmt::Display, ops::{Range, Add}, path::Path, process, cmp::{min, max}};
use thiserror::Error;

use crate::{lexer::LexerErr, parser::ParseErr, resolver::ResolveErr};

#[derive(Debug, Error)]
pub struct ContextError<'a> {
    inner: Error,
    context: &'a FileContext<'a>,
}

impl Display for ContextError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let span = &self.inner.span;
        let content = self.context.content(span);

        use owo_colors::OwoColorize;
        write!(
            f,
            "{}:{}:{}..{}: {}: {}: {}",
            self.context.file_name.bold(),
            (span.line + 1).bold(),
            (span.chars.start + 1).bold(),
            (span.chars.end).bold(),
            "error".bold().red(),
            self.inner,
            content
        )
    }
}

impl ContextError<'_> {
    pub fn throw(&self) -> ! {
        eprintln!("{}", self);
        process::exit(1)
    }
}

#[derive(Debug, Error)]
#[error("{kind}")]
pub struct Error {
    kind: ErrorKind,
    span: Span,
}

impl Error {
    pub fn with_ctx<'a>(self, context: &'a FileContext) -> ContextError<'a> {
        ContextError {
            inner: self,
            context,
        }
    }

    pub fn throw_with_ctx(self, context: &FileContext) -> ! {
        let ctx_err = self.with_ctx(context);
        ctx_err.throw()
    }
}

#[derive(Debug, Clone)]
pub struct Span {
    line: usize,
    chars: Range<usize>,
}

impl Span {
    pub fn new(line: usize, chars: Range<usize>) -> Self {
        Span { line, chars }
    }
}

impl Add for Span {
    type Output = Span;

    fn add(self, rhs: Self) -> Self::Output {
        let start = min(self.chars.start, rhs.chars.start);
        let end = max(self.chars.end, rhs.chars.end);
        Span::new(rhs.line, start..end)
    }
}

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error(transparent)]
    LexerErr(#[from] LexerErr),
    #[error(transparent)]
    ParseErr(#[from] ParseErr),
    #[error(transparent)]
    ResolveErr(#[from] ResolveErr),
}

pub trait WithSpan {
    fn with_span(self, span: Span) -> Error;
}

impl<T: Into<ErrorKind>> WithSpan for T {
    fn with_span(self, span: Span) -> Error {
        Error {
            kind: self.into(),
            span,
        }
    }
}

pub trait ErrorGroup {
    fn throw_all_with_ctx(self, context: &FileContext<'_>) -> !;
}

impl ErrorGroup for Vec<Error> {
    fn throw_all_with_ctx(self, context: &FileContext<'_>) -> ! {
        for err in self {
            let ctx_err = err.with_ctx(context);
            eprintln!("{}", ctx_err);
        }
        process::exit(1)
    }
}

pub trait ResultSplit<T> {
   fn result_split(self) -> Result<Vec<T>,Vec<Error>>; 
}

impl<T,I : Iterator<Item = Result<T, Error>>> ResultSplit<T> for I {
    fn result_split(self) -> Result<Vec<T>,Vec<Error>> {
        let (ok,err) : (Vec<_>, Vec<_>) = self.partition(Result::is_ok);

        if !err.is_empty() {
            let ok = ok.into_iter().map(|t| t.ok().unwrap()).collect();
            Ok(ok)
        }
        else {
            let err = err.into_iter().map(|t| t.err().unwrap()).collect();
            Err(err)
        }
    }
}

#[derive(Debug)]
pub struct FileContext<'a> {
    file_name: String,
    file_content: &'a str,
}

impl<'a> FileContext<'a> {
    pub fn new(path: Option<&Path>, file_content: &'a str) -> Self {
        FileContext {
            file_name: path
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or("stdin".to_string()),
            file_content,
        }
    }

    fn content(&self, span: &Span) -> &str {
        self.file_content
            .lines()
            .nth(span.line)
            .and_then(|line| line.get(span.chars.clone()))
            .expect("Something is wrong...")
    }
}
