#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedHttpStatuses {
    codes: Vec<u16>,
    ranges: Vec<(u16, u16)>,
}

impl AcceptedHttpStatuses {
    pub fn new(codes: Vec<u16>, ranges: Vec<(u16, u16)>) -> Result<Self, String> {
        if codes.is_empty() && ranges.is_empty() {
            return Err("at least one accepted HTTP status code or range is required".to_string());
        }
        if let Some(code) = codes.iter().find(|&&code| !(100..=599).contains(&code)) {
            return Err(format!("HTTP status code {code} must be within 100-599"));
        }
        if let Some((start, end)) = ranges
            .iter()
            .find(|(start, end)| !(100..=599).contains(start) || !(100..=599).contains(end))
        {
            return Err(format!(
                "HTTP status range {start}-{end} must stay within 100-599"
            ));
        }
        if let Some((start, end)) = ranges.iter().find(|(start, end)| start > end) {
            return Err(format!(
                "HTTP status range {start}-{end} starts after it ends"
            ));
        }

        Ok(Self { codes, ranges })
    }

    pub fn matches(&self, status: u16) -> bool {
        self.codes.contains(&status)
            || self
                .ranges
                .iter()
                .any(|(start, end)| (*start..=*end).contains(&status))
    }

    pub(crate) fn description(&self) -> String {
        self.codes
            .iter()
            .map(u16::to_string)
            .chain(
                self.ranges
                    .iter()
                    .map(|(start, end)| format!("{start}-{end}")),
            )
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Default for AcceptedHttpStatuses {
    fn default() -> Self {
        Self {
            codes: Vec::new(),
            ranges: vec![(200, 299)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_accepts_only_success_statuses() {
        let statuses = AcceptedHttpStatuses::default();

        assert!(statuses.matches(200));
        assert!(statuses.matches(299));
        assert!(!statuses.matches(199));
        assert!(!statuses.matches(300));
    }

    #[test]
    fn exact_codes_and_ranges_use_or_semantics() {
        let statuses =
            AcceptedHttpStatuses::new(vec![204, 403], vec![(300, 399)]).expect("valid matcher");

        assert!(statuses.matches(204));
        assert!(statuses.matches(403));
        assert!(statuses.matches(300));
        assert!(statuses.matches(399));
        assert!(!statuses.matches(200));
        assert!(!statuses.matches(400));
    }

    #[test]
    fn rejects_empty_or_invalid_matchers() {
        assert!(AcceptedHttpStatuses::new(Vec::new(), Vec::new()).is_err());
        assert!(AcceptedHttpStatuses::new(vec![600], Vec::new()).is_err());
        assert!(AcceptedHttpStatuses::new(Vec::new(), vec![(399, 300)]).is_err());
    }
}
