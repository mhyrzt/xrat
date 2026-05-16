mod check;
mod errors;

pub use check::{DownloadResult, download_speed_check};

#[cfg(test)]
mod tests;
