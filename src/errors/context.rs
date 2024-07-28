use unicode_segmentation::UnicodeSegmentation;

use crate::{frontend::Loc, utils::colors::*};
use core::fmt;
use std::path::PathBuf;

pub struct MeowindErrorContext {
    pub loc: Loc,
    pub ln_text: String,
    pub src_path: PathBuf,
}

impl MeowindErrorContext {
    pub fn new(loc: Loc, ln_text: String, src_path: PathBuf) -> MeowindErrorContext {
        MeowindErrorContext {
            loc,
            ln_text,
            src_path,
        }
    }

    pub fn body(&self, extends: usize) -> String {
        let content_graphemes: Vec<&str> = self.ln_text.graphemes(true).collect();

        let start_idx = self.loc.start_col - extends.min(self.loc.start_col);
        let end_idx = (self.loc.end_col + extends).min(content_graphemes.len());
        let mut ln_text = content_graphemes[start_idx..end_idx].to_owned();

        let highlight_end_idx = self.loc.end_col - start_idx - 1;
        ln_text.insert(highlight_end_idx, GRAY);
        ln_text.insert(highlight_end_idx, RESET);

        let highlight_start_idx = self.loc.start_col - start_idx - 1;
        ln_text.insert(highlight_start_idx, WHITE);
        ln_text.insert(highlight_start_idx, UNDERLINE);

        let mut ln_text = ln_text.join("").trim().to_owned();

        if start_idx > 0 {
            ln_text.insert_str(0, "... ");
        }

        if end_idx < content_graphemes.len() {
            ln_text.push_str(" ...");
        }

        ln_text.insert_str(0, GRAY);
        ln_text.push_str(RESET);

        return format!(
            "{BOLD}{}:({}, {}){RESET}: {ln_text}",
            self.src_path.display(),
            self.loc.ln,
            self.loc.start_col
        );
    }
}

impl fmt::Display for MeowindErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.body(20))
    }
}
