use std::path::PathBuf;

pub static DEFAULT_SRC_CONTENTS: String = String::new();

pub struct SourceFile<'a> {
    pub path: PathBuf,
    pub contents: &'a String,
    pub chars: Vec<char>,
    pub lines: Vec<&'a str>,
}

impl<'a> SourceFile<'a> {
    pub fn new(path: PathBuf, contents: &'a String) -> SourceFile<'a> {
        SourceFile {
            path,
            chars: contents.chars().collect(),
            lines: contents.split("\n").collect(),
            contents,
        }
    }
}

impl Default for SourceFile<'_> {
    fn default() -> Self {
        SourceFile::new(PathBuf::new(), &DEFAULT_SRC_CONTENTS)
    }
}

impl Clone for SourceFile<'_> {
    fn clone(&self) -> Self {
        SourceFile::new(self.path.clone(), self.contents)
    }
}

#[derive(Clone, Debug)]
pub struct SourcePoint {
    pub ln: usize,
    pub col: usize,
}

impl SourcePoint {
    pub fn new(ln: usize, col: usize) -> Self {
        Self { ln, col }
    }
}

#[derive(Clone, Debug)]
pub struct SourceSpan {
    pub start: SourcePoint,
    pub end: SourcePoint,
}

impl SourceSpan {
    pub fn new(start: SourcePoint, end: SourcePoint) -> Self {
        Self { start, end }
    }

    pub fn one_ln(ln: usize, start: usize, end: usize) -> Self {
        Self::new(SourcePoint::new(ln, start), SourcePoint::new(ln, end))
    }

    pub fn mul_ln(start_ln: usize, start_col: usize, end_ln: usize, end_col: usize) -> Self {
        Self::new(
            SourcePoint::new(start_ln, start_col),
            SourcePoint::new(end_ln, end_col),
        )
    }
}
