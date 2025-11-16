use std::{fmt::Display, fs, path::Path};

use chrono::{NaiveDateTime, format::StrftimeItems};
use serde::{Deserialize, Serialize};

use crate::{ZkError, ZkResult};

const DATE_FMT_STR: &str = "%Y-%m-%d %I:%M:%S %p";

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct FrontMatter {
    pub title: String,
    pub created_at: NaiveDateTime,
    pub tag_strings: Vec<String>,
}

impl FrontMatter {
    pub fn new(
        title: impl Into<String>,
        created_at: NaiveDateTime,
        tag_strings: Vec<impl Into<String>>,
    ) -> Self {
        let tag_strings = tag_strings.into_iter().map(|e| e.into()).collect();

        FrontMatter {
            title: title.into(),
            created_at,
            tag_strings,
        }
    }

    /// Reads in file and returns the front matter as well as the content after it.
    /// expected format for front matter as follows
    ///```md
    /// ---
    /// Title: LOL
    /// Date: 2025-01-01 12:50:19 AM
    /// Tags: #Daily{#ffffff} #barber{#000000}
    /// ---
    /// ```
    pub fn extract_from_file(path: impl AsRef<Path>) -> ZkResult<(Self, String)> {
        let string = fs::read_to_string(&path)?;
        Self::extract_from_str(&string).map_err(|e| {
            ZkError::ParseError(format!(
                "Unable to parse frontmatter from file {:#?}, reason: {e}",
                path.as_ref()
            ))
        })
    }

    /// Returns the front matter as well as the content after it.
    /// expected format for front matter as follows
    ///```md
    /// ---
    /// Title: LOL
    /// Date: 2025-01-01 12:50:19 AM
    /// Tags: #Daily{#ffffff} #barber{#000000}
    /// ---
    /// ```
    pub fn extract_from_str(string: impl Into<String>) -> ZkResult<(Self, String)> {
        let string: String = string.into();
        // we just want to strictly match this, else we error

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
        let title = lines
            .get(1)
            .ok_or(ZkError::ParseError("Title line doesn't exist!".to_owned()))?
            .strip_prefix("Title: ")
            .ok_or(ZkError::ParseError(
                "Title line doesn't start with \"Title: \" ".to_owned(),
            ))?;

        let created_at = lines
            .get(2)
            .ok_or(ZkError::ParseError("Date line doesn't exist!".to_owned()))?
            .strip_prefix("Date: ")
            .ok_or(ZkError::ParseError(
                "Date line doesn't start with \"Date: \" ".to_owned(),
            ))
            .map(|date_str| NaiveDateTime::parse_from_str(date_str, DATE_FMT_STR))?
            .map_err(|err| ZkError::ParseError(err.to_string()))?;

        let tag_strings: Vec<String> = lines
            .get(3)
            .ok_or_else(|| ZkError::ParseError("Tag line doesn't exist!".to_owned()))?
            .strip_prefix("Tags: ")
            .ok_or(ZkError::ParseError(
                "Tag line doesn't start with \"Tags: \" ".to_owned(),
            ))?
            .split_whitespace()
            .map(|e| e.to_owned())
            .collect::<Vec<_>>();

        delim_check(4)?;

        let remaining = lines[5..].join("\n");

        Ok((FrontMatter::new(title, created_at, tag_strings), remaining))
    }
}

impl Display for FrontMatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let date_fmt_items = StrftimeItems::new(DATE_FMT_STR);
        writeln!(f, "---")?;
        writeln!(f, "Title: {}", self.title)?;
        writeln!(
            f,
            "Date: {}",
            self.created_at.format_with_items(date_fmt_items)
        )?;
        write!(f, "Tags: ")?;

        for tag in &self.tag_strings {
            write!(f, "{} ", tag)?;
        }

        writeln!(f, "\n---")
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    use chrono::NaiveDateTime;

    use crate::{FrontMatter, zettel::frontmatter::DATE_FMT_STR};

    lazy_static! {
        static ref test_suite: [(&'static str, (FrontMatter, &'static str)); 1] = [(
            r#"---            
Title: LOL
Date: 2025-01-01 12:50:19 AM
Tags: whoa barber
---
"#,
            (
                FrontMatter::new(
                    "LOL",
                    NaiveDateTime::parse_from_str("2025-01-01 12:50:19 AM", DATE_FMT_STR).unwrap(),
                    vec!["whoa", "barber",],
                ),
                "",
            ),
        )];
    }

    #[test]
    fn test_extract_from_string() {
        for (raw_text, (front_matter, remaining)) in test_suite.iter() {
            let (extracted_front_matter, extracted_remaining) =
                FrontMatter::extract_from_str(*raw_text).unwrap();

            assert_eq!(extracted_front_matter, *front_matter);
            assert_eq!(extracted_remaining, *remaining);
        }
    }
}
