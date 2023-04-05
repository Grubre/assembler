use thiserror::Error;

use crate::{
    error::{Error, WithSpan, ResultSplit},
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
            .map(|value| format!("{:08b}", value))
            .ok_or(ResolveErr::UnknownLabel.with_span(span)),
        Unresolved::Value(bin) => Ok(bin),
    }
}

pub fn resolve_all_labels(labels: &Labels, unresolved: Vec<Unresolved>) -> Result<Vec<String>,Vec<Error>> {
    let ok = unresolved
        .into_iter()
        .map(|unr| resolve_label(labels, unr))
        .result_split()?;

    Ok(ok)  
}
