use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WildcardRange {
    pub start: i64,
    pub end: i64,
    pub step: u64,
    pub padding: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPreviewRequest {
    pub pattern: String,
    pub ranges: Vec<WildcardRange>,
    pub maximum: usize,
}

pub fn expand_wildcards(request: &BatchPreviewRequest) -> Result<Vec<String>> {
    let count = request.pattern.matches('*').count();
    if count == 0 || count > 2 || count != request.ranges.len() {
        return Err(Error::Task(
            "pattern must contain one or two wildcards with matching ranges".into(),
        ));
    }
    let mut values = Vec::with_capacity(count);
    for range in &request.ranges {
        if range.step == 0 || range.start > range.end {
            return Err(Error::Task("invalid wildcard range".into()));
        }
        let mut group = Vec::new();
        let mut n = range.start;
        while n <= range.end {
            group.push(if range.padding == 0 {
                n.to_string()
            } else {
                format!("{:0width$}", n, width = range.padding as usize)
            });
            n = n
                .checked_add(range.step as i64)
                .ok_or_else(|| Error::Task("wildcard range overflow".into()))?;
        }
        values.push(group);
    }
    let total = values
        .iter()
        .try_fold(1usize, |a, v| a.checked_mul(v.len()))
        .ok_or_else(|| Error::Task("batch is too large".into()))?;
    let maximum = request.maximum.clamp(1, 10_000);
    if total > maximum {
        return Err(Error::Task(format!(
            "batch contains {total} URLs; maximum is {maximum}"
        )));
    }
    let parts = request.pattern.split('*').collect::<Vec<_>>();
    let mut out = Vec::with_capacity(total);
    for first in &values[0] {
        if count == 1 {
            out.push(format!("{}{}{}", parts[0], first, parts[1]));
        } else {
            for second in &values[1] {
                out.push(format!("{}{}{}{}{}", parts[0], first, parts[1], second, parts[2]));
            }
        }
    }
    out.dedup();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_wildcard_padding_step() {
        let x = expand_wildcards(&BatchPreviewRequest {
            pattern: "https://x/p*.zip".into(),
            ranges: vec![WildcardRange {
                start: 1,
                end: 5,
                step: 2,
                padding: 3,
            }],
            maximum: 100,
        })
        .unwrap();
        assert_eq!(
            x,
            vec!["https://x/p001.zip", "https://x/p003.zip", "https://x/p005.zip"]
        )
    }
    #[test]
    fn two_wildcards_preserve_order() {
        let x = expand_wildcards(&BatchPreviewRequest {
            pattern: "https://x/d*/v*.mp4".into(),
            ranges: vec![
                WildcardRange {
                    start: 1,
                    end: 2,
                    step: 1,
                    padding: 0,
                },
                WildcardRange {
                    start: 1,
                    end: 2,
                    step: 1,
                    padding: 2,
                },
            ],
            maximum: 10,
        })
        .unwrap();
        assert_eq!(x[0], "https://x/d1/v01.mp4");
        assert_eq!(x[3], "https://x/d2/v02.mp4")
    }
    #[test]
    fn safety_limit() {
        assert!(
            expand_wildcards(&BatchPreviewRequest {
                pattern: "x*".into(),
                ranges: vec![WildcardRange {
                    start: 1,
                    end: 101,
                    step: 1,
                    padding: 0
                }],
                maximum: 100
            })
            .is_err()
        )
    }
}
