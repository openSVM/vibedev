<!--
Sync Impact Report:
- Version change: [No previous version] → 1.0.0
- Modified principles: None (initial constitution)
- Added sections: All sections (initial creation)
- Removed sections: None
- Templates requiring updates:
  ✅ plan-template.md - Constitution Check section validated
  ✅ spec-template.md - Requirements alignment validated
  ✅ tasks-template.md - Task categorization validated
- Follow-up TODOs: None - all placeholders filled
- Rationale: MAJOR version 1.0.0 for initial constitution establishment
-->

# vibedev Constitution

## Core Principles

### I. CLI-First Architecture

Every feature MUST be accessible via command-line interface with text-based I/O. The tool
prioritizes developer experience through predictable, composable commands that follow Unix
philosophy: do one thing well, accept standard input/arguments, emit to stdout (data) and
stderr (errors).

**Rationale**: CLI-first ensures scriptability, automation, and integration with existing
developer workflows. Text I/O enables piping, composition, and debuggability without GUI
dependencies.

### II. Multi-Tool Support

The analyzer MUST maintain parsers for all major AI coding assistants (Claude Code, Cursor,
Cline, Kiro, Copilot, Windsurf, Continue.dev, Aider, Cody, and 10+ others). Adding support
for a new tool MUST NOT break existing parsers. Each parser implements the `LogParser` trait
with `can_parse()` and `parse()` methods.

**Rationale**: Users increasingly work across multiple AI tools. A single analyzer prevents
fragmentation and provides holistic insights. Trait-based design ensures extensibility without
coupling.

### III. Privacy-First Data Handling

All dataset exports and backups MUST sanitize sensitive data using the 27-pattern `Sanitizer`
system. The tool MUST redact API keys, passwords, emails, phone numbers, SSNs, credit cards,
IP addresses, and personal file paths. Sanitization MUST be non-optional for export commands.

**Rationale**: AI logs contain intimate developer workflows and often leak credentials. Making
sanitization mandatory prevents accidental exposure while enabling safe dataset sharing and
research.

### IV. Offline-Capable Analysis

The tool MUST support embedded LLM analysis via llama.cpp integration with no internet or API
key requirements. Users MUST be able to analyze their data completely offline. Online APIs
(when used) are opt-in only.

**Rationale**: Privacy-conscious developers need offline analysis. AI logs are sensitive;
sending them to third-party APIs without explicit consent violates trust.

### V. Incremental Complexity

Basic commands (`discover`, `analyze`) MUST work with zero configuration. Advanced features
(`insights --html`, `ultra-deep`, TUI) are opt-in. The tool MUST NOT require learning curve
for core functionality.

**Rationale**: Developers need quick wins. A `discover` → `analyze` workflow should take
<2 minutes. Advanced features reward deeper engagement but never block basic usage.

### VI. Test Coverage for Critical Paths

Sanitization logic, parsers for top 3 tools (Claude/Cursor/Cline), and discovery scanner
MUST have automated tests. Tests MUST fail before implementation (TDD for new parsers).
Edge cases in sanitization (regex bypasses) require dedicated test cases.

**Rationale**: Sanitization bugs leak credentials. Parser bugs misreport costs. These are
unacceptable failures. Test-first development catches regressions before user impact.

### VII. Performance at Scale

The tool MUST handle Flatpak installations (40+ GB of logs) without crashing. Discovery
MUST use parallel scanning (rayon). Parsers MUST implement line limits (10k for generic
parser). Memory usage MUST NOT exceed 2GB for typical workloads (5GB of logs).

**Rationale**: AI tools generate massive logs. A slow or crashing analyzer is unusable.
Users delete the tool rather than wait 10 minutes for analysis.

## Documentation Standards

### User-Facing Documentation

- `README.md` MUST include quickstart (<5 commands to value)
- `CLAUDE.md` (this file) MUST document architecture for AI assistants
- Each module MUST have docstring explaining purpose, inputs, outputs
- CLI help text (`--help`) MUST include examples

### Architecture Documentation

- Parser implementations MUST document log format and file paths
- Data structures MUST use inline comments for non-obvious fields
- Breaking changes MUST be documented in CHANGELOG.md

**Rationale**: The project targets both end-users (developers) and contributors (AI coding
assistants, open-source contributors). Both audiences need quick onboarding.

## Development Workflow

### Adding a New Parser

1. Create `src/parsers/toolname.rs` implementing `LogParser` trait
2. Add parser to `src/parsers/mod.rs` exports
3. Update `AiTool` enum in `src/models.rs` if new tool variant needed
4. Add search patterns to `LogDiscovery` in `src/discovery.rs`
5. Write tests in `tests/parser_tests.rs` (verify test fails first)
6. Update `CLAUDE.md` with new parser documentation
7. Update `README.md` supported tools list

### Feature Development

- New commands MUST be added to `src/main.rs` CLI parser
- Analysis features MUST choose correct analyzer level (basic/comprehensive/deep)
- Data exports MUST route through `Sanitizer` before file writes
- Breaking changes to output formats MUST provide migration path

### Quality Gates

- `cargo test` MUST pass
- `cargo clippy` MUST show no warnings
- Binary size MUST remain <10MB (current: 8MB)
- Core commands MUST complete <30 seconds for 1GB of logs

**Rationale**: Rust's compile-time guarantees + clippy catch most bugs. Size discipline
prevents bloat. Performance SLAs ensure usability.

## Governance

### Constitution Authority

This constitution supersedes all other documentation when conflicts arise. The README provides
user-facing guidance; CLAUDE.md provides implementation guidance; this constitution defines
non-negotiable principles. All pull requests MUST verify compliance with these principles.

### Amendment Process

1. Propose amendment via GitHub issue with rationale
2. Discuss impact on existing features and architecture
3. Require approval from 2+ maintainers
4. Increment version per semantic versioning (see below)
5. Update all dependent templates in `.specify/templates/`
6. Add migration guidance for breaking changes

### Versioning Policy

- **MAJOR**: Principle removed/redefined (backward-incompatible governance change)
- **MINOR**: New principle added or section materially expanded
- **PATCH**: Clarifications, wording improvements, typo fixes

### Compliance Review

Before merging pull requests, verify:

- Does this PR violate CLI-first architecture? (Principle I)
- Does this PR break existing parsers? (Principle II)
- Does this PR export data without sanitization? (Principle III)
- Does this PR require internet connectivity for core features? (Principle IV)
- Does this PR increase configuration complexity? (Principle V)
- Does this PR lack tests for critical paths? (Principle VI)
- Does this PR handle large log volumes poorly? (Principle VII)

If any answer is "yes," the PR MUST justify the violation or be revised.

Use `CLAUDE.md` for runtime development guidance and architectural patterns.

---

**Version**: 1.0.0 | **Ratified**: 2026-01-07 | **Last Amended**: 2026-01-07
