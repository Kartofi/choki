pub fn count_char_occurrences(s: &str, target: char) -> usize {
    s.chars()
        .filter(|&c| c == target)
        .count()
}
pub fn map_compression_level(compression_float: f32) -> u32 {
    if compression_float <= 0.0 {
        0
    } else if compression_float >= 1.0 {
        10
    } else {
        (compression_float * 10.0).round() as u32
    }
}
