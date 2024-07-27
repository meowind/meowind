use unicode_segmentation::UnicodeSegmentation;

use crate::utils::colors::*;
use core::fmt;
use std::path::PathBuf;

pub struct MeowindErrorContext {
    pub ln: usize,
    pub ln_content: String,
    pub start_col: usize,
    pub end_col: usize,
    pub source_path: PathBuf,
}

impl MeowindErrorContext {
    pub fn new(
        ln: usize,
        ln_content: String,
        start_col: usize,
        end_col: usize,
        source_path: PathBuf,
    ) -> MeowindErrorContext {
        MeowindErrorContext {
            ln,
            ln_content,
            start_col,
            end_col,
            source_path,
        }
    }

    pub fn body(&self, extends: usize) -> String {
        let content_graphemes: Vec<&str> = self.ln_content.trim().graphemes(true).collect();

        let start_idx = self.start_col - extends.min(self.start_col);
        let end_idx = (self.end_col + extends).min(content_graphemes.len());
        let mut ln_content = content_graphemes[start_idx..end_idx].to_owned();

        let highlight_end_idx = self.end_col - start_idx - 1;
        ln_content.insert(highlight_end_idx, GRAY);
        ln_content.insert(highlight_end_idx, RESET);

        let highlight_start_idx = self.start_col - start_idx - 1;
        ln_content.insert(highlight_start_idx, WHITE);
        ln_content.insert(highlight_start_idx, UNDERLINE);

        if start_idx > 0 {
            ln_content.insert(0, "... ");
        }

        if end_idx < content_graphemes.len() {
            ln_content.push(" ...");
        }

        ln_content.insert(0, GRAY);
        ln_content.push(RESET);

        let ln_content = ln_content.join("");
        return format!(
            "{BOLD}{}:({}, {}){RESET}: {ln_content}",
            self.source_path.display(),
            self.ln,
            self.start_col
        );
    }
}

impl fmt::Display for MeowindErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.body(20))
    }
}
