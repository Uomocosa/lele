use std::process::{Command, Stdio};

pub fn is_powershell_installed() -> bool {
    // Hack: run a simple "-help" command in powershell, to see if it works
    let mut command = Command::new("powershell");
    command
        .arg("-help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_powershell_installed() {
        assert!(is_powershell_installed());
    }
}
