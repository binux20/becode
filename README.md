# 🐝 BeCode

> ⚠️ **PROPRIETARY SOFTWARE**  
> This source code is publicly visible for transparency and educational purposes only.  
> © 2026 All rights reserved. No license is granted for use, modification, or distribution.  
> Forking this repository does not grant any rights to use the code.

---

**Autonomous AI coding agent with beautiful TUI**

BeCode is a standalone Windows .exe code agent that helps you write, edit, and debug code using AI models from multiple providers.

## Features

- 🎨 **Beautiful TUI** - Terminal interface inspired by OpenCode
- 🔧 **9 Core Tools** - bash, read/write/edit files, glob/grep search, web fetch/search, task tracking
- 🤖 **7 Providers** - Anthropic, OpenAI, Gemini, Mistral, OpenRouter, + OpenAI-compatible
- 🔌 **No-Tools Mode** - Works with models that don't support function calling (via JSON blocks)
- 📎 **Attachments** - Send images and files with your prompts
- 🔒 **Permission System** - Read-only, workspace-write, or full access modes
- 💾 **Sessions** - Persistent conversation history with JSONL storage
- 🐝 **Mascot** - Friendly bee with encouraging phrases!

## Installation

### From Release

Download `becode.exe` from the [releases page](https://github.com/yourname/becode/releases).

### From Source

```bash
git clone https://github.com/yourname/becode.git
cd becode
cargo build --release
```

The binary will be at `target/release/becode.exe`.

## Quick Start

```bash
# Set your API key
becode auth set-key anthropic

# Launch TUI
becode

# Or run a one-shot task
becode run "Fix the bug in main.rs"

# With a specific provider
becode --provider openai run "Add unit tests"
```

## Configuration

Config file: `~/.becode/config.toml`

```toml
default_provider = "anthropic"
default_model = "claude-sonnet-4-20250514"

[agent]
max_steps = 25
enable_web_search = true

[ui]
theme = "dark"  # dark, light, hacker, bee-yellow
mascot_enabled = true

# Custom OpenAI-compatible provider
[providers.local-llama]
type = "openai-compatible-no-tools"
base_url = "http://localhost:11434/v1"
model = "llama3.1:70b"
```

## Providers

| Provider | Type | Tools | Vision |
|----------|------|-------|--------|
| Anthropic | Native | ✅ | ✅ |
| OpenAI | Native | ✅ | ✅ |
| Gemini | Native | ✅ | ✅ |
| Mistral | Native | ✅ | ❌ |
| OpenRouter | Native | ✅ | ✅ |
| OpenAI-Compatible | Native | ✅ | ? |
| OpenAI-Compatible-NoTools | JSON Blocks | ✅* | ? |

*Tools via JSON blocks in prompts - works with any model!

## Tools

| Tool | Permission | Description |
|------|------------|-------------|
| `bash` | WorkspaceWrite | Execute shell commands |
| `read_file` | ReadOnly | Read file contents |
| `write_file` | WorkspaceWrite | Write file contents |
| `edit_file` | WorkspaceWrite | Edit with string replacement |
| `glob_search` | ReadOnly | Find files by pattern |
| `grep_search` | ReadOnly | Search file contents |
| `web_fetch` | ReadOnly | Fetch URL contents |
| `web_search` | ReadOnly | Search the web |
| `task_track` | WorkspaceWrite | Track tasks in session |

## Keyboard Shortcuts (TUI)

| Key | Action |
|-----|--------|
| `Enter` | Submit message |
| `F1` | Help |
| `F2` | Change provider |
| `F3` | Change model |
| `F4` | Change project |
| `Ctrl+L` | Clear chat |
| `Ctrl+O` | Attach file |
| `Esc` | Quit |

## Easter Eggs 🥚

```bash
becode bee     # ASCII art bee
becode party   # Confetti!
```

## License

**Proprietary** — All rights reserved.  
This code is publicly visible for transparency only. No permission is granted to use, copy, modify, or distribute.

---

🐝 *"Turning bugs into features since 2026"*
