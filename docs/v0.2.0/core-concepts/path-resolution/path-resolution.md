# Path Resolution

Understanding how paths are resolved in Verdocs is crucial for creating well-connected and visually rich documentation. Verdocs supports two primary strategies for linking assets and other sections of the website.

## 1. Linking Images

All static images and assets must be stored in the `assets/` directory at the root of your project (the same level as the versioned folders). When referencing these images in Markdown, you have two options:

### Standard Relative Path
Standard relative paths work as they would in any Markdown editor:

```markdown
![Logo](../../assets/verdocs-logo.png)
```

### Verdocs Absolute Path Strategy
For images, any path that starts with `assets/` is automatically resolved relative to the site's root directory:

```markdown
![Logo](assets/verdocs-logo.png)
```

{IMPORTANT type="admonition" title="Note"}
Kindly name the images in lowercase and hyphen separated. Unknown special characters or whitespaces **might** break the image rendering.
{/IMPORTANT}

## 2. Linking Other Pages and Sections

Linking between documentation pages is handled intelligently by Verdocs' internal link resolver.

### Relative Linking
Standard relative links between Markdown files are supported:

```markdown
[Go to Features](../features/features.md)
```

### Verdocs Global Linking Strategy
To simplify linking within the same version of your documentation, use the `@/` global linking prefix. This prefix informs Verdocs that the path should be resolved relative to the **current version's root directory**.

This allows you to link to any page within the same version without worrying about your current file's depth in the directory tree:

```markdown
# To link to "v1.0.0/getting-started/installation/installation.md" from any file in v1.0.0:
[Installation Guide](@/getting-started/installation/installation.md)
```

### Path Normalization and Redirection
When you link to a `.md` file, Verdocs automatically:
-   Removes the `.md` extension.
-   Resolves the target route based on your folder structure.
-   Handles directory index files (e.g., `v1.0.0/home/home.md` is resolved to `/v1.0.0/home`).

By using the `@/` strategy, your internal links remain robust even if you move files or restructure your directories in the future.

{TIP type="admonition" title="Recommendation"}
It is strongly recommended that you use Verdocs' global linking format (`@/`) to reference images and other markdown pages on the website.
{/TIP}
