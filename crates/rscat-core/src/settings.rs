use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::{self, Display},
    fs,
    path::PathBuf,
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Runner {
    Cat,
    Parrot,
    Horse,
}

impl Runner {
    pub const ALL: [Runner; 3] = [Runner::Cat, Runner::Parrot, Runner::Horse];

    pub fn as_id(self) -> &'static str {
        match self {
            Runner::Cat => "cat",
            Runner::Parrot => "parrot",
            Runner::Horse => "horse",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Runner::Cat => "Cat",
            Runner::Parrot => "Parrot",
            Runner::Horse => "Horse",
        }
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self::Cat
    }
}

impl Display for Runner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl FromStr for Runner {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "cat" => Ok(Self::Cat),
            "parrot" => Ok(Self::Parrot),
            "horse" => Ok(Self::Horse),
            other => Err(format!("unsupported runner: {other}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    System,
    Light,
    Dark,
}

impl Theme {
    pub const ALL: [Theme; 3] = [Theme::System, Theme::Light, Theme::Dark];

    pub fn label(self) -> &'static str {
        match self {
            Theme::System => "System",
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::System
    }
}

impl Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl FromStr for Theme {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "system" => Ok(Self::System),
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            other => Err(format!("unsupported theme: {other}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpeedSource {
    Cpu,
    Memory,
}

impl SpeedSource {
    pub const ALL: [SpeedSource; 2] = [SpeedSource::Cpu, SpeedSource::Memory];

    pub fn label(self) -> &'static str {
        match self {
            SpeedSource::Cpu => "CPU",
            SpeedSource::Memory => "Memory",
        }
    }
}

impl Default for SpeedSource {
    fn default() -> Self {
        Self::Cpu
    }
}

impl Display for SpeedSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl FromStr for SpeedSource {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "cpu" => Ok(Self::Cpu),
            "memory" => Ok(Self::Memory),
            other => Err(format!("unsupported speed source: {other}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FpsLimit {
    Fps20,
    Fps30,
    Fps40,
}

impl FpsLimit {
    pub const ALL: [FpsLimit; 3] = [FpsLimit::Fps20, FpsLimit::Fps30, FpsLimit::Fps40];

    pub fn rate(self) -> f32 {
        match self {
            FpsLimit::Fps20 => 0.5,
            FpsLimit::Fps30 => 0.75,
            FpsLimit::Fps40 => 1.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FpsLimit::Fps20 => "20 FPS",
            FpsLimit::Fps30 => "30 FPS",
            FpsLimit::Fps40 => "40 FPS",
        }
    }
}

impl Default for FpsLimit {
    fn default() -> Self {
        Self::Fps40
    }
}

impl Display for FpsLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl FromStr for FpsLimit {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_ascii_lowercase().replace(' ', "").as_str() {
            "20fps" | "fps20" => Ok(Self::Fps20),
            "30fps" | "fps30" => Ok(Self::Fps30),
            "40fps" | "fps40" => Ok(Self::Fps40),
            other => Err(format!("unsupported fps limit: {other}")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub runner: Runner,
    pub theme: Theme,
    pub speed_source: SpeedSource,
    pub fps_limit: FpsLimit,
    pub launch_at_startup: bool,
    pub first_launch: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            runner: Runner::default(),
            theme: Theme::default(),
            speed_source: SpeedSource::default(),
            fps_limit: FpsLimit::default(),
            launch_at_startup: false,
            first_launch: true,
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self> {
        let path = settings_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read settings from {}", path.display()))?;
        toml::from_str(&raw)
            .with_context(|| format!("failed to parse settings from {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create settings dir {}", parent.display()))?;
        }

        let raw = toml::to_string_pretty(self).context("failed to serialize settings")?;
        fs::write(&path, raw)
            .with_context(|| format!("failed to write settings to {}", path.display()))
    }
}

pub fn settings_path() -> PathBuf {
    platform_config_dir().join("RsCat").join("settings.toml")
}

fn platform_config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(value) = env::var_os("APPDATA") {
            return PathBuf::from(value);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support");
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(value) = env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(value);
        }
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(".config");
        }
    }

    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
