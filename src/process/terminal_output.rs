use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct TerminalOutput {
    output: Vec<String>,
}

impl Default for TerminalOutput {
    fn default() -> Self {
        TerminalOutput::new()
    }
}

impl Display for TerminalOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.output {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl TerminalOutput {
    pub fn new() -> Self {
        TerminalOutput {
            output: Vec::<String>::new(),
        }
    }

    pub fn push_str(&mut self, string: &str) -> &mut Self {
        self.output.push(string.to_string());
        self
    }

    pub fn last(&self) -> Option<&String> {
        self.output.last()
    }
}
