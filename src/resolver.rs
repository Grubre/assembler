use thiserror::Error;

use crate::{
    error::{Error, WithSpan},
    parser::{Labels, Unresolved},
};

#[derive(Debug, Error)]
pub enum ResolveErr {
    #[error("Unknown label")]
    UnknownLabel,
}

pub fn resolve_label(labels: &Labels, unresolved: Unresolved) -> Result<String, Error> {
    match unresolved {
        //TODO: convert to binary
        Unresolved::LabelRef(label, span) => labels
            .0
            .get(&label)
            .ok_or(ResolveErr::UnknownLabel.with_span(span))
            .map(|value| format!("{:08b}", value)),
        Unresolved::Value(bin) => Ok(bin.clone()),
    }
}

pub fn resolve_all_labels(labels: &Labels, unresolved: Vec<Unresolved>) -> Result<Vec<String>,Vec<Error>> {
    let (ok,err) : (Vec<_>, Vec<_>) = unresolved
        .into_iter()
        .map(|unr| resolve_label(labels, unr))
        .partition(Result::is_ok);

    if !err.is_empty() {
        let err = err.into_iter().map(|arg| arg.unwrap_err()).collect();
        return Err(err);
    }

    Ok(ok.into_iter().map(|arg| arg.unwrap()).collect())  
}
