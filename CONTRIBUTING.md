# Contributing to WaLinux

Thank you for helping improve WaLinux. Bug reports, documentation fixes, packaging improvements, and code contributions are all welcome.

## Before you begin

- Search existing issues before creating a new one.
- Use the issue templates and include your Linux distribution and desktop environment for platform-specific bugs.
- Keep each pull request focused on one change.
- Do not include WhatsApp account data, session files, phone numbers, or private messages in reports or screenshots.

## Local development

Install the prerequisites listed in the [README](README.md#requirements), then run:

```bash
npm ci
npm run tauri dev
```

Before submitting a pull request, run:

```bash
npm run build
npm run format:rust
npm run check:rust
cargo test --manifest-path src-tauri/Cargo.toml
```

## Pull requests

1. Create a branch from `main` with a short descriptive name.
2. Explain what changed and why.
3. Link any related issue.
4. Include screenshots for visible UI changes.
5. Confirm the checks above pass, or explain why one could not be run.

By contributing, you agree that your contribution is licensed under the repository's MIT License.
