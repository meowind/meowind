#![feature(let_chains)]

pub mod errors;
pub mod frontend;
pub mod structs;
pub mod utils;

use crate::{
    errors::command_line::{CommandLineError, CommandLineErrorKind},
    frontend::lexing::lexer::Lexer,
    structs::{MeowindArguments, MeowindScriptSource},
    utils::{debug::Debugger, string::*},
};
use std::{cell::RefCell, env, fs, io::ErrorKind, path::PathBuf, process};

fn main() {
    let args = parse_arguments();

    let source_contents = read_source_contents(&args.path);
    let source = MeowindScriptSource::new(args.path.clone(), &source_contents);

    println!(
        "{GREEN}{BOLD}compiling{WHITE} {}{RESET}",
        args.path.display()
    );

    let mut debugger = Debugger::new(args.path.clone(), true, false);
    let debugger = &RefCell::new(&mut debugger);

    let mut lexer = Lexer::new(source, &debugger);
    let (tokens, errors) = lexer.tokenize();
    errors.throw_if_there();

    info!(&debugger, "== OUTPUT TOKENS ==");

    #[cfg(debug_assertions)]
    let tokens_info = tokens
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    info!(&debugger, "{tokens_info}");
    write_logs!(debugger);

    println!(
        "{GREEN}{BOLD}successfully compiled{WHITE} {}{RESET}",
        args.path.display()
    );

    process::exit(0);
}

fn parse_arguments() -> MeowindArguments {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        errors::throw(CommandLineError::new(
            CommandLineErrorKind::InvalidArguments,
            "path not provided",
        ));
    }

    if args.len() > 2 {
        errors::throw(CommandLineError::new(
            CommandLineErrorKind::InvalidArguments,
            format!("expected 1 argument, got {}", args.len() - 1),
        ));
    }

    MeowindArguments {
        path: PathBuf::from(&args[1]),
    }
}

fn read_source_contents(path: &PathBuf) -> String {
    let read_result = fs::read_to_string(path);

    let Ok(source_contents) = read_result else {
        let err = read_result.unwrap_err();
        if err.kind() != ErrorKind::NotFound {
            errors::throw(CommandLineError::new(
                CommandLineErrorKind::FailedToReadFile,
                format!("{}", err),
            ));
        }

        let rel_dir = path.parent().unwrap();
        if let Ok(abs_dir) = fs::canonicalize(rel_dir) {
            errors::throw(CommandLineError::new(
                CommandLineErrorKind::InvalidArguments,
                format!(
                    "file {:?} in directory \"{}\" does not exist",
                    path.file_name().unwrap(),
                    abs_dir.display()
                ),
            ));
        } else {
            errors::throw(CommandLineError::new(
                CommandLineErrorKind::InvalidArguments,
                "specified directory does not exist",
            ));
        }

        process::exit(1);
    };

    return source_contents;
}
