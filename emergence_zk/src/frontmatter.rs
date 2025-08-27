use std::{fmt::Display, fs, path::PathBuf};

use chrono::NaiveDateTime;

use crate::{Tag, ZkError, ZkResult};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrontMatter {
    pub name: String,
    pub created_at: NaiveDateTime,
    pub tags: Vec<Tag>,
}

impl FrontMatter {
    pub fn new(name: &str, created_at: NaiveDateTime, tags: Vec<Tag>) -> Self {
        FrontMatter {
            name: name.to_owned(),
            created_at,
            tags,
        }
    }

    /// Reads in file and returns the front matter as well as the content after it.
    /// expected format for front matter as follows
    ///```md
    /// ---
    /// Name: LOL
    /// Date: 2025-01-01 12:50:19 AM
    /// #penis{#ffffff} #barber{#000000}
    /// ---
    /// ```

    pub fn extract_from_file(path: &PathBuf) -> ZkResult<(Self, String)> {
        let string = fs::read_to_string(path)?;
        Self::extract_from_str(&string)
    }

    /// Returns the front matter as well as the content after it.
    /// expected format for front matter as follows
    ///```md
    /// ---
    /// Name: LOL
    /// Date: 2025-01-01 12:50:19 AM
    /// #penis{#ffffff} #barber{#000000}
    /// ---
    /// ```
    pub fn extract_from_str(string: &str) -> ZkResult<(Self, String)> {
        // we just want to strictly match this, else we error
        //
        let lines: Vec<_> = string.lines().collect();

        let delim_check = |line_number: usize| -> ZkResult<()> {
            let delim = lines
                .get(line_number)
                .ok_or(ZkError::ParseError(format!(
                    "Line Number {line_number} doesnt exist!"
                )))?
                .trim();
            if delim != "---" {
                return Err(ZkError::ParseError(
                    "FrontMatter Deliminator Corrupted!".to_owned(),
                ));
            }
            Ok(())
        };

        // check first line
        delim_check(0)?;

        //extract name
        let name = lines
            .get(1)
            .ok_or(ZkError::ParseError("Name line doesn't exist!".to_owned()))?
            .strip_prefix("Name: ")
            .ok_or(ZkError::ParseError(
                "Name line doesn't start with \"Name: \" ".to_owned(),
            ))?;

        let created_at = lines
            .get(2)
            .ok_or(ZkError::ParseError("Date line doesn't exist!".to_owned()))?
            .strip_prefix("Date: ")
            .ok_or(ZkError::ParseError(
                "Date line doesn't start with \"Date: \" ".to_owned(),
            ))
            .map(|date_str| NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %I:%M:%S %p"))?
            .map_err(|err| ZkError::ParseError(err.to_string()))?;

        let tags: Vec<Tag> = lines
            .get(3)
            .ok_or_else(|| ZkError::ParseError("Tag line doesn't exist!".to_owned()))?
            .split_whitespace()
            .map(|tag_str| {
                let par_idx = tag_str.find('{').ok_or_else(|| {
                    ZkError::ParseError("Unable to find color for Tag!".to_owned())
                })?;
                let tag_name = &tag_str[1..par_idx];
                let tag_color = &tag_str[par_idx + 1..par_idx + 8];
                Tag::new(tag_name, tag_color)
            })
            .collect::<Result<Vec<_>, _>>()?;

        delim_check(4)?;

        let remaining = lines[5..].join("\n");

        Ok((FrontMatter::new(name, created_at, tags), remaining))
    }
}

impl Display for FrontMatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "---")?;
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Date: {}", self.created_at)?;

        for tag in &self.tags {
            write!(f, "#{} ", tag)?;
        }

        writeln!(f, "---")
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    use chrono::NaiveDateTime;

    use crate::{FrontMatter, Tag};

    lazy_static! {
        static ref test_suite: [(&'static str, (FrontMatter, &'static str)); 1] = [(
            r#"---            
Name: LOL
Date: 2025-01-01 12:50:19 AM
#penis{#ffffff} #barber{#000000}
---
"#,
            (
                FrontMatter::new(
                    "LOL",
                    NaiveDateTime::parse_from_str("2025-01-01 12:50:19 AM", "%Y-%m-%d %I:%M:%S %p")
                        .unwrap(),
                    vec![
                        Tag::new("penis", "#ffffff").unwrap(),
                        Tag::new("barber", "#000000").unwrap(),
                    ],
                ),
                "",
            ),
        )];
    }

    #[test]
    fn test_extract_from_string() {
        for (raw_text, (front_matter, remaining)) in test_suite.iter() {
            let (extracted_front_matter, extracted_remaining) =
                FrontMatter::extract_from_str(raw_text).unwrap();

            assert_eq!(extracted_front_matter, *front_matter);
            assert_eq!(extracted_remaining, *remaining);
        }
    }
}
