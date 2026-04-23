# Project Requirements

To ensure that Verdocs can successfully generate your documentation site, your project must adhere to a specific directory structure and naming convention. Verdocs is opinionated about its architecture to provide optimal performance and versioning support.

## 1. Directory Structure

A Verdocs project is organized into three primary sections: global configuration, global assets, and versioned documentation folders.

```text
my-docs/
├── assets/                 # MANDATORY: Global assets (images, logos, etc.)
├── config.yml              # MANDATORY: Global site configuration
└── v1.0.0/                 # MANDATORY: Versioned folder (must start with 'v')
    ├── home/               # Subdirectory
    │   └── home.md         # MANDATORY: Folder index file
    └── getting-started/
        └── getting-started.md
```

## 2. Mandatory Folder Index Files

Verdocs strictly requires that every subdirectory within a versioned folder contains an "index" Markdown file. This file **must** have the exact same name as the folder it resides in.

*   **Correct:** `v1.0.0/api/api.md`
*   **Incorrect:** `v1.0.0/api/index.md`

If a subdirectory is found without a matching `.md` file, the generator will return an error and stop the build process. This ensures that every navigation point in your sidebar has a corresponding landing page.

## 3. Version Naming Rules

Every folder at the root of your project that is intended to be a documentation version must follow these rules:
-   **Prefix:** The folder name must start with a lowercase 'v'.
-   **Content:** The folder must contain at least one Markdown file or subdirectory.

Folders that do not start with 'v' (except for `assets`, `search-index`, and `out`) will be ignored by the generator.

## 4. Assets Directory

The `assets/` directory at the root is reserved for all static files. These files are automatically copied to the production build and are globally accessible across all versions of your documentation. Refer to the [Path Resolution Guide](@/core-concepts/path-resolution/path-resolution.md) for details on how to link these assets.
