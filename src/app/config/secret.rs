use serde::Deserialize;
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum SecretString {
    Literal(String),
    Env { env: String },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SecretError {
    #[error("missing environment variable {0}")]
    MissingEnv(String),
}

impl SecretString {
    pub fn resolve(&self) -> Result<String, SecretError> {
        self.resolve_with(|name| std::env::var(name))
    }

    fn resolve_with<F>(&self, get_env: F) -> Result<String, SecretError>
    where
        F: FnOnce(&str) -> Result<String, std::env::VarError>,
    {
        match self {
            Self::Literal(value) => Ok(value.clone()),
            Self::Env { env } => get_env(env).map_err(|_| SecretError::MissingEnv(env.clone())),
        }
    }
}

impl Default for SecretString {
    fn default() -> Self {
        Self::Literal(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::{SecretError, SecretString};

    #[test]
    fn parses_literal_and_env_secret_strings() {
        #[derive(serde::Deserialize)]
        struct Secrets {
            literal: SecretString,
            env_secret: SecretString,
        }

        let secrets: Secrets =
            toml::from_str("literal = \"plain\"\nenv_secret = { env = \"XRAT_SECRET\" }\n")
                .expect("secrets should parse");

        assert_eq!(secrets.literal, SecretString::Literal("plain".to_string()));
        assert_eq!(
            secrets.env_secret,
            SecretString::Env {
                env: "XRAT_SECRET".to_string()
            }
        );
    }

    #[test]
    fn resolves_literal_secret_string() {
        let secret = SecretString::Literal("plain".to_string());

        let resolved = secret
            .resolve_with(|_| panic!("literal should not read env"))
            .expect("literal should resolve");

        assert_eq!(resolved, "plain");
    }

    #[test]
    fn resolves_env_secret_string() {
        let secret = SecretString::Env {
            env: "XRAT_SECRET".to_string(),
        };

        let resolved = secret
            .resolve_with(|name| {
                assert_eq!(name, "XRAT_SECRET");
                Ok("from-env".to_string())
            })
            .expect("env secret should resolve");

        assert_eq!(resolved, "from-env");
    }

    #[test]
    fn reports_missing_env_secret_string() {
        let secret = SecretString::Env {
            env: "XRAT_SECRET".to_string(),
        };

        let error = secret
            .resolve_with(|_| Err(std::env::VarError::NotPresent))
            .expect_err("missing env should fail");

        assert_eq!(error, SecretError::MissingEnv("XRAT_SECRET".to_string()));
    }
}
