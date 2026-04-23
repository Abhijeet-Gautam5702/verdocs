# Versions

Versioning is the fundamental core of Verdocs. We believe documentation should evolve alongside your software, which is why we've made managing multiple versions of your site as intuitive as possible.

## How Versioning Works

In Verdocs, versioning is driven by your project's directory structure. Any directory at the root of your project that follows the required naming convention is treated as a separate documentation version.

### Naming Conventions
To be recognized by the Verdocs generator, a versioned folder must:
-   **Start with a lowercase 'v'**: For example, `v1.0.0`, `v2.1-beta`, or `v-latest`.
-   **Follow Semantic Versioning (Recommended)**: While not strictly enforced, using semver (e.g., `v1.2.3`) is highly recommended for clarity.

### Project Structure
A typical Verdocs project with multiple versions looks like this:

```text
my-docs/
├── assets/         # Global assets shared by all versions
├── config.yml      # Global site configuration
├── v1.0.0/         # Documentation for version 1.0.0
│   └── home/
│       └── home.md
└── v1.1.0/         # Documentation for version 1.1.0
    └── home/
        └── home.md
```

## Creating a New Version

If you already have a `v1.0.0` documentation site and want to create a `v1.1.0` version, follow these steps:

1.  **Duplicate the Folder**: Copy the contents of your existing version folder (`v1.0.0`) into a new folder named `v1.1.0`.
2.  **Modify the Content**: Edit the Markdown files within the `v1.1.0` folder to reflect the changes in your software.
3.  **Automatic Detection**: Verdocs will automatically detect the new folder and add it to the version selector in the sidebar.

## Version Switching

Verdocs provides a built-in version selector for users. When a user switches versions, Verdocs attempts to maintain their current position in the documentation. For example, if a user is viewing `/v1.0.0/api/auth` and switches to `v1.1.0`, Verdocs will automatically redirect them to `/v1.1.0/api/auth`.

By treating versioning as a structural rule, Verdocs ensures that your documentation remains organized and navigable, regardless of how many versions you manage.
