//! Utilities for predicting memory usage and reporting potential problems
//! early.

/// The suggested memory limit, 1 GiB.
pub const MEMORY_LIMIT: usize = 1 << 30;

/// Rounds a number down to a given number of decimals.
fn floor_to(n: f64, decimals: u8) -> f64 {
    let decimal_mult = f64::from(10u32.pow(u32::from(decimals)));
    (n * decimal_mult).floor() / decimal_mult
}

/// Stringifies a number representing a number of bytes in human-readable
/// form.
#[allow(clippy::must_use_candidate, clippy::cast_precision_loss)]
pub fn format_bytes(size: usize) -> String {
    if size == 1 {
        "1 byte".to_owned()
    } else if size < (1 << 10) {
        format!("{size} bytes")
    } else if size < (1 << 20) {
        format!("{:.2} KiB", floor_to((size as f64) / f64::from(1 << 10), 2))
    } else if size < (1 << 30) {
        format!("{:.2} MiB", floor_to((size as f64) / f64::from(1 << 20), 2))
    } else {
        format!("{:.2} GiB", floor_to((size as f64) / f64::from(1 << 30), 2))
    }
}

/// Estimates the memory usage in bytes based on the configured chunk size and
/// pool size.
#[allow(clippy::must_use_candidate)]
pub fn estimated_memory_usage(chunk_size: usize, pool_size: u8) -> usize {
    // `total_pool_size` is a necessary transformation of `pool_size` since
    // the internals of the task pool can cause up to `2n+3` chunks to be in
    // memory at any given time, where `n` is the pool size. In this case, we
    // are using `2n+5` since there will be one additional memory chunk at
    // either end, one for the next request and one for the most recent
    // response.
    let total_pool_size = usize::from(pool_size) * 2 + 5;
    chunk_size * total_pool_size
}

/// Checks roughly how much memory will be allocated during the backup or
/// extraction. This will prompt for confirmation if the threshold is exceeded
/// and confirmation is not overridden.
///
/// # Errors
///
/// This will return an error if the suggested memory limit is exceeded and is
/// not overridden.
pub fn check_memory(chunk_size: usize, pool_size: u8, override_limit: bool) -> Result<(), String> {
    let required_bytes = estimated_memory_usage(chunk_size, pool_size);

    if required_bytes > MEMORY_LIMIT {
        if !override_limit {
            Err(format!("The suggested memory limit of 1 GiB has been exceeded.\nThe expected memory usage with the current configuration is {}.\nChange the chunk size magnitude or pool size to lower the expected memory usage, or override the memory limit to proceed with the existing configuration.", format_bytes(required_bytes)))
        } else {
            println!("The suggested memory limit of 1 GiB has been exceeded and the expected memory usage will be {}, but the limit has been overridden", format_bytes(required_bytes));
            Ok(())
        }
    } else {
        Ok(())
    }
}

/// Memory tests.
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_floats_eq {
        ( $left:expr, $right:expr ) => {
            assert!(($left - $right).abs() < f64::EPSILON)
        };
    }

    #[test]
    fn test_floor_to() {
        assert_floats_eq!(floor_to(1.234, 0), 1.);
        assert_floats_eq!(floor_to(1.234, 1), 1.2);
        assert_floats_eq!(floor_to(1.234, 2), 1.23);
        assert_floats_eq!(floor_to(1.234, 3), 1.234);
        assert_floats_eq!(floor_to(1.234, 4), 1.234);
        assert_floats_eq!(floor_to(6.789, 0), 6.);
        assert_floats_eq!(floor_to(6.789, 1), 6.7);
        assert_floats_eq!(floor_to(6.789, 2), 6.78);
        assert_floats_eq!(floor_to(6.789, 3), 6.789);
        assert_floats_eq!(floor_to(6.789, 4), 6.789);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 bytes");
        assert_eq!(format_bytes(1), "1 byte");
        assert_eq!(format_bytes(2), "2 bytes");
        assert_eq!(format_bytes(1_023), "1023 bytes");
        assert_eq!(format_bytes(1_024), "1.00 KiB");
        assert_eq!(format_bytes(1_260), "1.23 KiB");
        assert_eq!(format_bytes(24_013), "23.45 KiB");
        assert_eq!(format_bytes(353_967), "345.67 KiB");
        assert_eq!(format_bytes(1_048_575), "1023.99 KiB");
        assert_eq!(format_bytes(1_048_576), "1.00 MiB");
        assert_eq!(format_bytes(1_289_749), "1.23 MiB");
        assert_eq!(format_bytes(24_589_108), "23.45 MiB");
        assert_eq!(format_bytes(362_461_266), "345.67 MiB");
        assert_eq!(format_bytes(1_073_741_823), "1023.99 MiB");
        assert_eq!(format_bytes(1_073_741_824), "1.00 GiB");
        assert_eq!(format_bytes(1_320_702_444), "1.23 GiB");
        assert_eq!(format_bytes(25_179_245_773), "23.45 GiB");
        assert_eq!(format_bytes(371_160_336_303), "345.67 GiB");
    }
}
