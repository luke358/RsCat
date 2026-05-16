# RsCat

RsCat is a cross-platform Rust desktop tray app inspired by RunCat 365.

The current implementation is an MVP scaffold:

- Rust core crate for settings, metrics, animation, and embedded runner assets.
- Cross-platform tray crate based on `tray-icon`.
- Lightweight settings window crate based on Slint.

## Development

```bash
cargo check
cargo run -p rscat-tray
```

## Platform Notes

- Windows and macOS are first-class targets.
- Linux tray support depends on the desktop environment and AppIndicator/KStatusNotifier support.
- GPU, temperature, launch-at-startup integration, packaging, and the endless game are intentionally outside the first MVP.
