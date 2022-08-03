use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct Range {
    pub c1: String,
    pub r1: usize,
    pub c2: String,
    pub r2: usize,
}

impl From<&str> for Range {
    fn from(s: &str) -> Self {
        if let Ok(re) = Regex::new(r#"(?i)^([a-z]+)(\d+):([a-z]+)(\d+)$"#) {
            if let Some(cap) = re.captures(s) {
                return Range {
                    c1: cap[1].to_string().to_uppercase(),
                    r1: cap[2].parse::<usize>().unwrap_or_default(), // default should be invalid as a range
                    c2: cap[3].to_string().to_uppercase(),
                    r2: cap[4].parse::<usize>().unwrap_or_default(), // default should be invalid as a range
                };
            }
        }

        // invalid range
        Range {
            c1: "A".to_string(),
            r1: 0,
            c2: "A".to_string(),
            r2: 0,
        }
    }
}

impl ToString for Range {
    fn to_string(&self) -> String {
        format!("{}{}:{}{}", self.c1, self.r1, self.c2, self.r2)
    }
}

impl From<&Range> for String {
    fn from(r: &Range) -> Self {
        r.to_string()
    }
}
