#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionLevel {
    HighQuality,
    Balanced,
    Strong,
}

#[derive(Debug)]
pub enum CompressionStatus {
    Idle,
    Running,
    Success(String),
    Error(String),
}
