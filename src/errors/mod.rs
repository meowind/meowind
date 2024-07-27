use std::process;

pub mod command_line;
pub mod context;
pub mod syntax;

pub struct MeowindErrorList<T: MeowindError> {
    pub errors: Vec<T>,
}

impl<T: MeowindError> MeowindErrorList<T> {
    pub fn new() -> MeowindErrorList<T> {
        MeowindErrorList { errors: Vec::new() }
    }

    pub fn push(&mut self, error: T) {
        self.errors.push(error);
    }

    pub fn throw_if_there(&self) {
        if self.errors.len() == 0 {
            return;
        }

        for error in &self.errors {
            println!("{}", error.to_string());
        }

        process::exit(1);
    }
}

pub trait MeowindError {
    fn to_string(&self) -> String;
}

pub fn throw(error: impl MeowindError) {
    let error_body = error.to_string();
    println!("{error_body}");

    process::exit(1);
}
