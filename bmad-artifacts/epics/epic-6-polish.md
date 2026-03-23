# Epic 6: Polish and Release

## Goal
Harden, test on real hardware, and prepare for public release.

---

## Story 6.1: Error handling audit

### Acceptance criteria
- [ ] Zero `unwrap()` or `expect()` calls in src/ (enforced by clippy lint)
- [ ] All collector errors logged but don't crash the app
- [ ] Missing vcgencmd produces warning on first tab render, not every tick
- [ ] Terminal restore works even on panic (panic hook installed)
- [ ] Ctrl+C handled gracefully

---

## Story 6.2: CLI argument parsing

### Acceptance criteria
- [ ] `--interval` / `-i`: refresh interval in milliseconds (default 1000)
- [ ] `--tab` / `-t`: starting tab number (1-6, default 1)
- [ ] `--board`: force board type (pi5/pi4b/zero2w/auto, default auto)
- [ ] `--version` / `-V`: print version and exit
- [ ] `--help` / `-h`: print usage
- [ ] Uses clap derive macros

---

## Story 6.3: Real hardware testing

### Acceptance criteria
- [ ] Run `scripts/capture-sysfs.sh` on Pi 5, Pi 4B, Zero 2W
- [ ] Commit fixture snapshots to `tests/fixtures/`
- [ ] All unit tests pass using real fixture data
- [ ] Binary runs without errors on all three boards
- [ ] Verify displayed values match `vcgencmd` / `htop` / `free` output

---

## Story 6.4: Cross-compilation CI

### Acceptance criteria
- [ ] GitHub Actions workflow builds for aarch64-unknown-linux-gnu and armv7-unknown-linux-gnueabihf
- [ ] Clippy lint check runs on every push
- [ ] Tests run on every push
- [ ] Release workflow: on git tag, builds all targets, creates GitHub Release with binaries

---

## Story 6.5: Installation and packaging

### Acceptance criteria
- [ ] `scripts/install.sh` — downloads correct binary for architecture, installs to /usr/local/bin
- [ ] Prebuilt binaries attached to GitHub releases
- [ ] README documents installation via curl one-liner
- [ ] Binary is stripped and under 5MB for all targets
