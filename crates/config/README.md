TOML configuration loader for the `activity` tool.

The single public function `load_config() -> miette::Result<Config>` resolves the config file path from `$XDG_CONFIG_HOME/io.github.jcayzac.activity/config.toml` (or `~/.config/io.github.jcayzac.activity/config.toml`), reads and parses it, and returns a `Config` struct containing the `office_ssid` field. If the file does not exist, a commented-out template is written and an error is returned asking the user to review it before re-running. An error is also returned if `office.ssid` is absent or empty.
