use crate::myers::Edit;
use crate::patch::Hunk;

pub trait ToPatch: Sized {
    fn to_patch(&self, old_name: Option<&str>, new_name: Option<&str>) -> String;
}

pub trait FromPatch: Sized {
    fn from_patch(s: &str) -> Result<Self, PatchError>;
}

#[derive(Debug)]
pub enum PatchError {
    InvalidFormat(String),
    UnexpectedToken(String),
}

impl<T: ToString> ToPatch for Edit<T> {
    fn to_patch(&self, _: Option<&str>, _: Option<&str>) -> String {
        match self {
            Edit::Equal(el) => format!(" {}", el.to_string()),
            Edit::Insert(el) => format!("+{}", el.to_string()),
            Edit::Delete(el) => format!("-{}", el.to_string()),
        }
    }
}

impl FromPatch for Edit<String> {
    fn from_patch(s: &str) -> Result<Self, PatchError> {
        match s.chars().next() {
            Some(' ') => Ok(Edit::Equal(s[1..].to_string())),
            Some('+') => Ok(Edit::Insert(s[1..].to_string())),
            Some('-') => Ok(Edit::Delete(s[1..].to_string())),
            _ => Err(PatchError::UnexpectedToken(s.to_string())),
        }
    }
}

impl<T: ToString> ToPatch for Hunk<T> {
    fn to_patch(&self, _old_name: Option<&str>, _new_name: Option<&str>) -> String {
        let old_edits = self
            .changes
            .iter()
            .filter(|e| !matches!(e, Edit::Insert(_)))
            .count();
        let new_edits = self
            .changes
            .iter()
            .filter(|e| !matches!(e, Edit::Delete(_)))
            .count();
        let header = format!(
            "@@ -{},{} +{},{} @@",
            self.old_start, old_edits, self.new_start, new_edits
        );
        let body = self
            .changes
            .iter()
            .map(|e| e.to_patch(None, None))
            .collect::<Vec<String>>();

        format!("{}\n{}", header, body.join("\n"))
    }
}

impl<T: ToString> ToPatch for Vec<Hunk<T>> {
    fn to_patch(&self, old_name: Option<&str>, new_name: Option<&str>) -> String {
        if self.is_empty() {
            return String::new();
        }

        let header = format!(
            "--- {}\n+++ {}\n",
            old_name.unwrap_or("old"),
            new_name.unwrap_or("new")
        );
        let hunks = self
            .iter()
            .map(|h| h.to_patch(None, None))
            .collect::<String>();
        format!("{}{}", header, hunks)
    }
}

impl FromPatch for Vec<Hunk<String>> {
    fn from_patch(s: &str) -> Result<Self, PatchError> {
        if s.is_empty() {
            return Ok(vec![]);
        }

        // can't use `.lines()` because of Windows \r
        // would break the roundtrip property
        let mut lines = s.split('\n');
        let first_line = lines.next().unwrap_or("");
        let second_line = lines.next().unwrap_or("");
        if !first_line.starts_with("---") || !second_line.starts_with("+++") {
            return Err(PatchError::InvalidFormat(format!(
                "{}\n{}",
                first_line, second_line
            )));
        }

        let mut current = None;
        let mut hunks = vec![];

        for e in lines {
            if e.starts_with("@@") {
                if let Some(c) = current {
                    hunks.push(c);
                }

                match parse_hunk_header(e) {
                    Ok((old_start, new_start)) => {
                        current = Some(Hunk {
                            old_start,
                            new_start,
                            changes: vec![],
                        });
                    }
                    Err(PatchError::InvalidFormat(err)) => {
                        return Err(PatchError::InvalidFormat(err))
                    }
                    _ => unreachable!(),
                }
            } else {
                if let Some(ref mut c) = current {
                    c.changes.push(Edit::from_patch(e)?);
                } else {
                    return Err(PatchError::InvalidFormat(e.to_string()));
                }
            }
        }

        if let Some(c) = current {
            hunks.push(c);
        }

        Ok(hunks)
    }
}

fn parse_hunk_header(s: &str) -> Result<(usize, usize), PatchError> {
    // s = "@@ -1,4 +1,4 @@"
    let s = s.trim_start_matches("@@ ").trim_end_matches(" @@");
    let parts: Vec<&str> = s.split(' ').collect();
    // parts = ["-1,4", "+1,4"]
    let old_start = parts[0]
        .trim_start_matches('-')
        .split(',')
        .next()
        .ok_or(PatchError::InvalidFormat(s.to_string()))?
        .parse::<usize>()
        .map_err(|_| PatchError::InvalidFormat(s.to_string()))?;
    let new_start = parts[1]
        .trim_start_matches('+')
        .split(',')
        .next()
        .ok_or(PatchError::InvalidFormat(s.to_string()))?
        .parse::<usize>()
        .map_err(|_| PatchError::InvalidFormat(s.to_string()))?;
    Ok((old_start, new_start))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::myers::diff;
    use crate::patch::{hunks, Hunk};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_serialization_roundtrip(
                    old in prop::collection::vec(".*", 0..20usize),
        new in prop::collection::vec(".*", 0..20usize),
        ) {
            let edits = diff(&old, &new);
            let hunks = hunks(edits.clone());
            let patch = hunks.to_patch(None, None);

            prop_assert_eq!(Vec::<Hunk<String>>::from_patch(&patch).unwrap(), hunks);
        }
    }
}
