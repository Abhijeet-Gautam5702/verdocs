# Admonitions

Admonitions are visually distinct, color-coded callouts used to highlight specific types of information. In Verdocs, they are a first-class feature designed to be professional, precise, and easy to use.

## How to Use Admonitions

Verdocs uses a specific syntax for admonitions that is compatible with most Markdown editors while allowing for rich, version-aware styling.

### Basic Syntax
The standard syntax for an admonition is:

```markdown
{TAG_NAME type="admonition" title="Admonition Title"}
Your content goes here. You can even include standard Markdown inside the block.
{/TAG_NAME}
```
For `TAG_NAME` = `TIP` the above block renders like this on the website:
{TIP type="admonition" title="Admonition Title"}
Your content goes here. You can even include standard Markdown inside the block.
{/TIP}

### Supported Default Tags
Verdocs comes with a set of pre-defined tags, each with its own semantic meaning and color:

*   `TIP`: For helpful suggestions, shortcuts, or "did you know" facts.
*   `NOTE`: For general information, background context, or side-bars.
*   `WARN`: For information that helps prevent common mistakes or pitfalls.
*   `IMPORTANT`: For critical information essential to the user's understanding.
*   `DANGER`: For dangerous actions that could lead to data loss or system failure.
*   `ERROR`: To indicate an invalid state, a failure, or an immediate need for correction.

## Customization

Admonitions are highly customizable through the `config.yml` file located at the root of your project. You can modify the primary colors for both light and dark themes.

### Defining Custom Colors
In your `config.yml`, the `theme.colors` and `dark_theme.colors` sections control the appearance of admonitions:

```yaml
theme:
  colors:
    tip: "#28a745"
    note: "#17a2b8"
    custom_tag: "#722ed1" # You can add completely new tags!
```

### Adding New Admonition Types
To create a new, custom admonition, simply add a key-value pair to the `colors` section of your `config.yml`. Once defined, you can use it in your Markdown:

```markdown
{CUSTOM_TAG type="admonition" title="Special Feature"}
This uses the custom color defined in config.yml.
{/CUSTOM_TAG}
```

## Inline Highlights
You can also use these tags for simple inline text highlighting without the full admonition block by omitting the `type="admonition"` attribute:

```markdown
{TIP}This is inline highlighted text.{/TIP}
```
The above renders to:
{TIP}This is inline highlighted text.{/TIP}

## Implementation Guidelines
To maintain a professional tone, use admonitions sparingly. Overusing them can distract the reader and dilute the impact of critical information. Always choose the tag that best fits the semantic weight of the content.
