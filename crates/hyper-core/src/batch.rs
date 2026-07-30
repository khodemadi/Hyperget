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
    if count != 1 || request.ranges.len() != 1 {
        return Err(Error::Task(
            if count > 1 {
                "multiple wildcards are not supported in this version"
            } else {
                "pattern must contain exactly one wildcard"
            }
            .into(),
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
    fn multiple_wildcards_are_rejected() {
        let result = expand_wildcards(&BatchPreviewRequest {
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
        });
        assert!(result.is_err())
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
    #[test]
    fn missing_wildcard_and_zero_step_are_rejected() {
        assert!(
            expand_wildcards(&BatchPreviewRequest {
                pattern: "https://x/file.zip".into(),
                ranges: vec![],
                maximum: 10
            })
            .is_err()
        );
        assert!(
            expand_wildcards(&BatchPreviewRequest {
                pattern: "https://x/file*.zip".into(),
                ranges: vec![WildcardRange {
                    start: 1,
                    end: 2,
                    step: 0,
                    padding: 0
                }],
                maximum: 10
            })
            .is_err()
        )
    }
}
