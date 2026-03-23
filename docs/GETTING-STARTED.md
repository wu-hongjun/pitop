# Getting Started with pitop Development

This guide gets you from zero to your first Claude Code session in 10 minutes.

## Prerequisites

- **Rust toolchain**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Cross-compilation toolchains** (for building on x86 for Pi):
  ```bash
  sudo apt install gcc-aarch64-linux-gnu gcc-arm-linux-gnueabihf
  rustup target add aarch64-unknown-linux-gnu armv7-unknown-linux-gnueabihf
  ```
- **Claude Code CLI**: `npm install -g @anthropic-ai/claude-code`
- **A Raspberry Pi** with SSH access (for testing)

## Optional tools

- **BMAD Method**: `npx bmad-method install` (run inside this repo)
- **Codex CLI**: For parallel side tasks
- **Gemini CLI**: For research questions

## Step 1: Verify the repo builds

```bash
cd pitop
cargo check
```

This should succeed with no errors. The binary doesn't do anything yet —
that's what Claude Code is for.

## Step 2: Capture test fixtures from your Pi

Copy the capture script to each Pi and run it:

```bash
scp scripts/capture-sysfs.sh pi@your-pi-5:~/
ssh pi@your-pi-5 "chmod +x capture-sysfs.sh && ./capture-sysfs.sh"
scp -r pi@your-pi-5:~/pi5 tests/fixtures/pi5/
```

Repeat for Pi 4B and Zero 2W. These fixtures become your test data.

## Step 3: Start Claude Code

```bash
claude
```

Claude Code automatically reads `CLAUDE.md` for project context.

## Step 4: Begin with Epic 1

Type into Claude Code:

```
Read bmad-artifacts/epics/epic-1-board-detection.md and implement
Story 1.1: Read device-tree compatible string. Follow the acceptance
criteria exactly.
```

Claude Code will create the board detection module, write tests, and
run them. Review the diff, then commit:

```bash
git add -A && git commit -m "feat(board): detect Pi model from device-tree [story-1.1]"
```

## Step 5: Continue through the stories

Work through stories in order: Epic 1 → Epic 2 → Epic 3 → etc.
Each story is self-contained with clear acceptance criteria.

See `docs/how-you-build.md` for the full workflow guide.

## Testing on real hardware

After building a few collectors:

```bash
# Build for your Pi 5
cargo build --release --target aarch64-unknown-linux-gnu

# Deploy and run
./scripts/deploy-test.sh pi@your-pi-5 aarch64
```

## Project structure

```
CLAUDE.md                    ← Claude Code reads this every session
bmad-artifacts/
  product-brief.md           ← What we're building and why
  epics/
    epic-1-board-detection   ← Start here
    epic-2-core-collectors   ← Then here
    epic-3-overview-ui       ← Then here
    epic-4-tabs              ← Then here
    epic-5-pi-specific       ← Then here
    epic-6-polish            ← Finally here
docs/
  design-research.md         ← All sysfs paths, hardware research
  engineering-pipeline.md    ← How the AI tools fit together
  how-you-build.md           ← What you literally type day by day
config/
  default.toml               ← Threshold defaults
scripts/
  capture-sysfs.sh           ← Run on Pi to get test fixtures
  deploy-test.sh             ← Build + deploy + run on Pi
src/
  main.rs                    ← Entry point (placeholder for now)
```
