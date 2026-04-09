# BeCode Desktop 🐝

Beautiful AI-powered coding assistant with a bee theme.

## Features

- 🎨 **Beautiful UI** - Modern glassmorphism design with bee-themed colors
- 🤖 **Multiple AI Providers** - Anthropic, OpenAI, Gemini, Mistral, OpenRouter
- 🔧 **Tool Execution** - Read/write files, run commands, search codebase
- 💬 **Streaming Chat** - Real-time responses with markdown rendering
- 📁 **File Tree** - Browse your project structure
- 💾 **Session Management** - Save and load chat sessions
- ⌨️ **Keyboard Shortcuts** - Ctrl+K command palette, and more
- 🤖 **Sub-Agents** - Specialized agents for exploration, planning, review

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.70+
- [Tauri CLI](https://tauri.app/v2/guides/getting-started/prerequisites)

### Setup

```bash
# Install frontend dependencies
cd frontend
npm install

# Run in development mode
npm run tauri dev
```

### Build

```bash
# Build for production
cd frontend
npm run tauri build
```

The installer will be in `src-tauri/target/release/bundle/`.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Open Command Palette |
| `Ctrl+Enter` | Send message |
| `Ctrl+L` | Clear chat |
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+,` | Open settings |
| `Ctrl+Shift+C` | Compact context |
| `Escape` | Close modal/Cancel |
| `↑` | Previous message in history |

## Slash Commands

| Command | Description |
|---------|-------------|
| `/help` | Show available commands |
| `/clear` | Clear chat history |
| `/compact` | Compact context (summarize old messages) |
| `/save [name]` | Save current session |
| `/load` | Load a saved session |
| `/model [name]` | Change model |
| `/provider [name]` | Change provider |
| `/agents on\|off` | Toggle sub-agents |

## Project Structure

```
becode/
├── frontend/           # React + TypeScript frontend
│   ├── src/
│   │   ├── components/ # UI components
│   │   ├── store/      # Zustand state management
│   │   ├── hooks/      # React hooks
│   │   └── types/      # TypeScript types
│   └── package.json
├── src-tauri/          # Rust backend (Tauri)
│   ├── src/
│   │   ├── commands/   # IPC command handlers
│   │   └── main.rs     # App entry point
│   └── Cargo.toml
└── src/                # Core BeCode library (shared)
    ├── providers/      # LLM providers
    ├── tools/          # Tool implementations
    └── agent/          # Agent runtime
```

## License

MIT
