#![feature(let_chains)]

pub mod errors;
pub mod frontend;
pub mod structs;
pub mod utils;

#[cfg(debug_assertions)]
use std::time::Instant;

use crate::{
    errors::command_line::{CommandLineError, CommandLineErrorKind},
    frontend::lexing::Lexer,
    structs::{MeowindArguments, MeowindScriptSource},
    utils::colors::*,
};
use std::{env, fs, io::ErrorKind, path::PathBuf, process};

fn main() {
    #[cfg(windows)]
    init_windows_colors();

    let args = parse_arguments();

    let source_contents = read_source_contents(&args.path);
    let source = MeowindScriptSource::new(args.path.clone(), &source_contents);

    println!(
        "{GREEN}{BOLD}compiling{WHITE} {}{RESET}",
        args.path.display()
    );

    #[cfg(debug_assertions)]
    let lexer_start = Instant::now();
    let mut lexer = Lexer::new(source);
    let (tokens, errors) = lexer.tokenize();

    #[cfg(debug_assertions)]
    let micros = lexer_start.elapsed().as_micros();
    #[cfg(debug_assertions)]
    let millis = lexer_start.elapsed().as_millis();

    errors.throw_if_there();

    // TODO: токены в будущем нада но щяс затычка чтоб варна не было
    #[cfg(not(debug_assertions))]
    let _ = tokens;

    #[cfg(debug_assertions)]
    let tokens_info = tokens
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    debug!(
        "lexer output:\n{}\nlexer finished in: {}us or {}ms",
        tokens_info, micros, millis
    );

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
