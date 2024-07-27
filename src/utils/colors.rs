use winapi_util::console::Console;

pub const RED: &str = "\x1b[91m";
pub const GREEN: &str = "\x1b[92m";
pub const BLUE: &str = "\x1b[94m";

pub const CYAN: &str = "\x1b[96m";
pub const MAGENTA: &str = "\x1b[95m";
pub const YELLOW: &str = "\x1b[93m";

pub const WHITE: &str = "\x1b[97m";
pub const GRAY: &str = "\x1b[90m";

pub const BOLD: &str = "\x1b[1m";
pub const ITALIC: &str = "\x1b[3m";
pub const UNDERLINE: &str = "\x1b[4m";

pub const RESET: &str = "\x1b[0m";

#[cfg(windows)]
pub fn init_windows_colors() {
    let mut con = Console::stdout();
    let _ = con
        .as_mut()
        .map(|con| con.set_virtual_terminal_processing(true));
}
