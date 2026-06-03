pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = 1_024 * KB;
    const GB: u64 = 1_024 * MB;
    const TB: u64 = 1_024 * GB;
    const PB: u64 = 1_024 * TB;

    match bytes {
        0..KB  => format!("{} B",    bytes),
        KB..MB => format!("{:.1} KB", bytes as f64 / KB as f64),
        MB..GB => format!("{:.1} MB", bytes as f64 / MB as f64),
        GB..TB => format!("{:.1} GB", bytes as f64 / GB as f64),
        TB..PB => format!("{:.1} TB", bytes as f64 / TB as f64),
        _      => format!("{:.1} PB", bytes as f64 / PB as f64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes() {
        assert_eq!(format_size(0),    "0 B");
        assert_eq!(format_size(512),  "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn kilobytes() {
        assert_eq!(format_size(1_024),     "1.0 KB");
        assert_eq!(format_size(1_536),     "1.5 KB");
        assert_eq!(format_size(1_048_575), "1024.0 KB");
    }

    #[test]
    fn megabytes() {
        assert_eq!(format_size(1_048_576), "1.0 MB");
        assert_eq!(format_size(3_251_634), "3.1 MB");
    }

    #[test]
    fn gigabytes() {
        assert_eq!(format_size(1_073_741_824), "1.0 GB");
        assert_eq!(format_size(5_368_709_120), "5.0 GB");
    }

    #[test]
    fn terabytes() {
        assert_eq!(format_size(1_099_511_627_776), "1.0 TB");
        assert_eq!(format_size(1_200_273_234_234), "1.1 TB");
    }

    #[test]
    fn petabytes() {
        assert_eq!(format_size(1_125_899_906_842_624), "1.0 PB");
        assert_eq!(format_size(2_251_799_813_685_248), "2.0 PB");
    }
}
