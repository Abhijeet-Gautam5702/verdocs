# Automatic Table of Contents (Minimap)

Verdocs includes an automatic "Minimap" on the right side of every page, providing a dynamic table of contents that reflects the structure of your current documentation.

## How it Works

The Verdocs generator automatically scans each Markdown file for headings.
1.  **Extraction:** Verdocs extracts all headings from H2 to H6 levels.
2.  **Slugification:** Each heading is converted into a URL-friendly slug (e.g., "Installation Guide" becomes `installation-guide`).
3.  **Rendering:** These headings are then displayed as links in the Minimap on the right.
4.  **Indentation:** The Minimap visually indents each link according to its heading level, providing a clear hierarchy of the page's content.

## Usage

The Minimap is entirely automated. You don't need to manually create tables of contents in your Markdown files.

-   **Navigation:** Clicking any link in the Minimap scrolls the page to that specific section.
-   **Scroll Padding:** Verdocs automatically accounts for the fixed navbar height when scrolling to a heading, ensuring your content is never hidden behind the header.

## Optimization Tips

To make the most of the Minimap:
-   **Logical Structure:** Use H2 headings for primary sections and H3-H6 for subsections.
-   **Descriptive Headings:** Keep headings clear and concise, as they form the navigation interface for the page.
-   **Heading Levels:** Verdocs uses the **first H1 heading** as the main page title. The Minimap begins with H2 headings to avoid redundancy.

{TIP type="admonition" title="Scroll Performance"}
Verdocs' Minimap is designed to be lightweight and fast, providing a smooth navigation experience even for very long documentation pages.
{/TIP}
