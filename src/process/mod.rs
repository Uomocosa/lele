// Automatic imports (from build.rs)
mod is_powershell_installed;
pub use is_powershell_installed::is_powershell_installed;
mod shell_type;
pub use shell_type::ShellType;
mod terminal;
pub use terminal::Terminal;
mod terminal_output;
pub use terminal_output::TerminalOutput;
