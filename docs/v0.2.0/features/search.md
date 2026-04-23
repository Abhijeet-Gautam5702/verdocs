# Search Engine Mechanics

Verdocs includes a high-performance, zero-config search engine that operates entirely on the client side. This ensures that search functionality is instantaneous and works even in offline or static hosting environments.

## How it Works

When you run `verdocs generate` or while the preview server is running, Verdocs:
1.  **Scans Content:** Iterates through all Markdown files within each version folder.
2.  **Extracts Metadata:** Identifies page titles (from H1 tags or file names) and indexes all text content.
3.  **Generates JSON Index:** A highly optimized `.json` search index is created for each version and stored in `out/search-index/`.
4.  **Client-Side Resolution:** The browser downloads only the index for the version currently being viewed, making search results available without further network requests.

## Usage

You can trigger the search interface in two ways:
1.  **Clicking:** Click the "Search..." bar in the top navigation area.
2.  **Keyboard Shortcut:** Press `Cmd + K` (macOS) or `Ctrl + K` (Linux/Windows).

### Search Results

Results are ranked based on matches in titles and body content. Each result provides a snippet of the matching line with the query terms highlighted, allowing for quick context assessment.

## Optimization Tips

To make your documentation more searchable:
-   **Use Clear H1 Headings:** Verdocs uses the first H1 heading in each file as the primary title in search results.
-   **Structured Hierarchy:** Clear heading structures (H2, H3, etc.) help index the document effectively.
-   **Descriptive Content:** Since Verdocs indexes the full text, ensure your documentation contains relevant keywords for better search discovery.

{NOTE type="admonition" title="Version Isolation"}
Searching is scoped to the **current version** you are viewing. When a user switches to a different version, Verdocs will load the search index for that specific version.
{/NOTE}
