use unicode_segmentation::UnicodeSegmentation;

use crate::{
    source::{SourceFile, SourcePoint, SourceSpan},
    utils::colors::*,
};

use pad::{Alignment, PadStr};

#[derive(Clone)]
pub struct ErrorContext<'a> {
    pub kind: ErrorContextKind,
    pub src: SourceFile<'a>,
}

#[derive(Clone)]
pub enum ErrorContextKind {
    Point(SourcePoint),
    Span(SourceSpan),
}

impl<'a> ErrorContext<'a> {
    pub fn new(kind: ErrorContextKind, src: SourceFile<'a>) -> Self {
        Self { kind, src }
    }

    pub fn point(ln: usize, col: usize, src: SourceFile<'a>) -> Self {
        Self {
            kind: ErrorContextKind::Point(SourcePoint { ln, col }),
            src,
        }
    }

    pub fn span(start: SourcePoint, end: SourcePoint, src: SourceFile<'a>) -> Self {
        Self {
            kind: ErrorContextKind::Span(SourceSpan { start, end }),
            src,
        }
    }

    pub fn body(&self, extends: usize) -> String {
        match &self.kind {
            ErrorContextKind::Point(point) => self.point_body(extends, point),
            ErrorContextKind::Span(span) => self.span_body(extends, span),
        }
    }

    fn point_body(&self, extends: usize, point: &SourcePoint) -> String {
        let ln_text = self.src.lines[point.ln - 1];
        let graphemes: Vec<&str> = ln_text.graphemes(true).collect();

        let start_idx = point.col - extends.min(point.col);
        let end_idx = (point.col + extends).min(graphemes.len());

        let ln_text = graphemes[start_idx..end_idx].join("").to_owned();
        let text_count_before_trim = ln_text.graphemes(true).count();
        let mut ln_text = ln_text.trim().to_owned();
        let trim_diff = text_count_before_trim - ln_text.graphemes(true).count();

        let ellipsis_start = format!("{GRAY}...{RESET} ");
        let ellipsis_end = format!(" {GRAY}...{RESET}");

        let mut point_col = point.col - start_idx - trim_diff;
        if start_idx > 0 {
            point_col += 4;
            ln_text.insert_str(0, ellipsis_start.as_str());
        }

        if end_idx < graphemes.len() {
            ln_text.push_str(ellipsis_end.as_str());
        }

        let loc = self.loc(point.ln, point.col);
        let loc_count = loc.graphemes(true).count();

        let point_text = "HERE ^";
        let point_body = point_text.pad(loc_count + point_col - 5, ' ', Alignment::Right, false);

        return format!("{}: {ln_text}\n{CYAN}{BOLD}{point_body}{RESET}", loc);
    }

    fn span_body(&self, extends: usize, span: &SourceSpan) -> String {
        let mut lines_graphemes: Vec<Vec<&str>> = self
            .src
            .lines
            .iter()
            .map(|ln| ln.graphemes(true).collect())
            .collect();

        let end_ln_len = lines_graphemes[span.end.ln - 1].len();

        let start_idx = span.start.col - extends.min(span.start.col);
        let end_idx = (span.end.col + extends).min(end_ln_len);

        lines_graphemes[span.start.ln - 1].drain(0..=start_idx);
        lines_graphemes[span.end.ln - 1].drain(end_idx..end_ln_len);

        let highlight_start_idx = span.start.col - start_idx - 1;
        let highlight_end_idx = if span.start.ln == span.end.ln {
            span.end.col - start_idx - 1
        } else {
            span.end.col - 1
        };

        lines_graphemes[span.end.ln - 1].insert(highlight_end_idx, GRAY);
        lines_graphemes[span.end.ln - 1].insert(highlight_end_idx, RESET);

        lines_graphemes[span.start.ln - 1].insert(highlight_start_idx, WHITE);
        lines_graphemes[span.start.ln - 1].insert(highlight_start_idx, UNDERLINE);

        let mut text = lines_graphemes[(span.start.ln - 1)..(span.end.ln)]
            .iter()
            .map(|ln| ln.join(""))
            .collect::<Vec<String>>()
            .join("\n")
            .trim()
            .to_owned();

        if start_idx > 0 {
            text.insert_str(0, "... ");
        }

        if end_idx < end_ln_len {
            text.push_str(" ...");
        }

        text.insert_str(0, GRAY);
        text.push_str(RESET);

        return format!("{}: {text}", self.loc(span.start.ln, span.start.col));
    }

    fn loc(&self, ln: usize, col: usize) -> String {
        format!("{BOLD}{}:({}, {}){RESET}", self.src.path.display(), ln, col)
    }
}

impl ToString for ErrorContext<'_> {
    fn to_string(&self) -> String {
        self.body(20)
    }
}
