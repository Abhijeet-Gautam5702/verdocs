# Verdocs 
![Version](https://img.shields.io/badge/Version-0.2.0-blue.svg)

![Banner](./resources/verdocs-logo-with-name.jpeg)

A high-performance, version-aware static documentation site generator built in Rust.

Verdocs is designed to help you create beautiful, searchable, and versioned documentation for your projects with minimal effort.

## Key Features
- **Version-Native**: Manage multiple documentation versions via folder structure.
- **Blazing Fast**: Near-instant builds leveraging Rust.
- **Zero-Config Search**: Minimal client-side search across all versions.
- **Live Reload**: See changes instantly as you edit in development.
- **Easy Deployment**: Generate structured files for VPS, Vercel, and GitHub Pages deployment in a single command.

## CLI Installation

1. Install Verdocs using the installer script:
```bash
curl -fsSL https://raw.githubusercontent.com/Abhijeet-Gautam5702/verdocs/main/scripts/install.sh | bash
```

2. Verify installation:
```bash
which verdocs # locate the binary
verdocs --version # check version
```

*For the complete installation and usage guide, visit the "Quickstart" section of the [official documentation](https://abhijeet-gautam5702.github.io/verdocs/)*

---

## Local Development

To install Verdocs locally, you'll need the [Rust toolchain](https://rustup.rs/) installed.

### 1. Install
Navigate to `verdocs` repository and run:
```bash
cargo install --path .
```
> This installs the verdocs binary in your machine, so you can run `verdocs` commands from anywhere.

### 2. Initialize
Create a new directory at any location of your choice and initialise `verdocs`
```bash
mkdir my-docs
cd my-docs
verdocs init
```

### 3. Preview
```bash
verdocs preview
```
Visit `http://localhost:8080` to see your documentation in real-time.

---

## Documentation

For detailed information on how to use Verdocs, including installation, core concepts, and deployment strategies, please visit the [documentation site](https://abhijeet-gautam5702.github.io/verdocs/)

---

## Changelog
Detailed changes for each release are documented in [CHANGELOG](CHANGELOG.md).
