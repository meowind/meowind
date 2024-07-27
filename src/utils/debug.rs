use std::path::PathBuf;

#[macro_export]
macro_rules! debug {
    (&$d:expr, $($arg:tt)*) => {{
        #[allow(path_statements)]
        {
            $d;
        }

        #[cfg(debug_assertions)]
        {
            let mut debugger = $d.borrow_mut();
            let l = format!($($arg)*);

            debugger.logs.push(l.to_string());
            if debugger.console_debug {
                println!("{}", l.to_string());
            }
        }
    }};
}

#[macro_export]
macro_rules! info {
    (&$d:expr, $($arg:tt)*) => {{
        #[allow(path_statements)]
        {
            $d;
        }

        #[cfg(debug_assertions)]
        {
            let mut debugger = $d.borrow_mut();
            let l = format!($($arg)*);

            debugger.logs.push(l.to_string());
            if debugger.console_info {
                println!("{}", l.to_string());
            }
        }
    }};
}

#[macro_export]
macro_rules! write_logs {
    ($d:expr) => {{
        #[allow(path_statements)]
        {
            $d;
        }

        #[cfg(debug_assertions)]
        {
            let debugger = $d.borrow_mut();

            let contents = debugger.logs.join("\n");
            let folder = debugger.path.parent().unwrap();

            let path_without_ext = debugger.path.with_extension("");
            let source_filename = path_without_ext.file_name().unwrap();
            let filename = format!("{}_debug.txt", source_filename.to_str().unwrap());
            let full_path = folder.join(filename);

            fs::write(full_path, contents).expect("failed to write log");
        }
    }};
}

pub struct Debugger {
    pub path: PathBuf,
    pub logs: Vec<String>,
    pub console_info: bool,
    pub console_debug: bool,
}

impl Debugger {
    pub fn new(path: PathBuf, console_info: bool, console_debug: bool) -> Debugger {
        Debugger {
            path,
            logs: Vec::new(),
            console_info,
            console_debug,
        }
    }
}
