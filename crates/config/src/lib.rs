#![warn(clippy::all)]
#![cfg_attr(not(test), forbid(unsafe_code))]

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

/// Loads config from the platform config dir. Creates the template file and
/// returns an error if it doesn't exist yet or if `office.ssid` is not set.
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

    let raw: RawConfig = toml::from_str(&contents).map_err(|e| {
        miette::Report::from(ConfigError::Parse {
            path: config_path.clone(),
            source: e,
        })
    })?;

    let office_ssid = raw
        .office
        .and_then(|o| o.ssid)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| miette::Report::from(ConfigError::MissingSsid { path: config_path }))?;

    Ok(Config { office_ssid })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{DEFAULT_CONFIG, RawConfig, load_config};

    #[test]
    fn parse_valid_toml() {
        let toml = "[office]\nssid = \"my-office\"";
        let raw: RawConfig = toml::from_str(toml).expect("should parse");
        assert_eq!(raw.office.unwrap().ssid.unwrap(), "my-office");
    }

    #[test]
    fn missing_ssid_is_none() {
        let toml = "[office]\n";
        let raw: RawConfig = toml::from_str(toml).expect("should parse");
        assert!(raw.office.unwrap().ssid.is_none());
    }

    #[test]
    fn missing_section_is_none() {
        let toml = "";
        let raw: RawConfig = toml::from_str(toml).expect("should parse");
        assert!(raw.office.is_none());
    }

    #[test]
    fn invalid_toml_returns_error() {
        let toml = "[office\nssid = [not valid";
        let result: Result<RawConfig, _> = toml::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn load_config_creates_template_and_errors() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let tmp_path = tmp.path().to_str().expect("utf8 path");

        let _guard = EnvGuard::set("HOME", tmp_path);
        let _guard2 = EnvGuard::unset("XDG_CONFIG_HOME");

        let result = load_config();
        assert!(
            result.is_err(),
            "should error when config file is newly created"
        );

        let config_path = format!(
            "{}/.config/io.github.jcayzac.activity/config.toml",
            tmp_path
        );
        let written = std::fs::read_to_string(&config_path).expect("file should exist");
        assert!(written.contains("[office]"));
        assert!(
            !written.contains("r-intra"),
            "template must not hardcode r-intra"
        );
        assert!(
            written.contains("# ssid ="),
            "ssid must be commented out in template"
        );
    }

    #[test]
    fn load_config_errors_when_ssid_commented_out() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let tmp_path = tmp.path().to_str().expect("utf8 path");
        let config_dir = format!("{}/.config/io.github.jcayzac.activity", tmp_path);
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(format!("{}/config.toml", config_dir), DEFAULT_CONFIG).unwrap();

        let _guard = EnvGuard::set("HOME", tmp_path);
        let _guard2 = EnvGuard::unset("XDG_CONFIG_HOME");

        let result = load_config();
        assert!(result.is_err(), "should error when ssid is not set");
    }

    #[test]
    fn load_config_succeeds_with_ssid_set() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let tmp_path = tmp.path().to_str().expect("utf8 path");
        let config_dir = format!("{}/.config/io.github.jcayzac.activity", tmp_path);
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            format!("{}/config.toml", config_dir),
            "[office]\nssid = \"corp-wifi\"\n",
        )
        .unwrap();

        let _guard = EnvGuard::set("HOME", tmp_path);
        let _guard2 = EnvGuard::unset("XDG_CONFIG_HOME");

        let config = load_config().expect("should succeed");
        assert_eq!(config.office_ssid, "corp-wifi");
    }

    struct EnvGuard {
        key: &'static str,
        original: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let original = std::env::var(key).ok();
            unsafe { std::env::set_var(key, value) };
            Self { key, original }
        }

        fn unset(key: &'static str) -> Self {
            let original = std::env::var(key).ok();
            unsafe { std::env::remove_var(key) };
            Self { key, original }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(v) => unsafe { std::env::set_var(self.key, v) },
                None => unsafe { std::env::remove_var(self.key) },
            }
        }
    }
}
