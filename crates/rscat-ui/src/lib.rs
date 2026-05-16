use anyhow::Result;
use rscat_core::Settings;
use slint::{CloseRequestResponse, ComponentHandle, SharedString};

slint::slint! {
    component OptionButton inherits Rectangle {
        in property <string> text;
        callback clicked;

        height: 32px;
        border-radius: 4px;
        border-width: 1px;
        border-color: touch.pressed ? rgb(37, 99, 235) : rgb(207, 214, 224);
        background: touch.pressed ? rgb(219, 234, 254) : (touch.has-hover ? rgb(238, 244, 255) : #ffffff);

        Text {
            text: root.text;
            color: #111827;
            font-size: 13px;
            font-weight: 600;
            horizontal-alignment: center;
            vertical-alignment: center;
        }

        touch := TouchArea {
            clicked => {
                root.clicked();
            }
        }
    }

    export component SettingsWindow inherits Window {
        in property <string> runner_label;
        in property <string> speed_source_label;
        in property <string> fps_limit_label;
        in property <string> metrics_label;
        callback choose_runner(string);
        callback choose_speed_source(string);
        callback choose_fps_limit(string);

        title: "RsCat Settings";
        width: 420px;
        height: 340px;
        background: #f7f8fa;

        VerticalLayout {
            padding: 20px;
            spacing: 14px;

            Text {
                text: "RsCat";
                font-size: 26px;
                font-weight: 700;
                color: #111827;
            }

            Text {
                text: root.metrics_label;
                color: #374151;
                font-size: 13px;
            }

            Rectangle {
                height: 1px;
                background: #d1d5db;
            }

            Text {
                text: "Runner: " + root.runner_label;
                color: #111827;
                font-size: 14px;
                font-weight: 600;
            }

            HorizontalLayout {
                spacing: 8px;
                OptionButton { text: "Cat"; clicked => { root.choose_runner("cat"); } }
                OptionButton { text: "Parrot"; clicked => { root.choose_runner("parrot"); } }
                OptionButton { text: "Horse"; clicked => { root.choose_runner("horse"); } }
            }

            Text {
                text: "Speed: " + root.speed_source_label;
                color: #111827;
                font-size: 14px;
                font-weight: 600;
            }

            HorizontalLayout {
                spacing: 8px;
                OptionButton { text: "CPU"; clicked => { root.choose_speed_source("cpu"); } }
                OptionButton { text: "Memory"; clicked => { root.choose_speed_source("memory"); } }
            }

            Text {
                text: "FPS Limit: " + root.fps_limit_label;
                color: #111827;
                font-size: 14px;
                font-weight: 600;
            }

            HorizontalLayout {
                spacing: 8px;
                OptionButton { text: "20"; clicked => { root.choose_fps_limit("20fps"); } }
                OptionButton { text: "30"; clicked => { root.choose_fps_limit("30fps"); } }
                OptionButton { text: "40"; clicked => { root.choose_fps_limit("40fps"); } }
            }
        }
    }
}

pub type SettingsWindowHandle = SettingsWindow;

pub fn create_settings_window(
    settings: &Settings,
    metrics_text: &str,
) -> Result<SettingsWindowHandle> {
    let window = SettingsWindow::new()?;
    window
        .window()
        .on_close_requested(|| CloseRequestResponse::HideWindow);
    apply_settings(&window, settings);
    window.set_metrics_label(SharedString::from(metrics_text));
    Ok(window)
}

pub fn apply_settings(window: &SettingsWindowHandle, settings: &Settings) {
    window.set_runner_label(SharedString::from(settings.runner.label()));
    window.set_speed_source_label(SharedString::from(settings.speed_source.label()));
    window.set_fps_limit_label(SharedString::from(settings.fps_limit.label()));
}

pub fn set_metrics(window: &SettingsWindowHandle, metrics_text: &str) {
    window.set_metrics_label(SharedString::from(metrics_text));
}
