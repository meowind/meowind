pub mod lexing;

pub struct Location {
    pub ln: usize,
    pub start_col: usize,
    pub end_col: usize,
}

impl Location {
    pub fn new(ln: usize, start_col: usize, end_col: usize) -> Location {
        Location {
            ln,
            start_col,
            end_col,
        }
    }
}
