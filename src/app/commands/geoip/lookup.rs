use std::net::IpAddr;

use serde::Serialize;

use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::cli::GeoIpLookupArgs;

use super::backend::override_backend_config;

pub(crate) async fn run(context: &AppContext, args: &GeoIpLookupArgs) -> crate::app::Result<()> {
    let ip: IpAddr = args.ip.parse().map_err(|_| {
        crate::app::AppError::InvalidArgument(format!("invalid IP address: {}", args.ip))
    })?;
    let config =
        override_backend_config(&context.app_config, args.backend.as_deref(), args.no_cache)?;
    let lookup = crate::support::geoip::build_lookup_chain(&config, &context.runtime_paths)?;

    let result = LookupResult {
        backend: lookup.backend_name().to_string(),
        country: lookup.country(ip).await?,
        city: lookup.city(ip).await?,
        asn: lookup.asn(ip).await?,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!(
            "{}",
            output::format_kv(
                Some("GeoIP lookup"),
                &[
                    ("backend", result.backend.clone()),
                    ("country", output::dash(result.country.as_deref())),
                    ("city", output::dash(result.city.as_deref())),
                    ("asn", output::dash(result.asn.as_deref())),
                ],
                output::color_enabled(),
            )
        );
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct LookupResult {
    backend: String,
    country: Option<String>,
    city: Option<String>,
    asn: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_lookup_result() {
        let result = LookupResult {
            backend: "mmdb".to_string(),
            country: Some("NL".to_string()),
            city: Some("Amsterdam/NL".to_string()),
            asn: Some("AS1 Example".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"backend\":\"mmdb\""));
    }
}
