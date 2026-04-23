# Code Blocks and Developer UX

Verdocs is built with developer documentation in mind. It includes several built-in features to make code blocks more useful and visually appealing.

## Syntax Highlighting

Verdocs includes **Highlight.js** (v11.9.0) with a customized version of the **GitHub Dark** theme for all code blocks.

### Supported Languages
By using standard Markdown fence syntax, you can specify the language of your code block for optimized highlighting:

```markdown
```rust
fn main() {
    println!("Hello, Verdocs!");
}
```


It will render as:
```rust
fn main() {
    println!("Hello, Verdocs!");
}
```

Verdocs supports dozens of popular programming languages including Rust, JavaScript, Python, Bash, YAML, and many others.

## Enhanced Developer Experience

In addition to syntax highlighting, Verdocs automatically injects several features into every code block to enhance usability:

1.  **Language Labels:** Every code block is automatically tagged with its language in the top-left corner (e.g., "RUST", "BASH", "YAML").
2.  **Copy-to-Clipboard Button:** Each code block features a built-in copy button that appears on hover, making it easy for users to grab code snippets.
3.  **Responsive Layout:** Code blocks are styled with a clean, dark interface and include horizontal scrolling for long lines of code.

## Optimization Tips

-   **Explicit Language Specification:** Always specify the language in your Markdown fence blocks (e.g., ` ```rust ` instead of just ` ``` `) to ensure the correct syntax highlighting and language label are applied.
-   **Clean Snippets:** Verdocs strips leading/trailing whitespace to ensure your code snippets are clean and ready to use.

{NOTE type="admonition" title="Accessibility"}
Verdocs ensures high contrast within code blocks to maintain accessibility for all users, regardless of their visual preferences.
{/NOTE}
