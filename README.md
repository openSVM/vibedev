# vibedev

Analyze your AI coding assistant usage patterns, costs, and productivity.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.82+-orange.svg)](https://www.rust-lang.org/)

## Features

- **Multi-Tool Support** - Claude Code, Cursor, Cline, Kiro, Copilot, Windsurf, and 15+ more
- **Interactive TUI** - Real-time visualization of your AI usage (like `dust` for logs)
- **Embedded LLM** - Offline AI chat to analyze your data (no API keys needed)
- **Deep Insights** - Usage patterns, cost estimates, productivity metrics
- **Dataset Export** - Create sanitized training datasets from your conversations
- **Backup & Restore** - Compress and archive your AI learning data

## Quick Start

```bash
# Install
cargo install --path .

# Discover AI tool logs on your system
vibedev discover

# Interactive TUI dashboard
vibedev tui

# Generate analysis report
vibedev analyze --format html --output report.html
```

## Commands

| Command | Description |
|---------|-------------|
| `discover` | Scan system for AI tool logs |
| `analyze` | Generate analysis reports (text/json/html/markdown) |
| `tui` | Interactive terminal dashboard |
| `chat` | Chat with your data using embedded LLM |
| `daemon` | Keep LLM loaded for fast queries |
| `models` | Manage offline LLM models |
| `backup` | Create compressed backup archive |
| `prepare` | Export sanitized training dataset |
| `insights` | Comprehensive HTML dashboard |
| `compare` | Compare usage across tools |

## Embedded LLM (Offline)

Analyze your data with a fully offline LLM - no API keys or internet required:

```bash
# Download a model (one-time)
vibedev models download qwen-coder-1.5b

# Chat with your data
vibedev chat --query "What are my top productivity patterns?"

# Keep model loaded for fast queries
vibedev daemon start
vibedev chat --query "How can I reduce costs?"
vibedev daemon stop
```

**Available Models:**
- `qwen-coder-0.5b` - Fast, 1GB
- `qwen-coder-1.5b` - Balanced, 3GB (recommended)
- `qwen-coder-3b` - High quality, 6GB
- `deepseek-coder-1.3b` - Efficient, 2.6GB

**GPU Acceleration:**
```bash
cargo build --release --features cuda   # NVIDIA
cargo build --release --features metal  # Apple Silicon
```

## Interactive TUI

```bash
vibedev tui
```

Real-time visualization inspired by `dust`:
- Storage breakdown by tool
- Session counts and token estimates
- Cost analysis
- Keyboard navigation (↑↓ to navigate, Enter to expand, q to quit)

## Analysis Report

```bash
vibedev analyze --output report.md
```

**Sample output:**
```
# AI Coding Tools - Analysis Report

## Global Metrics
- Total Storage: 1.01 GB
- Total Sessions: 1,020
- Estimated Tokens: 21,245,300
- Peak Usage Hour: 14:00 UTC
- Most Used Tool: Claude Code

## Cost Estimate
- Monthly Cost: $165.71
- Optimization Potential: $49.71 (30%)

## Recommendations
1. Backup debug logs (345 MB compressible)
2. Clean file-history older than 30 days
3. Consider token caching for repeated queries
```

## Supported Tools

**AI Coding Assistants:**
- Claude Code, Cursor, Windsurf, Cline, Continue.dev
- GitHub Copilot, Sourcegraph Cody, Amazon Q
- Kiro, Roo Code, Kilo, Aider

**Code Completion:**
- Tabnine, Supermaven, CodeWhisperer, CodeGPT, Bito AI

**Logs detected from:**
- `~/.claude/` - Claude Code
- `~/.cursor/` - Cursor
- `~/.config/Code/User/globalStorage/` - VSCode extensions
- `~/.config/Kiro/` - Kiro
- And 80+ more locations

## Dataset Export

Create sanitized datasets for fine-tuning:

```bash
vibedev prepare --output ~/datasets
```

**Sanitization removes:**
- API keys (OpenAI, Anthropic, GitHub, AWS, etc.)
- Passwords and credentials
- Email addresses and phone numbers
- Personal file paths
- IP addresses

See [PREPARE_GUIDE.md](PREPARE_GUIDE.md) for details.

## Build from Source

```bash
git clone https://github.com/openSVM/vibedev.git
cd vibedev
cargo build --release
./target/release/vibedev --help
```

**Requirements:**
- Rust 1.82+
- ~8MB binary size

## Configuration

vibedev works out of the box with no configuration. Optional settings:

```bash
# Filter to specific tool
vibedev analyze --tool claude

# Time range
vibedev analyze --days 30

# Output format
vibedev analyze --format json --output data.json
```

## Contributing

Contributions welcome! Please read the code of conduct and submit PRs.

```bash
# Run tests
cargo test

# Run with debug logging
vibedev --verbose analyze
```

## License

MIT License - see [LICENSE](LICENSE) for details.

---

Built with Rust. Inspired by the AI coding revolution.
