# pitop — AI-Native Engineering Pipeline

## The right tool for each job

You have three AI coding CLIs, BMAD method, and OpenClaw at your disposal. Each has a sweet spot. The pipeline below assigns each tool to what it does best, rather than using one tool for everything.

---

## Tool roles

### Claude Code CLI — Primary builder (70% of work)

Claude Code is the backbone. It handles Rust best of the three CLIs, has the deepest context window for large files, and integrates natively with the BMAD method. Use it for:

- All Rust implementation (collectors, UI, app logic)
- Architecture decisions and refactoring
- Complex multi-file changes that need full project context
- BMAD agent interactions (analyst, architect, developer, SM personas)
- Writing and running tests
- Code review and PR-quality diffs

**Setup**: Install BMAD in the project root (`npx bmad-method install`), then run Claude Code from the same directory. It picks up the `.bmad/` context automatically.

### Codex CLI — Parallel task executor (20% of work)

Codex excels at well-scoped, isolated tasks that can run in parallel. Use it for:

- Generating boilerplate (Cargo.toml, CI configs, cross-compilation scripts)
- Writing documentation (README, man pages, CONTRIBUTING.md)
- Implementing isolated utility modules (ring buffer, format helpers, sysfs reader)
- Creating test fixtures and mock data files
- Bash scripts (install scripts, release packaging)

**Why not primary**: Codex runs sandboxed with network off by default and has a shorter context window. Great for self-contained units, not for large cross-file refactors.

### Gemini CLI — Research and validation (10% of work)

Gemini's strength is its massive context window and web-grounded search. Use it for:

- Validating sysfs paths across kernel versions
- Researching edge cases ("what happens to hwmon numbering after reboot?")
- Cross-referencing Raspberry Pi forum posts for hardware quirks
- Generating comprehensive test matrices
- Reviewing generated code for correctness against Linux kernel docs

### OpenClaw — Autonomous CI/CD monitor (always-on)

OpenClaw runs as a persistent background agent on your dev machine or a spare Pi. Configure it with skills to:

- Watch the git repo for new commits, auto-trigger `cargo clippy` and `cargo test`
- Cross-compile for all three targets on push, report build failures via Telegram/Discord
- Monitor a test Pi 5 over SSH: run the built binary, capture screenshots, report crashes
- Track GitHub issues and auto-label them based on content
- Run nightly benchmarks (memory usage, CPU overhead) and alert on regressions

**Setup**: Create a custom OpenClaw skill (`pitop-ci`) that wraps your build/test/deploy scripts. Use the Heartbeat/cron system for scheduled monitoring.

### BMAD Method — Process orchestration (wraps everything)

BMAD provides the scaffolding that keeps all the AI tools aligned. Without it, each tool drifts and you lose coherence. The key BMAD artifacts for pitop:

| BMAD Phase | Artifact | Who creates it | Tool |
|-----------|----------|---------------|------|
| Analysis | `product-brief.md` | You + Analyst agent | Claude Code |
| Planning | `prd.md` (PRD) | PM agent | Claude Code |
| Solutioning | `architecture.md` | Architect agent | Claude Code |
| Solutioning | `tech-stack.md` | Architect agent | Claude Code |
| Planning | `epics/` directory with stories | SM agent | Claude Code |
| Implementation | Code per story | Developer agent | Claude Code + Codex |
| QA | Tests per story | QA agent | Claude Code |

---

## The pipeline: end to end

### Phase 0 — Repository bootstrap (30 min)

```bash
# 1. Create repo
mkdir pitop && cd pitop && git init

# 2. Install BMAD
npx bmad-method install
# Select: Standard Track (not Quick, not Enterprise)
# Enable modules: default set is fine

# 3. Scaffold Rust project
cargo init --name pitop
# Add basic Cargo.toml dependencies manually or via Codex

# 4. Commit the skeleton
git add -A && git commit -m "chore: initial scaffold with BMAD"
```

### Phase 1 — BMAD planning (2-3 hours)

This is where you invest upfront to save days later. Every minute here prevents an hour of rework.

```
Step 1: Load Analyst agent in Claude Code
        → Interactive interview about pitop
        → Output: bmad-artifacts/product-brief.md

Step 2: Load PM agent
        → Reads product-brief.md
        → Output: bmad-artifacts/prd.md
        → Contains: FRs, NFRs, epics, acceptance criteria

Step 3: Load Architect agent
        → Reads prd.md + our design-research.md
        → Output: bmad-artifacts/architecture.md
        → Contains: module breakdown, data flow, dependency decisions
        → Output: bmad-artifacts/tech-stack.md

Step 4: Load SM (Scrum Master) agent
        → Reads PRD + architecture
        → Output: bmad-artifacts/epics/epic-1-board-detection.md
                   bmad-artifacts/epics/epic-2-core-collectors.md
                   bmad-artifacts/epics/epic-3-overview-ui.md
                   bmad-artifacts/epics/epic-4-tabs.md
                   bmad-artifacts/epics/epic-5-pi-specific.md
                   bmad-artifacts/epics/epic-6-polish.md
        → Each epic has 3-5 stories with clear acceptance criteria

Step 5: Git commit all artifacts
        git add bmad-artifacts/ && git commit -m "docs: BMAD planning complete"
```

**Why this matters for AI coding**: Every story file becomes a self-contained context document that you can feed to any AI tool. The story says exactly what to build, what the acceptance criteria are, and what architecture constraints apply. This eliminates the "context collapse" problem where the AI forgets what you're building halfway through.

### Phase 2 — Foundation sprint (Claude Code, 1-2 days)

Work through stories in order. Each story follows this loop:

```
┌─────────────────────────────────────────────┐
│  BMAD Dev Loop (per story)                  │
│                                             │
│  1. Load story context into Claude Code     │
│     "Implement story: [paste story.md]"     │
│                                             │
│  2. Claude Code writes implementation       │
│     → Creates/modifies files                │
│     → Runs cargo check / cargo clippy       │
│                                             │
│  3. Claude Code writes tests                │
│     → Unit tests for the module             │
│     → Runs cargo test                       │
│                                             │
│  4. You review the diff                     │
│     → Accept, request changes, or refine    │
│                                             │
│  5. Git commit with story reference          │
│     "feat(board): detect Pi 5 via device-   │
│      tree [story-1.1]"                      │
│                                             │
│  6. Mark story complete in epic file        │
└─────────────────────────────────────────────┘
```

**Parallel work with Codex**: While Claude Code works on core implementation stories, use Codex in a separate terminal for:
- `codex "Create a GitHub Actions workflow that cross-compiles pitop for aarch64 and armv7"`
- `codex "Write a comprehensive README.md for a Rust TUI called pitop that monitors Raspberry Pi systems"`
- `codex "Generate a man page for pitop with all CLI flags documented"`

### Phase 3 — UI sprint (Claude Code, 1-2 days)

The TUI is where Claude Code shines — it can hold the entire ratatui widget tree in context and refactor layout across files. Feed it the overview tab story, let it build the gauge/sparkline layout, then iterate visually by running the binary on your Pi over SSH.

**Iteration loop**:
```
Claude Code builds UI code
  → You cargo build --release --target aarch64-unknown-linux-gnu
  → scp to Pi
  → Run over SSH, take screenshot or describe what you see
  → Feed description back to Claude Code
  → Repeat until polished
```

OpenClaw can automate the middle steps: set up a skill that watches for new builds, auto-deploys to the Pi, runs for 5 seconds, captures terminal output, and sends it back to you.

### Phase 4 — Pi-specific features (Claude Code + Gemini, 1-2 days)

This is where Gemini earns its keep. For each Pi-specific collector (PMIC, fan, PCIe, PoE):

1. **Gemini**: Research the exact sysfs paths and parsing format for this feature. Ask it to find edge cases, kernel version differences, and failure modes.
2. **Claude Code**: Implement the collector with the researched paths. Write tests that use mock sysfs data.
3. **Test on real hardware**: Deploy to your Pi 5 and Zero 2W. Capture real sysfs output for test fixtures.

### Phase 5 — Integration and polish (all tools, 1-2 days)

- **Claude Code**: End-to-end integration, error handling, graceful degradation when features aren't available
- **Codex**: Release packaging scripts, install scripts, changelog generation
- **Gemini**: Final review pass — feed it the entire codebase and ask for correctness issues
- **OpenClaw**: Automated nightly builds, regression monitoring, deploy to test Pis

---

## Project file structure (with BMAD)

```
pitop/
├── .bmad/                          # BMAD method config (auto-generated)
│   ├── agents/                     # Agent persona definitions
│   └── config.yml                  # BMAD settings
│
├── bmad-artifacts/                 # All planning docs (version-controlled)
│   ├── product-brief.md
│   ├── prd.md
│   ├── architecture.md
│   ├── tech-stack.md
│   └── epics/
│       ├── epic-1-board-detection.md
│       ├── epic-2-core-collectors.md
│       ├── epic-3-overview-ui.md
│       ├── epic-4-tabs.md
│       ├── epic-5-pi-specific.md
│       └── epic-6-polish.md
│
├── CLAUDE.md                       # Claude Code project instructions
│                                   # (arch constraints, coding style,
│                                   #  test expectations, do/don't rules)
│
├── src/                            # Rust source (see design doc)
│   ├── main.rs
│   ├── app.rs
│   ├── board/
│   ├── collectors/
│   ├── ui/
│   └── util/
│
├── tests/
│   ├── fixtures/                   # Real sysfs captures from each Pi model
│   │   ├── pi5/
│   │   ├── pi4b/
│   │   └── zero2w/
│   └── integration/
│
├── .github/
│   └── workflows/
│       ├── ci.yml                  # Lint + test on push
│       ├── cross-build.yml         # Matrix build for 3 targets
│       └── release.yml             # Auto-release on tag
│
├── scripts/
│   ├── capture-sysfs.sh            # Run on real Pi to capture test fixtures
│   ├── deploy-test.sh              # scp + run on test Pi
│   └── benchmark.sh                # Memory/CPU overhead measurement
│
├── Cargo.toml
├── README.md
├── CHANGELOG.md
└── config/
    └── default.toml                # Default thresholds and refresh rates
```

---

## The CLAUDE.md file — the most important file

This is the project-level instruction file that Claude Code reads on every invocation. It's what keeps the AI aligned across sessions. Here's what it should contain:

```markdown
# pitop — Claude Code instructions

## Project
Rust TUI system monitor for Raspberry Pi 5, 4B, and Zero 2W.
Uses ratatui + crossterm. Targets aarch64 and armv7 Linux.

## Architecture rules
- All system data read from procfs/sysfs directly. No sysinfo crate.
- vcgencmd calls go through util/vcgencmd.rs (async, cached, with timeout).
- Board detection at startup determines which collectors are active.
- UI uses lazy refresh: only the active tab's expensive collectors run.
- Ring buffers for all sparkline history (fixed 60-sample window).

## Coding standards
- No unwrap() in production code. Use anyhow for error handling.
- All sysfs reads must handle ENOENT gracefully (feature not available).
- Collector trait: fn collect(&mut self) -> Result<()>
- Tests use fixtures from tests/fixtures/{pi5,pi4b,zero2w}/.
- Clippy must pass with no warnings.

## Do not
- Do not use the sysinfo crate.
- Do not hardcode hwmon numbers (they change across reboots).
- Do not shell out for data that's available via sysfs.
- Do not use std::process::Command — use tokio::process::Command.

## Current sprint
See bmad-artifacts/epics/ for current stories.
```

---

## Realistic timeline

| Day | Focus | Primary tool | Parallel tool |
|-----|-------|-------------|---------------|
| 1 morning | BMAD planning (brief → PRD → arch → stories) | Claude Code | — |
| 1 afternoon | Board detection + core collectors (CPU, mem, thermal) | Claude Code | Codex: CI/CD, README |
| 2 | Remaining collectors + process scanner | Claude Code | Codex: install scripts |
| 3 | Overview tab + tab framework | Claude Code | Gemini: sysfs edge cases |
| 4 | All 6 tabs complete | Claude Code | OpenClaw: auto-deploy to Pi |
| 5 | Pi 5 specifics (PMIC, fan, PCIe, PoE) | Claude Code | Gemini: kernel compat |
| 6 | Pi 4B + Zero 2W testing, polish, release | Claude Code | Codex: packaging + changelog |
| 7 | Buffer / bug fixes / real-hardware validation | All | OpenClaw: regression watch |

Seven days to a polished v1.0 with prebuilt binaries for three boards.

---

## Key principles for AI-native development

1. **Docs before code** — BMAD artifacts aren't bureaucracy, they're context engineering. Every story.md you write saves 10 prompts later.

2. **One tool, one job** — Don't ask Claude Code to research kernel docs. Don't ask Gemini to write Rust. Don't ask Codex to refactor across 8 files. Play to strengths.

3. **Git is the integration bus** — All tools commit to the same repo. The git log is the source of truth, not any chat transcript.

4. **Test fixtures from real hardware** — Run `capture-sysfs.sh` on each Pi model to capture real `/proc` and `/sys` snapshots. These become your test data. AI tools can't simulate a real Pi 5 PMIC output.

5. **CLAUDE.md is your constitution** — Update it as architecture evolves. It's the single document that prevents context collapse across coding sessions.

6. **OpenClaw as your tireless QA** — Set it up once to auto-build, auto-deploy, auto-test. It catches regressions while you sleep.

7. **Human reviews every merge** — AI writes, you review. Never auto-merge AI-generated code. The diff review is where you catch hallucinated sysfs paths and subtle logic errors.
