use crate::db::ConfigWithLatestTest;
use crate::support::refs::short_ref;

#[derive(Debug, Clone, PartialEq)]
pub struct TuiConfigRow {
    pub id: i64,
    pub r#ref: String,
    pub name: String,
    pub protocol: String,
    pub address: String,
    pub port: i64,
    pub network: String,
    pub tls: Option<String>,
    pub icmp_ms: Option<i64>,
    pub real_delay_ms: Option<i64>,
    pub tcp_ms: Option<i64>,
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub dial_endpoint_country: Option<String>,
    pub dial_endpoint_location: Option<String>,
    pub dial_endpoint_asn: Option<String>,
    pub dial_endpoint_fronting: Option<String>,
    pub failure_reason: Option<String>,
    pub source_id: Option<i64>,
    pub tested_at: Option<String>,
    pub imported_at: String,
    pub is_active: bool,
    pub is_enabled: bool,
    pub is_deleted: bool,
}

impl TuiConfigRow {
    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            "-"
        } else {
            &self.name
        }
    }

    pub fn display_ref(&self) -> &str {
        short_ref(&self.r#ref)
    }

    pub fn endpoint(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    pub fn address_label(&self) -> &str {
        if self.address.is_empty() {
            "-"
        } else {
            &self.address
        }
    }

    pub fn port_label(&self) -> String {
        self.port.to_string()
    }

    pub fn network_label(&self) -> String {
        match &self.tls {
            Some(tls) if !tls.is_empty() => format!("{}+{}", self.network, tls),
            _ => self.network.clone(),
        }
    }

    pub fn delay_label(&self) -> String {
        if self.failure_reason.is_some() {
            "fail".to_string()
        } else {
            Self::ms_label(self.real_delay_ms)
        }
    }

    pub fn icmp_label(&self) -> String {
        Self::ms_label(self.icmp_ms)
    }

    pub fn tcp_label(&self) -> String {
        Self::ms_label(self.tcp_ms)
    }

    pub fn download_label(&self) -> String {
        Self::mbps_label(self.download_mbps, 1)
    }

    pub fn upload_label(&self) -> String {
        Self::mbps_label(self.upload_mbps, 1)
    }

    pub fn download_detail_label(&self) -> String {
        Self::mbps_label(self.download_mbps, 2)
    }

    pub fn upload_detail_label(&self) -> String {
        Self::mbps_label(self.upload_mbps, 2)
    }

    pub fn country_label(&self) -> &str {
        self.dial_endpoint_country.as_deref().unwrap_or("-")
    }

    pub fn location_label(&self) -> &str {
        self.dial_endpoint_location.as_deref().unwrap_or("-")
    }

    pub fn asn_label(&self) -> &str {
        self.dial_endpoint_asn.as_deref().unwrap_or("-")
    }

    pub fn fronting_label(&self) -> Option<&str> {
        self.dial_endpoint_fronting.as_deref()
    }

    pub fn needs_location_enrichment(&self) -> bool {
        if self.address.trim().is_empty() {
            return false;
        }
        let has_real_geo = self.dial_endpoint_country.is_some()
            || self.dial_endpoint_asn.is_some()
            || self
                .dial_endpoint_location
                .as_deref()
                .is_some_and(|label| !crate::support::geoip::is_classified_placeholder(label));
        !has_real_geo
    }

    pub fn apply_location_meta(&mut self, meta: crate::support::geoip::EndpointGeoMeta) {
        if !meta.has_lookup_metadata() {
            return;
        }

        let crate::support::geoip::EndpointGeoMeta {
            location,
            country,
            asn,
            source: _,
            fronting,
        } = meta;
        if let Some(location) = location {
            self.dial_endpoint_location = Some(location);
        }
        if let Some(country) = country {
            self.dial_endpoint_country = Some(country);
        }
        if let Some(asn) = asn {
            self.dial_endpoint_asn = Some(asn);
        }
        self.dial_endpoint_fronting = fronting;
    }

    pub fn clear_test_fields(&mut self) {
        self.icmp_ms = None;
        self.real_delay_ms = None;
        self.tcp_ms = None;
        self.download_mbps = None;
        self.upload_mbps = None;
        self.dial_endpoint_country = None;
        self.dial_endpoint_location = None;
        self.dial_endpoint_asn = None;
        self.dial_endpoint_fronting = None;
        self.failure_reason = None;
        self.tested_at = None;
    }

    fn ms_label(value: Option<i64>) -> String {
        value
            .map(|value| format!("{value}ms"))
            .unwrap_or_else(|| "-".to_string())
    }

    fn mbps_label(value: Option<f64>, decimals: usize) -> String {
        value
            .map(|value| format!("{value:.decimals$} Mbps"))
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn matches_search(&self, query: &str) -> bool {
        self.display_name().to_lowercase().contains(query)
            || self.r#ref.to_lowercase().contains(query)
            || self.protocol.to_lowercase().contains(query)
            || self.address.to_lowercase().contains(query)
            || self.network.to_lowercase().contains(query)
            || self
                .dial_endpoint_country
                .as_deref()
                .is_some_and(|value| value.to_lowercase().contains(query))
            || self
                .dial_endpoint_location
                .as_deref()
                .is_some_and(|value| value.to_lowercase().contains(query))
            || self
                .dial_endpoint_asn
                .as_deref()
                .is_some_and(|value| value.to_lowercase().contains(query))
            || self
                .source_id
                .map(|source_id| source_id.to_string().contains(query))
                .unwrap_or(false)
    }
}

impl From<ConfigWithLatestTest> for TuiConfigRow {
    fn from(value: ConfigWithLatestTest) -> Self {
        let config = value.config;
        Self {
            id: config.id,
            r#ref: config.r#ref,
            name: config.name.unwrap_or_default(),
            protocol: config.protocol,
            address: config.address,
            port: config.port,
            network: config.network,
            tls: config.tls,
            icmp_ms: value.icmp_ms,
            real_delay_ms: value.real_delay_ms,
            tcp_ms: value.tcp_ms,
            download_mbps: value.download_mbps,
            upload_mbps: value.upload_mbps,
            dial_endpoint_country: value.dial_endpoint_country,
            dial_endpoint_location: value.dial_endpoint_location,
            dial_endpoint_asn: value.dial_endpoint_asn,
            dial_endpoint_fronting: value.dial_endpoint_fronting,
            failure_reason: value.failure_reason,
            source_id: config.subscription_id,
            tested_at: value.tested_at,
            imported_at: config.imported_at,
            is_active: config.is_active,
            is_enabled: config.is_enabled,
            is_deleted: config.is_deleted,
        }
    }
}
