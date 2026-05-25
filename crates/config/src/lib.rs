#![warn(clippy::all)]
#![forbid(unsafe_code)]

use camino::Utf8PathBuf;
use miette::Diagnostic;
use serde::Deserialize;
use thiserror::Error;

const DEFAULT_CONFIG: &str = r#"[office]
# SSID of the network at your office.
# This is used to identify days worked from the office vs. from home.
# ssid = "<your work wifi network's SSID>"
"#;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug, Error, Diagnostic)]
pub enum ConfigError {
    #[error("No home directory found. Set $HOME or $XDG_CONFIG_HOME.")]
    #[diagnostic(code(config::no_home))]
    NoHome,

    #[error("Failed to create config directory `{path}`: {source}")]
    #[diagnostic(code(config::create_dir))]
    CreateDir {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write default config to `{path}`: {source}")]
    #[diagnostic(code(config::write_default))]
    WriteDefault {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Config file created at `{path}`. Review it and re-run.")]
    #[diagnostic(
        code(config::created),
        help("Open the file, verify the settings, then run the program again.")
    )]
    Created { path: Utf8PathBuf },

    #[error("Failed to read config file `{path}`: {source}")]
    #[diagnostic(code(config::read))]
    Read {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse config file `{path}`: {source}")]
    #[diagnostic(
        code(config::parse),
        help("Check the `[office]` section: it must contain `ssid = \"<your-ssid>\"`")
    )]
    Parse {
        path: Utf8PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Missing `[office]` section or `ssid` key in `{path}`.")]
    #[diagnostic(
        code(config::missing_ssid),
        help("Add `ssid = \"<your-ssid>\"` under the `[office]` section in the config file.")
    )]
    MissingSsid { path: Utf8PathBuf },
}

// ---------------------------------------------------------------------------
// Config struct
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawConfig {
    office: Option<OfficeSection>,
}

#[derive(Debug, Deserialize)]
struct OfficeSection {
    ssid: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    pub office_ssid: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

fn config_base_dir() -> Result<Utf8PathBuf, ConfigError> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME")
        && !xdg.is_empty()
    {
        return Ok(Utf8PathBuf::from(xdg));
    }
    let home = std::env::var("HOME").map_err(|_| ConfigError::NoHome)?;
    if home.is_empty() {
        return Err(ConfigError::NoHome);
    }
    Ok(Utf8PathBuf::from(home).join(".config"))
}

/// Parses a TOML string into a [`Config`].
///
/// Separated from file I/O so it can be tested without touching the filesystem.
pub(crate) fn parse_config(contents: &str, path: &Utf8PathBuf) -> Result<Config, ConfigError> {
    let raw: RawConfig = toml::from_str(contents).map_err(|e| ConfigError::Parse {
        path: path.clone(),
        source: e,
    })?;

    let office_ssid = raw
        .office
        .and_then(|o| o.ssid)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ConfigError::MissingSsid { path: path.clone() })?;

    Ok(Config { office_ssid })
}

/// Loads config from the platform config dir.
///
/// Creates a template file and returns an error if the file does not exist yet.
/// Returns an error if `[office] ssid` is not set.
pub fn load_config() -> miette::Result<Config> {
    let config_dir = config_base_dir()
        .map(|base| base.join("io.github.jcayzac.activity"))
        .map_err(miette::Report::from)?;

    let config_path = config_dir.join("config.toml");

    if !config_path.exists() {
        std::fs::create_dir_all(&config_dir).map_err(|e| {
            miette::Report::from(ConfigError::CreateDir {
                path: config_dir.clone(),
                source: e,
            })
        })?;

        std::fs::write(&config_path, DEFAULT_CONFIG).map_err(|e| {
            miette::Report::from(ConfigError::WriteDefault {
                path: config_path.clone(),
                source: e,
            })
        })?;

        return Err(miette::Report::from(ConfigError::Created {
            path: config_path,
        }));
    }

    let contents = std::fs::read_to_string(&config_path).map_err(|e| {
        miette::Report::from(ConfigError::Read {
            path: config_path.clone(),
            source: e,
        })
    })?;

    parse_config(&contents, &config_path).map_err(miette::Report::from)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use camino::Utf8PathBuf;

    use super::parse_config;

    fn dummy_path() -> Utf8PathBuf {
        Utf8PathBuf::from("/config.toml")
    }

    #[test]
    fn valid_config_parses() {
        let config =
            parse_config("[office]\nssid = \"my-office\"", &dummy_path()).expect("should parse");
        assert_eq!(config.office_ssid, "my-office");
    }

    #[test]
    fn missing_ssid_is_error() {
        assert!(parse_config("[office]\n", &dummy_path()).is_err());
    }

    #[test]
    fn missing_section_is_error() {
        assert!(parse_config("", &dummy_path()).is_err());
    }

    #[test]
    fn invalid_toml_is_error() {
        assert!(parse_config("[office\nssid = [not valid", &dummy_path()).is_err());
    }
}
