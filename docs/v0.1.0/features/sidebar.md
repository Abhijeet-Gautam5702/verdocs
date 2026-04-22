# Advanced Sidebar Management

Verdocs uses an opinionated folder-based sidebar management system. It's designed to automatically organize your documentation based on your directory structure while providing powerful defaults for navigation.

## Sidebar Organization

The sidebar is automatically generated from the directories and Markdown files within each version folder. Verdocs follows specific rules to ensure the most important pages are prioritized.

### Priority Sorting
Verdocs uses a specific sorting logic for the sidebar items:
1.  **Home Page First:** Any folder or Markdown file named **"home"** (case-insensitive) is automatically moved to the top of the sidebar. This ensures your introductory content is always accessible first.
2.  **Alphabetical Order:** All other folders and files are sorted alphabetically.

### Hierarchy and Nesting
The sidebar reflects your folder structure exactly:
-   **Folders:** Folders become expandable groups in the sidebar.
-   **Files:** Markdown files become clickable links.
-   **Nesting:** Verdocs supports multi-level nesting, creating a clear hierarchical navigation.

## Folder Index Rules

For every directory you create in a version (e.g., `v1.0.0/api/`), Verdocs requires a corresponding Markdown file with the same name (e.g., `v1.0.0/api/api.md`). This file acts as the primary landing page for that sidebar group. Refer to [Project Requirements](@/project-requirements/project-requirements.md) for more information.

## Automatic Title Conversion

Verdocs automatically converts your folder and file names into human-readable titles in the sidebar.
-   **Slug to Title:** A file named `getting-started.md` will appear as "Getting Started" in the sidebar.
-   **Capitalization:** Names are automatically title-cased.

## Usage Tips

-   **Naming for Order:** Since sorting is alphabetical, you can prefix your folders with numbers (e.g., `01-basics`, `02-advanced`) to control their order in the sidebar.
-   **Clean Folder Names:** Use simple, hyphenated names for your folders and files to ensure clean URLs and professional-looking sidebar titles.
-   **Nested Groups:** Group related documentation into folders to keep the sidebar organized and easy to navigate.
