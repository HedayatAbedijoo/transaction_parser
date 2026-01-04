#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("missing input csv path. usage: cargo run -- <transactions.csv>")]
    MissingArg,
    #[error("failed to open input file: {0}")]
    OpenInput(#[from] std::io::Error),
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("process error: {0}")]
    Process(String),
}
