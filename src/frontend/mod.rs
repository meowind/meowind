pub mod lexing;

pub struct Loc {
    pub ln: usize,
    pub start_col: usize,
    pub end_col: usize,
}

impl Loc {
    pub fn new(ln: usize, start_col: usize, end_col: usize) -> Loc {
        Loc {
            ln,
            start_col,
            end_col,
        }
    }
}
