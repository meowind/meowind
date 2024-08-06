use std::process;

pub mod command_line;
pub mod compiler;
pub mod context;
pub mod syntax;

impl<T: MeowindError> ErrorList for Vec<T> {
    fn throw_if_there(&self) {
        if self.len() == 0 {
            return;
        }

        for error in self {
            println!("{}", error.to_string());
        }

        process::exit(1);
    }
}

pub trait ErrorList {
    fn throw_if_there(&self);
}

pub trait MeowindError {
    fn to_string(&self) -> String;
}

pub fn throw(error: impl MeowindError) {
    let error_body = error.to_string();
    println!("{error_body}");

    process::exit(1);
}
