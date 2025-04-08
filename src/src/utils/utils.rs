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

// Buffer stuff
pub fn replace_bytes(buffer: &mut Vec<u8>, target: &[u8], replacement: &[u8]) {
    let mut i = 0;
    while i <= buffer.len() - target.len() {
        if &buffer[i..i + target.len()] == target {
            buffer.splice(i..i + target.len(), replacement.iter().cloned());
            i += replacement.len();
        } else {
            i += 1;
        }
    }
}

pub fn split_buffer_inxeses(buffer: &[u8], delimiter: &[u8]) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let mut start = 0;

    let mut i = 0;
    while i <= buffer.len() - delimiter.len() {
        if &buffer[i..i + delimiter.len()] == delimiter {
            segments.push((start, i));
            start = i + delimiter.len();
            i += delimiter.len();
        } else {
            i += 1;
        }
    }
    // Push the last segment
    segments.push((start, buffer.len()));

    segments
}
