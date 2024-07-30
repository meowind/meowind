use unicode_segmentation::UnicodeSegmentation;

use crate::{
    frontend::{lexing::Token, Loc},
    structs::ScriptSource,
    utils::colors::*,
};
use core::{fmt, panic};
use std::path::PathBuf;

use pad::{Alignment, PadStr};

#[derive(Clone)]
pub struct ErrorContext {
    pub kind: ErrorContextKind,
    pub ln: usize,
    pub ln_text: String,
    pub src_path: PathBuf,
}

#[derive(Clone)]
pub enum ErrorContextKind {
    PointOnSpan { start: usize, end: usize },
    PointOnColumn(usize),
}

impl ErrorContext {
    pub fn new(
        kind: ErrorContextKind,
        ln: usize,
        ln_text: String,
        src_path: PathBuf,
    ) -> ErrorContext {
        ErrorContext {
            kind,
            ln,
            ln_text,
            src_path,
        }
    }

    pub fn body(&self, extends: usize) -> String {
        match self.kind {
            ErrorContextKind::PointOnSpan { start, end } => self.span_body(extends, start, end),
            ErrorContextKind::PointOnColumn(col) => self.col_body(extends, col),
        }
    }

    fn span_body(&self, extends: usize, start: usize, end: usize) -> String {
        let graphemes: Vec<&str> = self.ln_text.graphemes(true).collect();

        let start_idx = start - extends.min(start);
        let end_idx = (end + extends).min(graphemes.len());
        let mut ln_text = graphemes[start_idx..end_idx].to_owned();

        let highlight_end_idx = end - start_idx - 1;
        ln_text.insert(highlight_end_idx, GRAY);
        ln_text.insert(highlight_end_idx, RESET);

        let highlight_start_idx = start - start_idx - 1;
        ln_text.insert(highlight_start_idx, WHITE);
        ln_text.insert(highlight_start_idx, UNDERLINE);

        let mut ln_text = ln_text.join("").trim().to_owned();

        if start_idx > 0 {
            ln_text.insert_str(0, "... ");
        }

        if end_idx < graphemes.len() {
            ln_text.push_str(" ...");
        }

        ln_text.insert_str(0, GRAY);
        ln_text.push_str(RESET);

        return format!("{}: {ln_text}", self.loc(start));
    }

    fn col_body(&self, extends: usize, col: usize) -> String {
        let graphemes: Vec<&str> = self.ln_text.graphemes(true).collect();

        let start_idx = col - extends.min(col);
        let end_idx = (col + extends).min(graphemes.len());

        let ln_text = graphemes[start_idx..end_idx].join("").to_owned();
        let text_count_before_trim = ln_text.graphemes(true).count();
        let mut ln_text = ln_text.trim().to_owned();
        let trim_diff = text_count_before_trim - ln_text.graphemes(true).count();

        let ellipsis_start = format!("{GRAY}...{RESET} ");
        let ellipsis_end = format!(" {GRAY}...{RESET}");

        let mut point_col = col - start_idx - trim_diff;
        if start_idx > 0 {
            point_col += 4;
            ln_text.insert_str(0, ellipsis_start.as_str());
        }

        if end_idx < graphemes.len() {
            ln_text.push_str(ellipsis_end.as_str());
        }

        let loc = self.loc(col);
        let loc_count = loc.graphemes(true).count();

        let point_text = "HERE ^";
        let point_count = point_text.graphemes(true).count();
        let point_body = point_text.pad(loc_count + point_col - 6, ' ', Alignment::Right, false);

        return format!("{}: {ln_text}\n{CYAN}{BOLD}{point_body}{RESET}", loc);
    }

    fn loc(&self, col: usize) -> String {
        format!(
            "{BOLD}{}:({}, {}){RESET}",
            self.src_path.display(),
            self.ln,
            col
        )
    }
}

impl ToString for ErrorContext {
    fn to_string(&self) -> String {
        self.body(20)
    }
}

#[derive(Clone)]
pub struct ErrorContextBuilder {
    kind: Option<ErrorContextKind>,
    ln: Option<usize>,
    ln_text: Option<String>,
    src_path: Option<PathBuf>,
}

impl ErrorContextBuilder {
    pub fn col(col: usize) -> ErrorContextBuilder {
        ErrorContextBuilder {
            kind: Some(ErrorContextKind::PointOnColumn(col)),
            ..Default::default()
        }
    }

    pub fn span(start: usize, end: usize) -> ErrorContextBuilder {
        ErrorContextBuilder {
            kind: Some(ErrorContextKind::PointOnSpan { start, end }),
            ..Default::default()
        }
    }

    pub fn from_src_and_ln(&self, src: &ScriptSource, ln: usize) -> ErrorContextBuilder {
        ErrorContextBuilder {
            ln: Some(ln),
            ln_text: Some(src.lines[ln - 1].to_owned()),
            src_path: Some(src.path.clone()),
            ..self.clone()
        }
    }

    pub fn ln(&self, ln: usize) -> ErrorContextBuilder {
        ErrorContextBuilder {
            ln: Some(ln),
            ..self.clone()
        }
    }

    pub fn ln_text(&self, ln_text: String) -> ErrorContextBuilder {
        ErrorContextBuilder {
            ln_text: Some(ln_text),
            ..self.clone()
        }
    }

    pub fn src_path(&self, src_path: PathBuf) -> ErrorContextBuilder {
        ErrorContextBuilder {
            src_path: Some(src_path),
            ..self.clone()
        }
    }

    pub fn build(&self) -> ErrorContext {
        let Some(kind) = &self.kind else {
            panic!("cannot build ErrorContext without kind");
        };

        let Some(ln) = self.ln else {
            panic!("cannot build ErrorContext without line");
        };

        let Some(ln_text) = &self.ln_text else {
            panic!("cannot build ErrorContext without line text");
        };

        let Some(src_path) = &self.src_path else {
            panic!("cannot build ErrorContext without source path");
        };

        return ErrorContext::new(kind.clone(), ln, ln_text.clone(), src_path.clone());
    }
}

impl Default for ErrorContextBuilder {
    fn default() -> Self {
        Self {
            kind: None,
            ln: None,
            ln_text: None,
            src_path: None,
        }
    }
}
