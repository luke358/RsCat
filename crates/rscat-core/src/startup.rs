use anyhow::Result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupState {
    Unsupported,
    Disabled,
    Enabled,
}

pub fn get_startup_state() -> StartupState {
    StartupState::Unsupported
}

pub fn set_startup_enabled(_enabled: bool) -> Result<StartupState> {
    Ok(StartupState::Unsupported)
}
