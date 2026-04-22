# Quick Start Guide

This guide will walk you through installing Verdocs and deploying your first versioned documentation site.

## 1. Installation

Install Verdocs using the official installer script:

```bash
curl -fsSL https://raw.githubusercontent.com/verdocs/verdocs/main/install.sh | bash
```

## 2. Verify Installation

After installation, verify that the `verdocs` binary is available in your PATH:

```bash
# Check version
verdocs --version

# View help
verdocs --help

# Locate the binary
which verdocs
```

## 3. Initialize Your Project

Create a new directory for your documentation and initialize it:

```bash
mkdir my-docs
cd my-docs
verdocs init
```

The `init` command generates a project skeleton with sample versioned content and a default configuration file.

{IMPORTANT type="admonition" title="Project Requirements"}
Before modifying your project, please review the [Project Requirements](@/project-requirements/project-requirements.md) to understand the mandatory directory structure and file naming rules.
{/IMPORTANT}

## 4. Managing Versions

By default, Verdocs initializes with sample versioned folders. You should rename or create folders following the Semantic Versioning format (e.g., `v1.0.0`, `v2.0.0-beta`). 

{IMPORTANT type="admonition"}
**Note:** Every versioned folder must start with a lowercase 'v' to be recognized by the generator.
{/IMPORTANT}

## 5. Assets and Images

Place all static assets, such as images, diagrams, and PDFs, in the `assets/` directory at the root of your project. 

### Referencing Images
You can reference images in your Markdown files using two strategies:
- **Relative Paths:** `../../assets/logo.png`
- **Verdocs Absolute Path:** `assets/logo.png` (Verdocs will automatically resolve this relative to the site root).

## 6. Configuration

Edit the `config.yml` file in your project root to customize your website. This file controls:
- **Metadata:** Website title, description, and favicon.
- **Branding:** Navbar logo and primary theme colors.
- **Theming:** Custom colors for different admonition types (Tip, Note, Warning, etc.).

## 7. Preview and Live Development

Start the development server to preview your changes in real-time:

```bash
verdocs preview
```

The server will host your site locally (default: `http://localhost:8080/<version>`). Any changes saved to your Markdown files or configuration will trigger an instant reload in the browser.

## 8. Links and Redirections

Verdocs supports flexible linking between pages:
- **Relative Linking:** `[Getting Started](./getting-started/getting-started.md)`
- **Verdocs Absolute Linking:** Use the `@/` prefix to reference paths relative to the current version root. For example, to link to `getting-started.md` from anywhere within the same version: `[Getting Started](@/getting-started/getting-started.md)`.

## 9. Production Build

Once your documentation is ready, generate a production-ready static build:

```bash
verdocs generate
```

This command creates an `out/` directory containing the optimized HTML, CSS, JavaScript, and search indices for your website.

## 10. Deployment

The contents of the `out/` directory are completely static. You can deploy them to any hosting provider, such as:
- **Vercel / Netlify:** Push the `out/` folder or configure your build command.
- **GitHub Pages:** Deploy the `out/` folder to a `gh-pages` branch.
- **Nginx/Apache:** Copy the `out/` content to your web root.
