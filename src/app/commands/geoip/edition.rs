use std::str::FromStr;

use crate::app::AppError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MmdbEdition {
    Country,
    City,
    Asn,
}

impl MmdbEdition {
    pub(crate) fn file_name(self) -> &'static str {
        match self {
            Self::Country => "GeoLite2-Country.mmdb",
            Self::City => "GeoLite2-City.mmdb",
            Self::Asn => "GeoLite2-ASN.mmdb",
        }
    }

    pub(crate) fn canonical_name(self) -> &'static str {
        self.file_name().trim_end_matches(".mmdb")
    }
}

impl std::fmt::Display for MmdbEdition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.canonical_name())
    }
}

impl FromStr for MmdbEdition {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "geolite2-country" | "country" => Ok(Self::Country),
            "geolite2-city" | "city" => Ok(Self::City),
            "geolite2-asn" | "asn" => Ok(Self::Asn),
            _ => Err(AppError::InvalidArgument(format!(
                "unsupported geoip edition '{value}'; valid values: GeoLite2-Country, GeoLite2-City, GeoLite2-ASN, country, city, asn"
            ))),
        }
    }
}

pub(crate) const SUPPORTED_EDITIONS: [MmdbEdition; 3] =
    [MmdbEdition::Country, MmdbEdition::City, MmdbEdition::Asn];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_and_short_edition_names() {
        assert_eq!(
            MmdbEdition::from_str("GeoLite2-Country").unwrap(),
            MmdbEdition::Country
        );
        assert_eq!(MmdbEdition::from_str("city").unwrap(), MmdbEdition::City);
        assert_eq!(MmdbEdition::from_str("ASN").unwrap(), MmdbEdition::Asn);
    }

    #[test]
    fn rejects_invalid_edition_name() {
        let error = MmdbEdition::from_str("geo").expect_err("edition should fail");
        assert!(error.to_string().contains("unsupported geoip edition"));
    }
}
