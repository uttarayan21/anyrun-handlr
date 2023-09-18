use anyrun_plugin::Match;
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

// pub trait Completions {
//     fn matches(input: impl AsRef<str>) -> Result<Vec<Match>>;
// }

// pub struct FileCompletions {
//     component: String,
// }

// impl Completions for FileCompletions {
//     fn matches(input: impl AsRef<str>) -> Result<Vec<Match>> {
//         todo!()
//     }
// }

pub struct FileHistoryCache {
    root: u64,
    component: String,
    parent: Option<u64>,
}
