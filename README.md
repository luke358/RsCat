# RsCat

RsCat is a cross-platform Rust desktop tray app inspired by RunCat 365.

The current implementation is an MVP:

- Rust core crate for settings, metrics, animation, and embedded runner assets.
- Cross-platform tray crate based on `tray-icon`.
- Lightweight settings window crate based on Slint.

## Demo

![RsCat tray animation](docs/images/demo.gif)

## Development

```bash
cargo check
cargo run -p rscat-tray
```

## Release

Create and push a version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions will build and publish a GitHub Release with:

- macOS Apple Silicon `.app.zip`
- macOS Intel `.app.zip`
- Windows x64 `.exe.zip`
- Linux x64 `.tar.gz`

The first release pipeline is unsigned. macOS notarization, Windows signing, Linux desktop files, and installer formats are separate packaging steps.

## macOS Gatekeeper

Current macOS release builds are unsigned and not notarized. If macOS shows:

```text
"RsCat" is damaged and can't be opened. You should move it to the Trash.
```

remove the quarantine attribute from the extracted app:

```bash
xattr -dr com.apple.quarantine /Applications/RsCat.app
```

If the app is still in Downloads, use the actual path:

```bash
xattr -dr com.apple.quarantine ~/Downloads/RsCat.app
```

This is only for local testing. Public distribution should use Developer ID signing, Apple notarization, and stapling.

## Platform Notes

- Windows and macOS are first-class targets.
- Linux tray support depends on the desktop environment, GTK event loop compatibility, and AppIndicator/KStatusNotifier support.
- GPU, temperature, launch-at-startup integration, packaging, and the endless game are intentionally outside the first MVP.
