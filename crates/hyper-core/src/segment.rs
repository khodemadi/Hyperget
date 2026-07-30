use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    pub index: u32,
    pub start: u64,
    pub end: u64,
}
pub fn split_segments(total: u64, count: u8) -> Vec<Segment> {
    if total == 0 || count == 0 {
        return vec![];
    };
    let n = u64::from(count).min(total);
    let base = total / n;
    let extra = total % n;
    let mut start = 0;
    (0..n)
        .map(|i| {
            let len = base + u64::from(i < extra);
            let s = Segment {
                index: i as u32,
                start,
                end: start + len - 1,
            };
            start = s.end + 1;
            s
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_gaps_or_overlap() {
        for total in 1..500 {
            for count in 1..16 {
                let s = split_segments(total, count);
                assert_eq!(s[0].start, 0);
                assert_eq!(s.last().unwrap().end, total - 1);
                for w in s.windows(2) {
                    assert_eq!(w[0].end + 1, w[1].start)
                }
            }
        }
    }
}
