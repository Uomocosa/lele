use crate::{
    process::{ShellType, TerminalOutput, is_powershell_installed},
    thread::timeout_wrapper,
};
use anyhow::Result;
use regex::Regex;
use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Debug)]
pub struct Terminal {
    shell_type: ShellType,
    stdin: ChildStdin,
    stdout_reader: Arc<Mutex<BufReader<ChildStdout>>>,
    stop_function: fn(&[u8]) -> bool,
    fmt_function: fn(&[u8]) -> String,
    timeout_duration: Duration,
    n_command_to_read: u32,
    debug: bool,
}

impl Default for Terminal {
    fn default() -> Self {
        Terminal::new()
    }
}

impl Terminal {
    pub fn new() -> Self {
        if cfg!(target_os = "windows") {
            if is_powershell_installed() {
                Terminal::new_powershell_terminal()
            } else {
                Terminal::new_cmd_terminal()
            }
        } else {
            Terminal::new_sh_terminal()
        }
    }

    pub fn new_sh_terminal() -> Self {
        let shell_type = ShellType::Sh;
        create_new_terminal(shell_type)
    }

    pub fn new_cmd_terminal() -> Self {
        let shell_type = ShellType::Cmd;
        create_new_terminal(shell_type)
    }

    pub fn new_powershell_terminal() -> Self {
        let shell_type = ShellType::Powershell;
        let mut terminal = create_new_terminal(shell_type);
        // Hack: powershell always throws some boilerplate at the start.
        write(&mut terminal.stdin, "");
        read_line_per_line_with_timeout(
            terminal.stdout_reader.clone(),
            terminal.stop_function,
            terminal.fmt_function,
            Duration::from_millis(1000),
            false,
        )
        .expect("This should not fail");
        terminal
    }

    pub fn get_default_stop_function(shell_type: &ShellType) -> fn(&[u8]) -> bool {
        match shell_type {
            ShellType::Powershell => powershell_stop_function,
            ShellType::Cmd => cmd_stop_function,
            ShellType::Sh => todo!(),
        }
    }

    pub fn get_default_fmt_function(shell_type: &ShellType) -> fn(&[u8]) -> String {
        match shell_type {
            ShellType::Powershell => powershell_fmt_function,
            ShellType::Cmd => cmd_fmt_function,
            ShellType::Sh => todo!(),
        }
    }

    pub fn cmd(&mut self, command: &str) -> &mut Self {
        write(&mut self.stdin, command);
        self.n_command_to_read += 1;
        if command.is_empty() {
            return self;
        } else {
            write(&mut self.stdin, "");
        }
        self
    }

    pub fn read_all(&mut self) -> Result<TerminalOutput> {
        let mut output = TerminalOutput::new();
        if self.n_command_to_read == 0 {
            self.n_command_to_read = 1;
        };
        if self.debug {
            println!(">>> reading {} command/s", self.n_command_to_read)
        }
        for _ in 0..self.n_command_to_read {
            output.push_str(&read_line_per_line_with_timeout(
                self.stdout_reader.clone(),
                self.stop_function,
                self.fmt_function,
                self.timeout_duration,
                self.debug,
            )?);
        }
        self.n_command_to_read = 0;
        Ok(output)
    }

    pub fn read(&mut self) -> Result<TerminalOutput> {
        let mut output = TerminalOutput::new();
        output.push_str(&read_line_per_line_with_timeout(
            self.stdout_reader.clone(),
            self.stop_function,
            self.fmt_function,
            self.timeout_duration,
            self.debug,
        )?);
        if self.n_command_to_read > 0 {
            self.n_command_to_read -= 1;
        }
        Ok(output)
    }

    pub fn read_byte_per_byte(&mut self) -> Result<TerminalOutput> {
        let mut output = TerminalOutput::new();
        output.push_str(&read_byte_per_byte_with_timeout(
            self.stdout_reader.clone(),
            self.stop_function,
            self.fmt_function,
            self.timeout_duration,
            self.debug,
        )?);
        if self.n_command_to_read > 0 {
            self.n_command_to_read -= 1;
        }
        Ok(output)
    }

    pub fn get_shell_type(&self) -> &ShellType {
        &self.shell_type
    }

    pub fn set_timout_duration(&mut self, timeout_duration: Duration) -> &mut Self {
        self.timeout_duration = timeout_duration;
        self
    }

    pub fn set_stop_function(&mut self, stop_function: fn(&[u8]) -> bool) -> &mut Self {
        self.stop_function = stop_function;
        self
    }

    pub fn set_debug(&mut self, debug: bool) -> &mut Self {
        self.debug = debug;
        self
    }

    pub fn close(self) {
        drop(self)
    }
}

fn create_new_terminal(shell_type: ShellType) -> Terminal {
    let mut command = match shell_type {
        ShellType::Sh => Command::new("sh"),
        ShellType::Cmd => Command::new("cmd"),
        ShellType::Powershell => Command::new("powershell"),
    };
    #[allow(clippy::zombie_processes)]
    // Not sure if this is exactly correct, did not find any other way.
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect(">>> UNEXPECTED ERROR:\n\t>> Could not spawn() new Child for Terminal\n");

    let stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    // let mut stderr = child.stderr.take().expect("Failed to open stdout");
    let stdout_reader = BufReader::new(stdout);
    let stdout_reader = Arc::new(Mutex::new(stdout_reader));
    // let mut stderr_reader= BufReader::new(stderr);
    let stop_function = Terminal::get_default_stop_function(&shell_type);
    let fmt_function = Terminal::get_default_fmt_function(&shell_type);

    Terminal {
        shell_type,
        stdin,
        stdout_reader,
        stop_function,
        fmt_function,
        timeout_duration: Duration::from_millis(1500),
        n_command_to_read: 0,
        debug: false,
    }
}

fn cmd_stop_function(buf: &[u8]) -> bool {
    let pattern = r".*:\\.*>\n";
    // println!("pattern: {:?}", pattern);
    let text_buffer = String::from_utf8_lossy(buf);
    let re = Regex::new(pattern).expect("pattern is correct");
    re.is_match(&text_buffer)
}

fn powershell_stop_function(buf: &[u8]) -> bool {
    let pattern = r"PS.*> \n";
    // println!("pattern: {:?}", pattern);
    let text_buffer = String::from_utf8_lossy(buf);
    let re = Regex::new(pattern).expect("pattern is correct");
    re.is_match(&text_buffer)
}

fn cmd_fmt_function(buf: &[u8]) -> String {
    // The buffer taken from the spawned child, specifically I refer to: stdout(Stdio::piped()).
    // It has an encoding that I cannot figure out, for reference the 'é' char is the 0x8A byte.
    let output = String::from_utf8_lossy(buf).to_string();
    let start_pattern = r".*:\\.*>.*\n";
    let end_pattern = r".*:\\.*>\n";
    let start_re = Regex::new(start_pattern).expect("pattern is correct");
    let end_re = Regex::new(end_pattern).expect("pattern is correct");
    let start_match = start_re.find(&output);
    let end_match = end_re.find(&output);

    match (start_match, end_match) {
        (Some(start), Some(end)) => {
            if end.start() > start.end() {
                let start_index = start.end();
                let end_index = end.start();
                output[start_index..end_index].trim().to_string()
            } else {
                "".to_string()
            }
        }
        (Some(start), None) => {
            let start_index = start.end();
            output[start_index..].trim().to_string()
        }
        (None, Some(end)) => {
            let end_index = end.start();
            output[..end_index].trim().to_string()
        }
        (None, None) => output,
    }
}

fn powershell_fmt_function(buf: &[u8]) -> String {
    // The buffer taken from the spawned child, specifically I refer to: stdout(Stdio::piped()).
    // It has an encoding that I cannot figure out, for reference the 'é' char is the 0x8A byte.
    let output = String::from_utf8_lossy(buf).to_string();
    let start_pattern = r"PS.*> .*\n";
    let end_pattern = r"PS.*> \n";
    let start_re = Regex::new(start_pattern).expect("pattern is correct");
    let end_re = Regex::new(end_pattern).expect("pattern is correct");
    let start_match = start_re.find(&output);
    let end_match = end_re.find(&output);

    match (start_match, end_match) {
        (Some(start), Some(end)) => {
            if end.start() > start.end() {
                let start_index = start.end();
                let end_index = end.start();
                output[start_index..end_index].trim().to_string()
            } else {
                "".to_string()
            }
        }
        (Some(start), None) => {
            let start_index = start.end();
            output[start_index..].trim().to_string()
        }
        (None, Some(end)) => {
            let end_index = end.start();
            output[..end_index].trim().to_string()
        }
        (None, None) => output,
    }
}

fn write(stdin: &mut ChildStdin, command: &str) {
    stdin
        .write_all(format!("{command}\n").as_bytes())
        .expect(">>> ERROR:\n\t>> Could not write on stdin");
    stdin
        .flush()
        .expect(">>> ERROR:\n\t>> Could not flush() stdin\n");
}

fn read_line_per_line_with_timeout(
    stdout_reader: Arc<Mutex<BufReader<ChildStdout>>>, // Take Arc<Mutex>
    stop_function: fn(&[u8]) -> bool,
    fmt_function: fn(&[u8]) -> String,
    timeout_duration: Duration,
    debug: bool,
) -> Result<String> {
    timeout_wrapper(
        move || {
            // Lock the Mutex to get mutable access to BufReader within the thread
            let mut stdout_reader_guard = stdout_reader.lock().unwrap();
            read_line_per_line(&mut stdout_reader_guard, stop_function, fmt_function, debug) // Dereference MutexGuard for &mut BufReader
        },
        timeout_duration,
    )
}

fn read_byte_per_byte_with_timeout(
    stdout_reader: Arc<Mutex<BufReader<ChildStdout>>>, // Take Arc<Mutex>
    stop_function: fn(&[u8]) -> bool,
    fmt_function: fn(&[u8]) -> String,
    timeout_duration: Duration,
    debug: bool,
) -> Result<String> {
    timeout_wrapper(
        move || {
            // Lock the Mutex to get mutable access to BufReader within the thread
            let mut stdout_reader_guard = stdout_reader.lock().unwrap();
            read_byte_per_byte(&mut stdout_reader_guard, stop_function, fmt_function, debug) // Dereference MutexGuard for &mut BufReader
        },
        timeout_duration,
    )
}

fn read_line_per_line(
    stdout_reader_buf: &mut BufReader<ChildStdout>,
    stop_function: fn(&[u8]) -> bool,
    fmt_function: fn(&[u8]) -> String,
    debug: bool,
) -> String {
    let mut buffer: Vec<u8> = Vec::new();
    loop {
        stdout_reader_buf
            .read_until(b'\n', &mut buffer)
            .expect(">>> ERROR:\n\t>> Could not read from stdout_reader\n");
        let output = String::from_utf8_lossy(&buffer).to_string(); // to remove
        if debug {
            println!("output:\n{:?}", output); // to remove
        }
        if stop_function(&buffer) {
            break;
        }
    }
    fmt_function(&buffer)
}

fn read_byte_per_byte(
    stdout_reader_buf: &mut BufReader<ChildStdout>,
    stop_function: fn(&[u8]) -> bool,
    fmt_function: fn(&[u8]) -> String,
    debug: bool,
) -> String {
    let mut buffer: Vec<u8> = Vec::new();
    let mut single_byte_buf: [u8; 1] = [0_u8];
    loop {
        stdout_reader_buf
            .read_exact(&mut single_byte_buf)
            .expect(">>> ERROR:\n\t>> Could not read from stdout_reader\n");
        if debug {
            print!("{}", String::from_utf8_lossy(&single_byte_buf));
        }
        buffer.push(single_byte_buf[0]);
        if stop_function(&buffer) {
            break;
        }
    }
    fmt_function(&buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_terminal() -> Result<()> {
        let terminal = Terminal::new();
        terminal.close();
        Ok(())
    }

    #[test]
    fn dir_command() -> Result<()> {
        let mut terminal = Terminal::new();
        let dir = terminal
            .cmd("dir")
            .set_timout_duration(Duration::from_millis(5000))
            .read()?; // This works
        // println!("Output:\n\t{}", dir);
        assert!(!dir.to_string().is_empty());
        terminal.close();
        Ok(())
    }

    #[test]
    fn mantain_terminal_state() -> Result<()> {
        let mut terminal = Terminal::new();
        let dir_1 = terminal.cmd("cd ~").cmd("dir").read_all()?; // This works
        let dir_2 = terminal.cmd("cd ..").cmd("dir").read_all()?;
        let dir_1 = dir_1.last().expect("dir_1 should return an output");
        let dir_2 = dir_2.last().expect("dir_2 should return an output");
        assert_ne!(dir_1, dir_2);
        // println!("dir_1: \n{}", dir_1);
        // println!("dir_2: \n{}", dir_2);
        terminal.close();
        Ok(())
    }

    #[test]
    fn can_different_terminal_run_simultanely() -> Result<()> {
        let mut terminal_1 = Terminal::new();
        let mut terminal_2 = Terminal::new();
        terminal_1.set_timout_duration(Duration::from_secs(3));
        terminal_2.set_timout_duration(Duration::from_secs(3));
        let dir_11 = terminal_1.cmd("cd ~").cmd("dir").read_all()?; // This works
        let dir_12 = terminal_2.cmd("cd ~").cmd("dir").read_all()?; // This works
        let dir_21 = terminal_1.cmd("cd ..").cmd("dir").read_all()?;
        let dir_11 = dir_11.last().expect("dir_1 should return an output");
        let dir_21 = dir_21.last().expect("dir_2 should return an output");
        let dir_12 = dir_12.last().expect("dir_2 should return an output");
        assert_eq!(dir_11, dir_12);
        assert_ne!(dir_11, dir_21);
        // println!("dir_11: \n{}", dir_11);
        // println!("dir_12: \n{}", dir_12);
        // println!("dir_21: \n{}", dir_21);
        terminal_1.close();
        terminal_2.close();
        Ok(())
    }

    #[test]
    fn cmd_terminals() -> Result<()> {
        let mut terminal_1 = Terminal::new_cmd_terminal();
        let mut terminal_2 = Terminal::new_cmd_terminal();
        let dir_11 = terminal_1.cmd("cd %HOMEPATH%").cmd("dir").read_all()?; // This works
        let dir_12 = terminal_2.cmd("cd %HOMEPATH%").cmd("dir").read_all()?; // This works
        let dir_21 = terminal_1.cmd("cd ..").cmd("dir").read_all()?;
        let dir_11 = dir_11.last().expect("dir_1 should return an output");
        let dir_21 = dir_21.last().expect("dir_2 should return an output");
        let dir_12 = dir_12.last().expect("dir_2 should return an output");
        assert_eq!(dir_11, dir_12);
        assert_ne!(dir_11, dir_21);
        // println!("dir_11: \n{}", dir_11);
        // println!("dir_12: \n{}", dir_12);
        // println!("dir_21: \n{}", dir_21);
        terminal_1.close();
        terminal_2.close();
        Ok(())
    }

    #[test]
    #[ignore = "it takes a LOT of time ~15s"]
    fn dir_command_byte_by_byte() -> Result<()> {
        let mut terminal = Terminal::new();
        let dir = terminal
            .cmd("dir")
            .set_timout_duration(Duration::from_secs(30))
            .read_byte_per_byte()?; // This works
        // println!("Output:\n\t{}", dir);
        assert!(!dir.to_string().is_empty());
        terminal.close();
        Ok(())
    }

    // #[test]
    // #[ignore = "Use this test to find the correct powershell encoding (not yet found)"]
    // fn test_powershell_encoding() {
    //     use encoding_rs::{Encoding, UTF_8}; // use 'cargo add encoding_rs'
    //     // THIS DOES NOT WORK:
    //     fn latin1_to_string(s: &[u8]) -> String {
    //         s.iter().map(|&c| c as char).collect()
    //     }

    //     // I tested the following encododings, none of them worked for the 'é' char.
    //     // UTF_8, UTF_16LE, WINDOWS_1250, WINDOWS_1251,
    //     // ISO_8859_10, ISO_8859_13, ISO_8859_14,

    //     let buf = &[138_u8]; // This byte should be equal to é
    //     println!("latin1_to_string: {:?}", latin1_to_string(buf));
    //     let encoding = encoding_rs::ISO_8859_13;

    //     let (decoded, _, had_errors) = encoding.decode(buf);
    //     println!("encoding.decode:\n{}\n", decoded);
    //     let (decoded, had_errors) = encoding.decode_with_bom_removal(buf);
    //     println!("encoding.decode_with_bom_removal:\n{}\n", decoded);
    //     let (decoded, had_errors) = encoding.decode_without_bom_handling(buf);
    //     println!("encoding.decode_without_bom_handling:\n{}\n", decoded);
    //     let decoded = encoding.decode_without_bom_handling_and_without_replacement(buf);
    //     if let Some(str) = decoded {
    //         println!("encoding.decode_without_bom_handling_and_without_replacement:\n{}\n", str);
    //     }
    // }
}
