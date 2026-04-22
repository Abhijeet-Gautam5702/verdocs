# CLI Reference

The Verdocs Command Line Interface (CLI) is the primary way to interact with your documentation projects. This page provides a detailed reference for all supported commands and their options.

## General Usage

```bash
verdocs [COMMAND] [OPTIONS]
```

To see the help for any command, you can use the `--help` flag:

```bash
verdocs --help
verdocs [COMMAND] --help
```

---

## Commands

### `init`

Initializes a new documentation project by creating a skeleton project structure.

**Usage:**
```bash
verdocs init [PATH]
```

**Arguments:**
- `PATH`: The directory where you want to initialize the project (defaults to the current directory `.`).

**What it does:**
- Creates an `assets/` directory.
- Creates a `search-index/` directory.
- Writes a default `config.yml` file.
- Adds a sample `verdocs-logo.png` to the `assets/` folder.
- Populates the project with two sample versioned folders: `v1.0.0` and `v1.1.0`.

---

### `generate`

Builds your documentation project into a production-ready static website.

**Usage:**
```bash
verdocs generate [PATH] [OPTIONS]
```

**Arguments:**
- `PATH`: The source directory of your documentation project (defaults to `.`).

**Options:**
- `-h`, `--host <HOST>`: Specifies the target hosting platform for optimization. Supported values: `vps` (default), `vercel`, `gh-pages`.

**What it does:**
- Validates the project structure and `config.yml`.
- Parses all Markdown files within versioned folders.
- Generates a client-side search index for each version.
- Produces a fully static `out/` directory containing optimized HTML, CSS, and JS.
- **Host Optimizations:** 
    - `vps`: Creates a root `index.html` redirector.
    - `vercel`: Creates `index.html` and `vercel.json` for clean URLs.
    - `gh-pages`: Creates `index.html` and `.nojekyll`.

---

### `preview`

Starts a local development server with hot-reloading for real-time documentation previewing.

**Usage:**
```bash
verdocs preview [PATH] [OPTIONS]
```

**Arguments:**
- `PATH`: The source directory of your documentation project (defaults to `.`).

**Options:**
- `-p`, `--port <PORT>`: Specifies the port to run the preview server on (defaults to `8080`).

**What it does:**
- Performs an initial site generation.
- Starts a local server (e.g., `http://localhost:8080`).
- Watches for changes to any Markdown files or the `config.yml`.
- Automatically re-generates the site and triggers a browser refresh when changes are saved.

---

### `clean`

Removes generated build files and configuration to reset the project state.

**Usage:**
```bash
verdocs clean [PATH] [OPTIONS]
```

**Arguments:**
- `PATH`: The project directory to clean (defaults to `.`).

**Options:**
- `-f`, `--full`: A destructive flag that removes all versioned directories (`v*`) in addition to the `out/` folder and `config.yml`.

**What it does:**
- Deletes the `out/` directory.
- Deletes the `config.yml` file.
- If `--full` is provided, it also removes every versioned documentation folder found in the root.

---

### `self-update`

Updates Verdocs to the latest version.

**Usage:**
```bash
verdocs self-update
```

**What it does:**
- Downloads and executes the latest installation script from the official repository.
- Replaces the current `verdocs` binary with the latest version.
- Requires `sudo` if the binary is installed in a protected directory like `/usr/local/bin`.

---

### `uninstall`

Removes the Verdocs binary and its configuration directory from your system.

**Usage:**
```bash
verdocs uninstall
```

**What it does:**
- Deletes the `~/.verdocs` configuration and analytics directory.
- Deletes the `verdocs` binary from its current installation path.
- Requires `sudo` if the binary is installed in a protected directory.

---

## Global Options

- `-V`, `--version`: Displays the current version of the Verdocs CLI.
- `-h`, `--help`: Displays help information for the CLI or a specific subcommand.
