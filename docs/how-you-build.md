# pitop — How You Actually Build This

A practical guide for a human using Claude Code as the orchestrator,
with BMAD, Codex, Gemini, and OpenClaw as supporting tools.

---

## Your setup (one-time, ~20 minutes)

Open a terminal on your dev machine (Mac/Linux, not the Pi).

```bash
# 1. Create the project
mkdir pitop && cd pitop && git init

# 2. Install BMAD method into the project
npx bmad-method install
# It will ask questions. Choose:
#   - Standard Track (not Quick, not Enterprise)
#   - Accept defaults for everything else
# This creates a .bmad/ folder with agent definitions

# 3. Initialize Rust project
cargo init --name pitop

# 4. Commit the skeleton
git add -A && git commit -m "chore: initial project scaffold"
```

Now you have a project with BMAD agents ready and a Rust skeleton.

---

## How BMAD actually works (the 2-minute version)

BMAD gives Claude Code different "hats" to wear. Instead of saying
"build me a system monitor," you walk through roles in order:

```
Analyst    → Interviews you about what you want
PM         → Writes a formal requirements doc from the interview
Architect  → Designs the technical solution from the requirements
SM         → Breaks the design into small, buildable stories
Developer  → Builds each story one at a time
```

Each role produces a markdown file. Those files become the context
that keeps every subsequent AI interaction aligned. That's the
whole trick — structured docs prevent context collapse.

You invoke these roles by typing natural language into Claude Code.
There are no special commands to memorize.

---

## Day 1: Planning (your morning)

You sit at your terminal with Claude Code open in the pitop directory.

### Step 1 — The interview (~20 min)

You type into Claude Code:

```
Load the BMAD analyst agent. I want to build a Rust TUI called pitop
that monitors Raspberry Pi system resources. I have a Pi 5, Pi 4B,
and Zero 2W. The Pi 5 has PoE and PCIe with an NVMe drive. I want
something like mactop but for Pi hardware.
```

Claude Code (wearing the Analyst hat) will interview you. It asks
things like:

  - "Is this a single-user SSH tool or multi-user?"
  - "What Pi-specific hardware do you want to surface?"
  - "Do you need process management or just monitoring?"

Answer in plain English. When it's done, it writes
`bmad-artifacts/product-brief.md` to your project.

**What you do**: Answer questions honestly. Push back if it
overcomplicates things. Say "keep it simple" freely.

### Step 2 — The requirements doc (~15 min)

You type:

```
Now load the PM agent. Read the product brief and write a PRD.
```

Claude Code reads the brief, asks a few clarifying questions,
then writes `bmad-artifacts/prd.md`. This contains functional
requirements, non-functional requirements, and a list of epics.

**What you do**: Read the PRD. If something's wrong or missing,
just say so: "Add PCIe gen detection to the requirements" or
"I don't need a config file for v1."

### Step 3 — The architecture (~20 min)

You type:

```
Load the architect agent. Also read the file pitop-design-research.md
that I have in the project root — it contains all the sysfs paths and
hardware research. Design the architecture based on the PRD and research.
```

(You drop the design-research.md we made earlier into the project root.)

Claude Code produces `bmad-artifacts/architecture.md` with the module
structure, data flow, and dependency decisions.

**What you do**: Review the module breakdown. Ask questions like
"Why not use sysinfo crate?" and let it explain. Push for simplicity.

### Step 4 — The stories (~20 min)

You type:

```
Load the SM agent. Break the epics from the PRD into implementable
stories. Each story should be small enough for one Claude Code session.
Include acceptance criteria for each.
```

Claude Code creates story files in `bmad-artifacts/epics/`. Each story
looks roughly like:

```markdown
# Story 1.1: Board detection

## Description
Detect whether we're running on Pi 5, Pi 4B, or Zero 2W by reading
/proc/device-tree/compatible.

## Acceptance criteria
- [ ] Reads /proc/device-tree/compatible at startup
- [ ] Returns Pi5, Pi4B, Zero2W, or Unknown enum variant
- [ ] Falls back to /sys/firmware/devicetree/base/model if needed
- [ ] Unknown boards still run with generic Linux collectors
- [ ] Unit test with mock device-tree content

## Architecture notes
See architecture.md section 3.1. Uses board/ module.
```

**What you do**: Read the stories. Reorder if needed. Merge stories
that are too small, split ones that are too big. Commit everything:

```bash
git add bmad-artifacts/ && git commit -m "docs: BMAD planning complete"
```

### Step 5 — Write CLAUDE.md (~10 min)

This is the most important file in your project. You type:

```
Create a CLAUDE.md file in the project root that captures our
architecture decisions, coding standards, and constraints.
Reference the architecture doc. This file will be read by Claude
Code on every future session.
```

Claude Code writes it. You review and tweak. This file persists across
every future Claude Code session — it's your "constitution."

**Total planning time: ~90 minutes. This saves days.**

---

## Day 1 afternoon + Day 2: Building

Now you switch to implementation. Here's what your workflow looks like,
hour by hour.

### The basic loop (you'll do this 15-20 times)

```
You type:
  "Implement story 1.1: Board detection. Here are the acceptance
   criteria: [paste from story file]"

Claude Code:
  - Reads CLAUDE.md for constraints
  - Creates src/board/mod.rs, src/board/pi5.rs, etc.
  - Writes the implementation
  - Runs cargo check and cargo clippy
  - Writes unit tests
  - Runs cargo test
  - Shows you the diff

You:
  - Read the code (this is critical — never skip this)
  - Say "looks good" or "the hwmon path shouldn't be hardcoded,
    enumerate /sys/class/hwmon/ instead"
  - Once satisfied: git add && git commit

Move to next story.
```

### Running things in parallel

While Claude Code works on story 1.2 (CPU collector), open a second
terminal and use Codex for isolated tasks:

```bash
# In terminal 2
codex "Create a GitHub Actions CI workflow for this Rust project.
It should: lint with clippy, run tests, and cross-compile for
aarch64-unknown-linux-gnu and armv7-unknown-linux-gnueabihf.
Put it in .github/workflows/ci.yml"
```

Or:

```bash
codex "Write a shell script called scripts/capture-sysfs.sh that,
when run on a Raspberry Pi, captures snapshots of /proc/stat,
/proc/meminfo, /proc/net/dev, /sys/class/thermal/thermal_zone0/temp,
and the output of vcgencmd get_throttled, vcgencmd measure_temp.
Save everything to a directory named after the board model."
```

Codex writes the file, you review, commit.

### When to use Gemini

You hit a question that needs research, not code:

```bash
gemini "On Raspberry Pi OS Bookworm with kernel 6.6+, what is the
exact hwmon device name and numbering for the RP1 ADC temperature
sensor? Does the hwmon number change across reboots? How should I
discover it programmatically?"
```

Gemini searches, reads forum posts, gives you the answer. You feed
that answer back into Claude Code as context for the next story.

### Using Claude Code subagents for parallel work

For a bigger push, you can ask Claude Code to spawn subagents:

```
I need to implement 3 independent collector modules in parallel:
- src/collectors/memory.rs (reads /proc/meminfo)
- src/collectors/network.rs (reads /proc/net/dev)
- src/collectors/disk.rs (reads /proc/diskstats)

These modules have no dependencies on each other. Spawn subagents
to implement all three simultaneously. Each should include unit tests
using fixture data from tests/fixtures/pi5/.
```

Claude Code spawns 3 subagents, each working in its own context.
You review the results when they finish.

---

## Day 3-4: The UI

The TUI is the most iterative part. Your loop changes slightly:

```
You type:
  "Implement the overview tab. It should show: board name in a
   header, per-core CPU gauges, memory gauge with sparkline,
   temperature reading, throttle status indicator, and network
   throughput. Use ratatui's Layout, Gauge, Sparkline, and
   Paragraph widgets."

Claude Code writes the UI code.

You:
  cargo build --release --target aarch64-unknown-linux-gnu
  scp target/aarch64-unknown-linux-gnu/release/pitop pi@raspberrypi:~/
  ssh pi@raspberrypi ./pitop

  Look at it running. Describe what you see back to Claude Code:
  "The CPU gauges are too wide, they overflow on an 80-col terminal.
   The temperature shows 45234 instead of 45.2 — the millidegree
   conversion is missing. The sparkline is empty, I think the ring
   buffer isn't being fed."

Claude Code fixes the issues. Rebuild, re-deploy, repeat.
```

**Tip**: If you have a Pi with a display, take a screenshot with
`scrot` and drag it into Claude.ai web — Claude can see the
terminal screenshot and identify layout issues visually.

---

## Day 5-6: Pi-specific features + OpenClaw

### OpenClaw as your automated QA

Set up OpenClaw on a spare machine (or even on the Pi 4B itself):

```
# Install OpenClaw
# Connect it to your preferred messaging app (Telegram, Discord, etc.)

# Create a custom skill: pitop-ci
# This skill watches your git repo and on new commits:
#   1. Pulls the latest code
#   2. Cross-compiles for aarch64
#   3. Copies binary to the test Pi 5 via SSH
#   4. Runs it for 10 seconds, captures terminal output
#   5. Sends you the output in your messaging app
#   6. Reports "BUILD OK" or "FAILED: <error>"
```

Now every time you push a commit, you get a message on your phone
showing whether pitop built and ran successfully on real hardware.

You can also set up a Heartbeat schedule:

```
# Every night at 2am:
#   1. Pull latest code
#   2. Build for all 3 targets
#   3. Run on Pi 5 for 60 seconds, measure RSS memory usage
#   4. Compare to yesterday's measurement
#   5. If memory increased >20%, alert me
```

### Building Pi 5 PMIC collector

This is where you combine multiple tools:

```
Step 1 (Gemini): Research
  gemini "Show me the exact output format of vcgencmd pmic_read_adc
  on a Pi 5. What are all 12 rail names? What are the voltage and
  current field formats? Can this command fail, and what does failure
  look like?"

Step 2 (Claude Code): Implement
  "Implement the PMIC power collector based on this vcgencmd output
  format: [paste Gemini's answer]. Parse all 12 rails into a struct.
  Calculate total estimated wattage using the formula from our
  research doc. Handle the case where vcgencmd is not available."

Step 3 (Real hardware): Validate
  SSH to your Pi 5, run the binary, confirm the numbers match what
  vcgencmd shows directly.
```

### Building PCIe detection

```
You type into Claude Code:
  "Implement PCIe link detection for the Pi 5 power tab.
   Read /sys/bus/pci/devices/*/current_link_speed and
   current_link_width. Map GT/s values to PCIe generation names:
   2.5 GT/s = Gen 1, 5.0 GT/s = Gen 2, 8.0 GT/s = Gen 3.
   Show the device name from the 'device' and 'vendor' files.
   Handle the case where no PCIe devices exist (Zero 2W, Pi 4B)."
```

---

## Day 7: Polish and release

```
You type into Claude Code:
  "Review the entire codebase. Check for:
   - Any unwrap() calls that should be error handling
   - Hardcoded hwmon paths that should be discovered
   - Missing graceful degradation when features aren't available
   - Clippy warnings
   - Missing or incomplete tests
   List everything that needs fixing."

Claude Code audits and lists issues. You fix them one by one.

Then, Codex for release packaging:
  codex "Create a release script that builds pitop for all three
  targets, strips the binaries, creates tarballs, and generates
  a changelog from git commits since the last tag."

And a README:
  codex "Write a polished README.md for pitop. Include: what it is,
  screenshots section (placeholder), supported boards, installation
  from prebuilt binary, building from source, keyboard shortcuts,
  and a feature comparison table vs htop/pitop-go/pi_dashboard."
```

---

## Quick reference: which tool for what

| I need to... | Tool | Why |
|-------------|------|-----|
| Plan the project | Claude Code + BMAD agents | Structured planning with context that persists |
| Implement a feature | Claude Code | Best Rust support, multi-file awareness |
| Write CI/CD configs | Codex CLI | Self-contained, doesn't need project context |
| Write docs/README | Codex CLI | Good at standalone text, runs fast |
| Research kernel/sysfs details | Gemini CLI | Massive context, web-grounded |
| Validate cross-kernel compat | Gemini CLI | Can digest long forum threads |
| Auto-build on commit | OpenClaw | Persistent background agent |
| Auto-deploy to test Pi | OpenClaw | Cron/heartbeat scheduling |
| Regression monitoring | OpenClaw | Always-on, alerts via messaging |
| Implement 3+ independent modules | Claude Code subagents | Parallel execution, same session |
| Big refactor across many files | Claude Code agent teams | Teammates with shared context |
| Quick utility script | Codex CLI | Fast, sandboxed, no setup |

---

## What you're actually doing all day

Honestly, your job as the human is:

1. **Deciding what to build next** — pick the next story
2. **Providing context** — paste the story + relevant research
3. **Reviewing code** — read every diff before committing
4. **Testing on real hardware** — SSH to your Pis, run the binary, describe what you see
5. **Course-correcting** — "that's wrong because..." or "simplify this"
6. **Committing** — you are the gatekeeper for what goes into git

You are NOT writing Rust from scratch. You are NOT debugging
by staring at code for hours. You're directing, reviewing, and
validating. Think of yourself as a tech lead with a very fast,
very literal junior team that needs clear specs and honest feedback.

The BMAD planning phase is where you do your deepest thinking.
The implementation phase is where you do your most careful reviewing.
Both matter. Neither can be skipped.
