# Customisation Guide

Verdocs is designed to be highly flexible through a single configuration file: `config.yml`. This file, located at the root of your project, controls everything from website metadata to granular theme colors.

## The `config.yml` Structure

The configuration is divided into four main sections: General Metadata, Hosting, Branding, and Theming.

### 1. General Metadata

These fields define the identity of your documentation site.

| Field | Type | Description |
| :--- | :--- | :--- |
| `title` | String | The name of your website. Appears in the browser tab and navbar. |
| `description` | String | A brief description used for SEO and meta tags. |

**Example:**
```yaml
title: "Project Phoenix Docs"
description: "The official technical documentation for Project Phoenix."
```

---

### 2. Subfolder Hosting

If you are not hosting your documentation at the root of a domain, you must configure the `base_path`.

| Field | Type | Description |
| :--- | :--- | :--- |
| `base_path` | String | The URL prefix for your site (e.g., `/verdocs/` for GitHub Pages). Defaults to `""`. |

**Example:**
```yaml
base_path: "/verdocs/"
```

{IMPORTANT type="admonition" title="GitHub Pages"}
`base_path` is specifically needed when you're deploying your site to GitHub Pages. Always set it to your repository name.
For more details on how to setup a github pages website, [click here](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site).
{/IMPORTANT}

---

### 3. Branding (Assets)

Customise the visual identity of your site by referencing files in your `assets/` directory.

| Field | Type | Description |
| :--- | :--- | :--- |
| `navbar_logo` | String | The filename of the logo shown in the top-left of the navbar. |
| `favicon` | String | The filename of the icon shown in the browser tab. |

**Example:**
```yaml
navbar_logo: "logo.png"
favicon: "favicon.ico"
```

---

### 4. Theming (Dual-Theme Support)

Verdocs features full support for Light and Dark modes. You can customise both independently via the `theme` and `dark_theme` sections.

#### Standard Theme Fields

| Field | Description |
| :--- | :--- |
| `primary_color` | The accent color used for links, active sidebar items, and progress bars. |
| `background_color` | The main background color of the documentation pages. |
| `text_color` | The primary font color for body text. |
| `colors` | A map of semantic tags (admonitions) to their respective hex colors. |

#### Semantic Colors (Admonitions)

The `colors` section (aliased as `admonitions`) allows you to define the brand colors for callouts like Tips, Notes, and Warnings.

**Example Configuration:**
```yaml
theme:
  primary_color: "#007bff"
  background_color: "#ffffff"
  text_color: "#333333"
  colors:
    tip: "#28a745"
    note: "#17a2b8"
    warn: "#ffc107"
    important: "#fd7e14"
    danger: "#dc3545"
    error: "#bd2130"
    custom_tag: "#722ed1" # You can add as many as you want!

dark_theme:
  primary_color: "#007bff"
  background_color: "#0d1117"
  text_color: "#c9d1d9"
  colors:
    tip: "#2ea043"
    # ... define other dark mode variants here
```

## How to Apply Changes

1.  **Modify `config.yml`**: Open the file in your preferred editor and update the values.
2.  **Preview**: If the preview server is running (`verdocs preview`), it will detect the changes and reload the site automatically.
3.  **Generate**: For production, run `verdocs generate` to bake these settings into your static HTML files.

{TIP type="admonition" title="Hex Only"}
Verdocs currently supports standard 6-digit hex codes (e.g., `#FFFFFF`) for all color fields.
{/TIP}
