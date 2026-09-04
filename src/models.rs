use std::path::PathBuf;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionLevel {
    HighQuality,
    Balanced,
    Strong,
}

#[derive(Debug)]
pub struct CompressionResult {
    pub input: PathBuf,
    pub output: PathBuf,
    pub input_size: u64,
    pub output_size: u64,
}

#[derive(Debug)]
pub enum CompressionStatus {
    Idle,
    Running,
    Success(Vec<CompressionResult>),
    Error(String),
}

impl CompressionResult {
    pub fn saved_bytes(&self) -> u64 {
        self.input_size.saturating_sub(self.output_size)
    }

    pub fn reduction_percent(&self) -> f64 {
        if self.input_size == 0 {
            return 0.0;
        }

        100.0 * (1.0 - self.output_size as f64 / self.input_size as f64)
    }
}
