# Preview Server Internals (Hot Reloading)

The Verdocs preview server is designed for a seamless developer experience, allowing you to see your changes in real-time as you write.

## How it Works

When you run `verdocs preview`, Verdocs starts a local development server with a built-in file watcher and hot-reloading capability.

1.  **File Watcher:** Verdocs uses a high-performance native file watcher to monitor your project's root directory for any changes to Markdown files (`.md`) or the `config.yml` configuration file.
2.  **Instant Re-generation:** When a change is detected, Verdocs re-generates the site in memory and updates its internal status.
3.  **Browser Refresh:** If the script detects that the server has a newer version of the site, it automatically refreshes the browser page.

## Usage

Start the development server with:

```bash
verdocs preview
```

By default, the server is available at `http://localhost:8080`. You can customize the port using the `--port` or `-p` flag:

```bash
verdocs preview --port 3000
```

## Troubleshooting

-   **Manual Refresh:** If for any reason the page doesn't reload automatically, you can always perform a manual refresh (`Cmd + R` or `Ctrl + R`).
-   **Large Projects:** For very large documentation sites, you may notice a slight delay between saving a file and the browser refreshing while Verdocs re-generates the site.

{NOTE type="admonition" title="No WebSockets"}
Unlike many other hot-reloading systems, Verdocs uses a polling strategy instead of WebSockets. This makes it more robust in environments like remote development (over SSH) or within certain containerized setups where WebSockets may be blocked or unstable.
{/NOTE}
