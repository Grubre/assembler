use thiserror::Error;

use crate::{
    error::{Error, ResultSplit, WithSpan},
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

pub fn resolve_all_labels(
    labels: &Labels,
    unresolved: Vec<Unresolved>,
) -> Result<Vec<String>, Vec<Error>> {
    let ok = unresolved
        .into_iter()
        .map(|unr| resolve_label(labels, unr))
        .result_split()?;

    Ok(ok)
}

#[cfg(test)]
mod tests {
    use crate::token::Span;

    use super::*;
    use std::collections::HashMap;

    fn create_test_labels() -> Labels {
        let mut labels_map = HashMap::new();
        labels_map.insert("label1".to_string(), 5);
        labels_map.insert("label2".to_string(), 10);
        Labels(labels_map)
    }

    #[test]
    fn test_resolve_label_known() {
        let labels = create_test_labels();
        let unresolved = Unresolved::LabelRef("label1".to_string(), Span::new(0, 0..6));
        let result = resolve_label(&labels, unresolved).unwrap();
        assert_eq!(result, "00000101");
    }

    #[test]
    fn test_resolve_label_unknown() {
        let labels = create_test_labels();
        let unresolved = Unresolved::LabelRef("unknown".to_string(), Span::new(0, 0..6));
        let result = resolve_label(&labels, unresolved);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_label_value() {
        let labels = create_test_labels();
        let unresolved = Unresolved::Value("01010101".to_string());
        let result = resolve_label(&labels, unresolved).unwrap();
        assert_eq!(result, "01010101");
    }

    #[test]
    fn test_resolve_all_labels() {
        let labels = create_test_labels();
        let unresolved = vec![
            Unresolved::LabelRef("label1".to_string(), Span::new(0, 0..6)),
            Unresolved::Value("01010101".to_string()),
            Unresolved::LabelRef("label2".to_string(), Span::new(0, 7..13)),
        ];
        let result = resolve_all_labels(&labels, unresolved).unwrap();
        assert_eq!(result, vec!["00000101", "01010101", "00001010"]);
    }
}
