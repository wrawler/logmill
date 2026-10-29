use clap::Parser;

/// Custom parser to convert strings like "10M", "500K", or "1G" into raw bytes.
pub(crate) fn parse_size(s: &str) -> Result<usize, String> {
    let s = s.trim().to_uppercase();

    // Determine the multiplier based on the suffix
    let (num_str, multiplier) = if s.ends_with("MB") || s.ends_with('M') {
        (s.trim_end_matches("MB").trim_end_matches('M'), 1024 * 1024)
    } else if s.ends_with("KB") || s.ends_with('K') {
        (s.trim_end_matches("KB").trim_end_matches('K'), 1024)
    } else if s.ends_with("GB") || s.ends_with('G') {
        (
            s.trim_end_matches("GB").trim_end_matches('G'),
            1024 * 1024 * 1024,
        )
    } else if s.ends_with('B') {
        (s.trim_end_matches('B'), 1)
    } else {
        // If no letter is provided, assume raw bytes
        (s.as_str(), 1)
    };

    // Parse the remaining number and multiply it
    match num_str.trim().parse::<usize>() {
        Ok(0) => Err("size must be greater than zero".to_string()),
        Ok(num) => num
            .checked_mul(multiplier)
            .ok_or_else(|| format!("'{}' is too large for this platform", s)),
        Err(_) => Err(format!(
            "'{}' is not a valid number. Use formats like '10M', '500K', or '1048576'.",
            s
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_size;

    #[test]
    fn parses_units_and_whitespace() {
        assert_eq!(parse_size(" 2K ").unwrap(), 2 * 1024);
        assert_eq!(parse_size("3MB").unwrap(), 3 * 1024 * 1024);
        assert_eq!(parse_size("10").unwrap(), 10);
    }

    #[test]
    fn rejects_zero_and_overflow() {
        assert!(parse_size("0").is_err());
        assert!(parse_size("999999999999999999999G").is_err());
    }
}

#[derive(Parser, Debug)]
#[command(name = "Rust Log Rotator")]
#[command(about = "A zero-loss proxy log rotator for QA and dev environments.", long_about = None)]
pub struct Args {
    /// The path to the active log file
    #[arg(short, long, default_value = "app.log")]
    pub file: String,

    /// Maximum size before rotating (e.g., 10M, 500K, 2G, or raw bytes)
    #[arg(short, long, default_value = "1M", value_parser = parse_size)]
    pub max_size: usize,

    /// Directory to save the compressed .tar.gz archives
    #[arg(short, long, default_value = ".")]
    pub archive_dir: String,
}
