#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    pub id: String,
    pub start: usize,
    pub end: usize,
}

pub fn attribute(text: &str) -> Vec<Marker> {
    let bytes = text.as_bytes();
    let mut out: Vec<Marker> = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'[' {
            i += 1;
            continue;
        }
        // Scan inner until ']' or end. Accept digit groups separated by commas + spaces.
        let start = i;
        let mut j = i + 1;
        let mut ids: Vec<String> = Vec::new();
        let mut cur: String = String::new();
        let mut saw_digit_in_part = false;
        let mut valid = true;
        let mut closed = false;
        while j < bytes.len() {
            let b = bytes[j];
            if b == b']' {
                closed = true;
                break;
            }
            if b.is_ascii_digit() {
                cur.push(b as char);
                saw_digit_in_part = true;
                j += 1;
                continue;
            }
            if b == b',' {
                if !saw_digit_in_part {
                    valid = false;
                    break;
                }
                ids.push(std::mem::take(&mut cur));
                saw_digit_in_part = false;
                j += 1;
                while j < bytes.len() && bytes[j] == b' ' {
                    j += 1;
                }
                continue;
            }
            // Anything else inside is invalid for a citation marker.
            valid = false;
            break;
        }
        if !closed || !valid || !saw_digit_in_part {
            i += 1;
            continue;
        }
        ids.push(cur);
        let end = j + 1;
        for id in ids {
            if !id.is_empty() {
                out.push(Marker { id, start, end });
            }
        }
        i = end;
    }
    out
}

pub fn unique_marker_ids(markers: &[Marker]) -> Vec<String> {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out: Vec<String> = Vec::new();
    for m in markers {
        if seen.insert(m.id.clone()) {
            out.push(m.id.clone());
        }
    }
    out
}
