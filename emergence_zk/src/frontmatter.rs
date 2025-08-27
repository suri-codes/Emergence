use std::{fmt::Display, path::PathBuf};

use chrono::NaiveDateTime;

use crate::{Tag, ZkResult};

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
    pub fn extract_from_file(path: &PathBuf) -> ZkResult<(Self, String)> {
        todo!()
    }

    pub fn extract_from_str(string: &str) -> ZkResult<(Self, String)> {
        todo!()
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
    use std::str::FromStr;

    use chrono::NaiveDateTime;

    use crate::{FrontMatter, Tag};

    #[test]
    fn test_extract_from_string() {
        let frontmatter_string = r#"
---            
Name: LOL
Date: 2025-01-01 12:50:19 AM

#penis{#ffffff} #barber{#000000}
---
"#;

        let (frontmatter, _) = FrontMatter::extract_from_str(frontmatter_string).unwrap();
        assert_eq!(
            frontmatter,
            FrontMatter::new(
                "LOL",
                NaiveDateTime::from_str("2025-01-01 12:50:19 AM").unwrap(),
                vec![
                    Tag::new("penis", "#ffffff").unwrap(),
                    Tag::new("barber", "#000000").unwrap()
                ],
            )
        );
    }
}
