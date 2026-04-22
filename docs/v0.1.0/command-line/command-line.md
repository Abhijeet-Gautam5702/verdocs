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
verdocs generate [PATH]
```

**Arguments:**
- `PATH`: The source directory of your documentation project (defaults to `.`).

**What it does:**
- Validates the project structure and `config.yml`.
- Parses all Markdown files within versioned folders.
- Generates a client-side search index for each version.
- Produces a fully static `out/` directory containing the HTML, CSS, and JS files.

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

## Global Options

- `-V`, `--version`: Displays the current version of the Verdocs CLI.
- `-h`, `--help`: Displays help information for the CLI or a specific subcommand.
