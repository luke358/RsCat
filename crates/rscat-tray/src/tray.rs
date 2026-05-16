use anyhow::{Context, Result};
use rscat_core::{FpsLimit, FrameSet, Runner, RunnerFrame, Settings, SpeedSource};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
    Icon, TrayIcon, TrayIconBuilder,
};

const MENU_OPEN_SETTINGS: &str = "open-settings";
const MENU_QUIT: &str = "quit";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrayAction {
    OpenSettings,
    SetRunner(Runner),
    SetSpeedSource(SpeedSource),
    SetFpsLimit(FpsLimit),
    Quit,
}

pub struct TrayController {
    tray: TrayIcon,
    menu: Menu,
    runner_menu: Submenu,
    speed_menu: Submenu,
    fps_menu: Submenu,
}

impl TrayController {
    pub fn new(settings: &Settings, frames: &FrameSet, tooltip: &str) -> Result<Self> {
        let menu = Menu::new();
        let runner_menu = Submenu::new("Runner", true);
        let speed_menu = Submenu::new("Speed Source", true);
        let fps_menu = Submenu::new("FPS Limit", true);

        let controller_menu_items = build_static_items()?;
        let open_settings = &controller_menu_items.0;
        let quit = &controller_menu_items.1;

        append_runner_items(&runner_menu, settings)?;
        append_speed_items(&speed_menu, settings)?;
        append_fps_items(&fps_menu, settings)?;

        menu.append(open_settings)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&runner_menu)?;
        menu.append(&speed_menu)?;
        menu.append(&fps_menu)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(quit)?;

        let first_icon = frames
            .frame(0)
            .map(icon_from_frame)
            .transpose()?
            .context("runner has no frames")?;

        let tray = TrayIconBuilder::new()
            .with_tooltip(tooltip)
            .with_icon(first_icon)
            .with_icon_as_template(tray_icon_is_template())
            .with_menu(Box::new(menu.clone()))
            .with_menu_on_left_click(false)
            .build()?;

        Ok(Self {
            tray,
            menu,
            runner_menu,
            speed_menu,
            fps_menu,
        })
    }

    pub fn poll_action() -> Option<TrayAction> {
        let event = MenuEvent::receiver().try_recv().ok()?;
        let id = event.id.as_ref();

        if id == MENU_OPEN_SETTINGS {
            return Some(TrayAction::OpenSettings);
        }
        if id == MENU_QUIT {
            return Some(TrayAction::Quit);
        }
        if let Some(value) = id.strip_prefix("runner:") {
            return value.parse().ok().map(TrayAction::SetRunner);
        }
        if let Some(value) = id.strip_prefix("speed:") {
            return value.parse().ok().map(TrayAction::SetSpeedSource);
        }
        if let Some(value) = id.strip_prefix("fps:") {
            return value.parse().ok().map(TrayAction::SetFpsLimit);
        }

        None
    }

    pub fn set_icon(&self, frame: &RunnerFrame) -> Result<()> {
        self.tray
            .set_icon_with_as_template(Some(icon_from_frame(frame)?), tray_icon_is_template())?;
        Ok(())
    }

    pub fn set_tooltip(&self, tooltip: &str) -> Result<()> {
        self.tray.set_tooltip(Some(tooltip))?;
        Ok(())
    }

    pub fn set_runner_frames(&mut self, frames: &FrameSet) -> Result<()> {
        if let Some(frame) = frames.frame(0) {
            self.set_icon(frame)?;
        }
        Ok(())
    }

    pub fn refresh_menu(&self, settings: &Settings) -> Result<()> {
        self.runner_menu.remove_at(0);
        self.runner_menu.remove_at(0);
        self.runner_menu.remove_at(0);
        self.speed_menu.remove_at(0);
        self.speed_menu.remove_at(0);
        self.fps_menu.remove_at(0);
        self.fps_menu.remove_at(0);
        self.fps_menu.remove_at(0);

        append_runner_items(&self.runner_menu, settings)?;
        append_speed_items(&self.speed_menu, settings)?;
        append_fps_items(&self.fps_menu, settings)?;
        let _ = self.menu.items();
        Ok(())
    }
}

fn build_static_items() -> Result<(MenuItem, MenuItem)> {
    Ok((
        MenuItem::with_id(MENU_OPEN_SETTINGS, "Open Settings", true, None),
        MenuItem::with_id(MENU_QUIT, "Quit", true, None),
    ))
}

fn append_runner_items(menu: &Submenu, settings: &Settings) -> Result<()> {
    for runner in Runner::ALL {
        let label = selected_label(runner.label(), runner == settings.runner);
        menu.append(&MenuItem::with_id(
            format!("runner:{}", runner.as_id()),
            label,
            true,
            None,
        ))?;
    }
    Ok(())
}

fn append_speed_items(menu: &Submenu, settings: &Settings) -> Result<()> {
    for source in SpeedSource::ALL {
        let label = selected_label(source.label(), source == settings.speed_source);
        menu.append(&MenuItem::with_id(
            format!("speed:{}", source.label().to_ascii_lowercase()),
            label,
            true,
            None,
        ))?;
    }
    Ok(())
}

fn append_fps_items(menu: &Submenu, settings: &Settings) -> Result<()> {
    for limit in FpsLimit::ALL {
        let label = selected_label(limit.label(), limit == settings.fps_limit);
        let id = limit.label().to_ascii_lowercase().replace(' ', "");
        menu.append(&MenuItem::with_id(format!("fps:{id}"), label, true, None))?;
    }
    Ok(())
}

fn selected_label(label: &str, selected: bool) -> String {
    if selected {
        format!("[x] {label}")
    } else {
        format!("[ ] {label}")
    }
}

fn icon_from_frame(frame: &RunnerFrame) -> Result<Icon> {
    Ok(Icon::from_rgba(
        icon_rgba(frame),
        frame.width,
        frame.height,
    )?)
}

#[cfg(target_os = "macos")]
fn tray_icon_is_template() -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
fn tray_icon_is_template() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn icon_rgba(frame: &RunnerFrame) -> Vec<u8> {
    frame
        .rgba
        .chunks_exact(4)
        .flat_map(|pixel| {
            let alpha = pixel[3];
            let mask_alpha = if alpha == 0 { 0 } else { alpha.max(180) };
            [0, 0, 0, mask_alpha]
        })
        .collect()
}

#[cfg(not(target_os = "macos"))]
fn icon_rgba(frame: &RunnerFrame) -> Vec<u8> {
    frame.rgba.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_template_icon_keeps_alpha_and_removes_color() {
        let frame = RunnerFrame {
            rgba: vec![255, 0, 0, 255, 1, 2, 3, 0, 4, 5, 6, 12],
            width: 3,
            height: 1,
        };

        assert_eq!(
            icon_rgba(&frame),
            vec![0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 180]
        );
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn non_macos_icon_keeps_original_rgba() {
        let frame = RunnerFrame {
            rgba: vec![255, 0, 0, 255],
            width: 1,
            height: 1,
        };

        assert_eq!(icon_rgba(&frame), frame.rgba);
    }
}
