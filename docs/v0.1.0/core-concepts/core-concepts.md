# Core Concepts

Verdocs is an opinionated documentation generator designed for performance and ease of use. To achieve this, it employs a few unique "grammar" rules and a strict project structure that ensures consistency across different versions of your documentation.

## Grammar and Structure

Unlike generic static site generators, Verdocs is specifically tuned for documentation. Its primary differentiators include:

1.  **Strict Versioning:** Documentation isn't just a collection of files; it's a versioned history. The folder structure itself defines the available versions of your site.
2.  **Native Admonitions:** Verdocs introduces a custom syntax for callouts (tips, warnings, etc.) that are rendered beautifully without external plugins.
3.  **Intelligent Path Resolution:** It handles complex relative and global linking strategies, making it easy to move files without breaking links or image references.
4.  **Zero-Config Search:** A client-side search index is automatically generated for every version, ensuring high-speed search functionality.

## Documentation Sections

Explore the core features of Verdocs in detail:

| Section | Description |
| :--- | :--- |
| [Project Requirements](../project-requirements/project-requirements.md) | Mandatory directory structure and file naming rules. |
| [Admonitions](./admonitions/admonitions.md) | How to use and customize color-coded callouts. |
| [Path Resolution](./path-resolution/path-resolution.md) | Understanding relative and global (`@/`) linking. |
| [Versions](./versions/versions.md) | Naming conventions and managing multiple documentation versions. |
