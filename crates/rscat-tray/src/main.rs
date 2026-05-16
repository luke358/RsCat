mod tray;

use anyhow::{Context, Result};
use rscat_core::{
    load_runner_frames, AnimationClock, AnimationPlan, FrameSet, MetricSampler, Runner, Settings,
    SpeedSource,
};
use rscat_ui::SettingsWindowHandle;
use slint::{ComponentHandle, Timer, TimerMode};
use std::{
    cell::RefCell,
    rc::Rc,
    str::FromStr,
    time::{Duration, Instant},
};
use tray::{TrayAction, TrayController};

fn main() -> Result<()> {
    #[cfg(target_os = "linux")]
    tray_icon::gtk::init().context("failed to initialize GTK for Linux tray support")?;

    let settings = Rc::new(RefCell::new(Settings::load()?));
    let initial_settings = settings.borrow().clone();
    let frames = Rc::new(RefCell::new(load_runner_frames(initial_settings.runner)?));
    let sampler = Rc::new(RefCell::new(MetricSampler::new()));
    let snapshot = Rc::new(RefCell::new(sampler.borrow().average()));
    let plan = Rc::new(RefCell::new(AnimationPlan::from_snapshot(
        &snapshot.borrow(),
        initial_settings.speed_source,
        initial_settings.fps_limit,
    )));

    let tray = Rc::new(RefCell::new(TrayController::new(
        &initial_settings,
        &frames.borrow(),
        &snapshot.borrow().tooltip(),
    )?));
    let settings_window: Rc<RefCell<Option<SettingsWindowHandle>>> = Rc::new(RefCell::new(None));

    install_fetch_timer(
        settings.clone(),
        sampler.clone(),
        snapshot.clone(),
        plan.clone(),
        tray.clone(),
        settings_window.clone(),
    );
    install_animation_timer(frames.clone(), plan.clone(), tray.clone());
    install_menu_timer(settings, frames, snapshot, plan, tray, settings_window);

    slint::run_event_loop_until_quit().context("failed to run UI event loop")
}

fn install_fetch_timer(
    settings: Rc<RefCell<Settings>>,
    sampler: Rc<RefCell<MetricSampler>>,
    snapshot: Rc<RefCell<rscat_core::MetricsSnapshot>>,
    plan: Rc<RefCell<AnimationPlan>>,
    tray: Rc<RefCell<TrayController>>,
    settings_window: Rc<RefCell<Option<SettingsWindowHandle>>>,
) {
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
        let sampled = sampler.borrow_mut().sample();
        *snapshot.borrow_mut() = sampled;

        let current_settings = settings.borrow().clone();
        *plan.borrow_mut() = AnimationPlan::from_snapshot(
            &sampled,
            current_settings.speed_source,
            current_settings.fps_limit,
        );

        let tooltip = sampled.tooltip();
        let _ = tray.borrow().set_tooltip(&tooltip);
        if let Some(window) = settings_window.borrow().as_ref() {
            rscat_ui::set_metrics(window, &tooltip);
        }
    });
    std::mem::forget(timer);
}

fn install_animation_timer(
    frames: Rc<RefCell<FrameSet>>,
    plan: Rc<RefCell<AnimationPlan>>,
    tray: Rc<RefCell<TrayController>>,
) {
    let clock = Rc::new(RefCell::new(AnimationClock::default()));
    let last_tick = Rc::new(RefCell::new(Instant::now()));

    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(25), move || {
        let interval = plan.borrow().interval;
        if last_tick.borrow().elapsed() < interval {
            return;
        }
        *last_tick.borrow_mut() = Instant::now();

        let frames = frames.borrow();
        let index = clock.borrow_mut().next_frame(frames.len());
        if let Some(frame) = frames.frame(index) {
            let _ = tray.borrow().set_icon(frame);
        }
    });
    std::mem::forget(timer);
}

fn install_menu_timer(
    settings: Rc<RefCell<Settings>>,
    frames: Rc<RefCell<FrameSet>>,
    snapshot: Rc<RefCell<rscat_core::MetricsSnapshot>>,
    plan: Rc<RefCell<AnimationPlan>>,
    tray: Rc<RefCell<TrayController>>,
    settings_window: Rc<RefCell<Option<SettingsWindowHandle>>>,
) {
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(100), move || {
        while let Some(action) = TrayController::poll_action() {
            match action {
                TrayAction::Quit => {
                    let _ = slint::quit_event_loop();
                }
                TrayAction::OpenSettings => {
                    let _ = show_or_raise_settings(
                        settings.clone(),
                        snapshot.clone(),
                        settings_window.clone(),
                        frames.clone(),
                        plan.clone(),
                        tray.clone(),
                    );
                }
                TrayAction::SetRunner(runner) => {
                    let _ = update_runner(&settings, &frames, &tray, runner);
                    if let Some(window) = settings_window.borrow().as_ref() {
                        rscat_ui::apply_settings(window, &settings.borrow());
                    }
                }
                TrayAction::SetSpeedSource(speed_source) => {
                    let _ = update_speed_source(&settings, &snapshot, &plan, &tray, speed_source);
                    if let Some(window) = settings_window.borrow().as_ref() {
                        rscat_ui::apply_settings(window, &settings.borrow());
                    }
                }
                TrayAction::SetFpsLimit(fps_limit) => {
                    let _ = update_fps_limit(&settings, &snapshot, &plan, &tray, fps_limit);
                    if let Some(window) = settings_window.borrow().as_ref() {
                        rscat_ui::apply_settings(window, &settings.borrow());
                    }
                }
            }
        }
    });
    std::mem::forget(timer);
}

fn show_or_raise_settings(
    settings: Rc<RefCell<Settings>>,
    snapshot: Rc<RefCell<rscat_core::MetricsSnapshot>>,
    settings_window: Rc<RefCell<Option<SettingsWindowHandle>>>,
    frames: Rc<RefCell<FrameSet>>,
    plan: Rc<RefCell<AnimationPlan>>,
    tray: Rc<RefCell<TrayController>>,
) -> Result<()> {
    if let Some(window) = settings_window.borrow().as_ref() {
        window.window().show()?;
        return Ok(());
    }

    let window =
        rscat_ui::create_settings_window(&settings.borrow(), &snapshot.borrow().tooltip())?;

    {
        let settings = settings.clone();
        let frames = frames.clone();
        let tray = tray.clone();
        let weak_window = window.as_weak();
        window.on_choose_runner(move |value| {
            if let Ok(runner) = Runner::from_str(value.as_str()) {
                let _ = update_runner(&settings, &frames, &tray, runner);
                if let Some(window) = weak_window.upgrade() {
                    rscat_ui::apply_settings(&window, &settings.borrow());
                }
            }
        });
    }

    {
        let settings = settings.clone();
        let snapshot = snapshot.clone();
        let plan = plan.clone();
        let tray = tray.clone();
        let weak_window = window.as_weak();
        window.on_choose_speed_source(move |value| {
            if let Ok(speed_source) = SpeedSource::from_str(value.as_str()) {
                let _ = update_speed_source(&settings, &snapshot, &plan, &tray, speed_source);
                if let Some(window) = weak_window.upgrade() {
                    rscat_ui::apply_settings(&window, &settings.borrow());
                }
            }
        });
    }

    {
        let settings = settings.clone();
        let snapshot = snapshot.clone();
        let plan = plan.clone();
        let tray = tray.clone();
        let weak_window = window.as_weak();
        window.on_choose_fps_limit(move |value| {
            if let Ok(fps_limit) = rscat_core::FpsLimit::from_str(value.as_str()) {
                let _ = update_fps_limit(&settings, &snapshot, &plan, &tray, fps_limit);
                if let Some(window) = weak_window.upgrade() {
                    rscat_ui::apply_settings(&window, &settings.borrow());
                }
            }
        });
    }

    window.show()?;
    *settings_window.borrow_mut() = Some(window);
    Ok(())
}

fn update_runner(
    settings: &Rc<RefCell<Settings>>,
    frames: &Rc<RefCell<FrameSet>>,
    tray: &Rc<RefCell<TrayController>>,
    runner: Runner,
) -> Result<()> {
    {
        let mut settings = settings.borrow_mut();
        settings.runner = runner;
        settings.save()?;
    }

    let next_frames = load_runner_frames(runner)?;
    tray.borrow_mut().set_runner_frames(&next_frames)?;
    *frames.borrow_mut() = next_frames;
    tray.borrow().refresh_menu(&settings.borrow())?;
    Ok(())
}

fn update_speed_source(
    settings: &Rc<RefCell<Settings>>,
    snapshot: &Rc<RefCell<rscat_core::MetricsSnapshot>>,
    plan: &Rc<RefCell<AnimationPlan>>,
    tray: &Rc<RefCell<TrayController>>,
    speed_source: SpeedSource,
) -> Result<()> {
    {
        let mut settings = settings.borrow_mut();
        settings.speed_source = speed_source;
        settings.save()?;
        *plan.borrow_mut() = AnimationPlan::from_snapshot(
            &snapshot.borrow(),
            settings.speed_source,
            settings.fps_limit,
        );
    }

    tray.borrow().refresh_menu(&settings.borrow())?;
    Ok(())
}

fn update_fps_limit(
    settings: &Rc<RefCell<Settings>>,
    snapshot: &Rc<RefCell<rscat_core::MetricsSnapshot>>,
    plan: &Rc<RefCell<AnimationPlan>>,
    tray: &Rc<RefCell<TrayController>>,
    fps_limit: rscat_core::FpsLimit,
) -> Result<()> {
    {
        let mut settings = settings.borrow_mut();
        settings.fps_limit = fps_limit;
        settings.save()?;
        *plan.borrow_mut() = AnimationPlan::from_snapshot(
            &snapshot.borrow(),
            settings.speed_source,
            settings.fps_limit,
        );
    }

    tray.borrow().refresh_menu(&settings.borrow())?;
    Ok(())
}
