use std::fs::read_to_string;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn load(filename: &str) -> Result<Self, std::io::Error> {
        Ok(Self {
            lines: read_to_string(filename)?
                .lines()
                .map(|line| String::from(line))
                .collect(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
