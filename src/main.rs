#![feature(let_chains)]
#[allow(unused)]
pub mod errors;
pub mod frontend;
pub mod middlend;
pub mod structs;
pub mod utils;

use std::time::Instant;

use errors::ErrorList;
use frontend::{
    lexing::Token,
    parsing::{ast::project::ProjectNode, Parser},
};

use crate::{
    errors::command_line::{CommandLineError, CommandLineErrorKind},
    frontend::lexing::Lexer,
    structs::{MeowindArguments, ScriptSource},
    utils::colors::*,
};
use std::{env, fs, io::ErrorKind, path::PathBuf, process};

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    #[cfg(windows)]
    init_windows_colors();

    let args = parse_arguments();

    let source_contents = read_source_contents(&args.path);
    let source = ScriptSource::new(args.path.clone(), &source_contents);

    println!(
        "{GREEN}{BOLD}compiling{WHITE} {}{RESET}",
        args.path.display()
    );

    let comp_start = Instant::now();

    let tokens = run_lexer(source.clone());

    #[allow(unused)]
    let ast = run_parser(&tokens, source.clone());

    let comp_micros = comp_start.elapsed().as_micros();
    let comp_millis = comp_start.elapsed().as_millis();

    println!(
        "{GREEN}{BOLD}successfully compiled{WHITE} {} {GREEN}in{WHITE} {}us {GREEN}or{WHITE} {}ms{RESET}",
        args.path.display(),
        comp_micros,
        comp_millis
    );

    process::exit(0);
}

fn run_lexer(source: ScriptSource) -> Vec<Token> {
    #[cfg(debug_assertions)]
    let lexer_start = Instant::now();
    let lexer = Lexer::tokenize(source);

    lexer.errors.throw_if_there();

    #[cfg(debug_assertions)]
    let lexer_micros = lexer_start.elapsed().as_micros();
    #[cfg(debug_assertions)]
    let lexer_millis = lexer_start.elapsed().as_millis();

    #[cfg(debug_assertions)]
    let tokens_info = lexer
        .tokens
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    log!(
        "lexer output:\n{}\nlexer finished in: {}us or {}ms",
        tokens_info,
        lexer_micros,
        lexer_millis
    );

    return lexer.tokens;
}

fn run_parser(tokens: &Vec<Token>, source: ScriptSource) -> ProjectNode {
    #[cfg(debug_assertions)]
    let parser_start = Instant::now();
    let parser = Parser::parse(tokens, source);

    parser.errors.throw_if_there();

    #[cfg(debug_assertions)]
    let parser_micros = parser_start.elapsed().as_micros();
    #[cfg(debug_assertions)]
    let parser_millis = parser_start.elapsed().as_millis();

    log!(
        "parser output:\n{:#?}\nparser finished in: {}us or {}ms",
        parser.project,
        parser_micros,
        parser_millis
    );

    return parser.project;
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
