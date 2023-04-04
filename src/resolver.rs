use crate::parser::{Labels, Unresolved};

pub fn resolve_label(labels: &Labels, unresolved: &Unresolved) -> String {
    match unresolved {
        //TODO: convert to binary
        Unresolved::LabelRef(label) => format!("{:08b}",labels.0.get(label).unwrap()),
        Unresolved::Value(bin) => bin.clone(),
    }
}

pub fn resolve_all_labels(labels: &Labels, unresolved: Vec<Unresolved>) -> Vec<String> {
    unresolved
        .iter()
        .map(|unr| resolve_label(labels, unr))
        .collect()
}