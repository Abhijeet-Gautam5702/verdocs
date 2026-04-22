# verdocs

A high-performance, version-aware static documentation site generator built in Rust.

Verdocs is designed to help you create beautiful, searchable, and versioned documentation for your projects with minimal effort. It treats versioning as a first-class citizen, allowing you to manage multiple versions of your documentation side-by-side using a simple folder-based structure.

## Key Features

-   **Blazing Fast**: Leverages Rust's performance for near-instant builds and site navigation.
-   **Version-Native**: Manage multiple versions of your docs effortlessly using standard directory structures.
-   **Integrated Search**: Powerful, zero-config client-side search that "just works" across all versions.
-   **Live Reload**: See your changes in real-time as you edit with our built-in development server.
-   **Beautiful Admonitions**: Support for rich, color-coded callouts (Tip, Note, Warning, etc.).
-   **Simple Configuration**: Customize your site's theme, logo, and navigation via a single `config.yml`.

## Local Installation

To install Verdocs locally, you'll need the [Rust toolchain](https://rustup.rs/) (cargo and rustc) installed on your system.

### 1. Clone the repository
```bash
git clone https://github.com/Abhijeet-Gautam5702/verdocs.git
cd verdocs
```

### 2. Build and Install
Install the `verdocs` binary directly to your cargo bin directory:
```bash
cargo install --path .
```

### 3. Verify Installation
Ensure that your Cargo bin directory (usually `~/.cargo/bin`) is in your system's `PATH`, then run:
```bash
verdocs --help
```

## Quick Start

1.  **Initialize a new project**:
    ```bash
    mkdir my-docs
    cd my-docs
    verdocs init
    ```
    This creates a basic project structure with sample versioned content.

2.  **Start the preview server**:
    ```bash
    verdocs preview
    ```
    By default, your documentation will be available at `http://localhost:8080/v1.0.0/home`. The server includes hot-reloading, so any changes you make to your Markdown files will be reflected instantly.

3.  **Generate the static site**:
    ```bash
    verdocs generate
    ```
    This creates an `out/` directory containing your fully static documentation website, ready for deployment.

## Deployment

Once you have generated your documentation using `verdocs generate`, your website lives inside the `out/` directory. Since this is a completely static site, you can host it anywhere. Here are the four most common ways to deploy it:

### 1. Vercel (Recommended)
Vercel is the easiest way to host static sites with zero-configuration.

1.  **Install Vercel CLI**: `npm i -g vercel`
2.  **Configure Clean URLs**: Create a `vercel.json` file in your project root with the following content:
    ```json
    {
      "cleanUrls": true
    }
    ```
3.  **Deploy**: Run `vercel out` from your terminal and follow the prompts.

### 2. Personal VPS (Nginx)
If you are running your own server, Nginx is the best choice for serving static files efficiently.

1.  **Upload Files**: Copy the contents of your `out/` folder to your server (e.g., `/var/www/docs`).
2.  **Configure Nginx**: Update your site configuration to handle our folder-based routing:
    ```nginx
    server {
        listen 80;
        server_name yourdomain.com;
        root /var/www/docs;
        index index.html;

        location / {
            # This ensures /v1/home correctly serves /v1/home/index.html
            try_files $uri $uri/ $uri.html =404;
        }
    }
    ```
3.  **Restart Nginx**: `sudo systemctl restart nginx`

### 3. GitHub Pages
Perfect for hosting documentation directly alongside your source code for free.

1.  **Prepare Branch**: Create a new branch named `gh-pages`.
2.  **Upload**: Push the contents of your `out/` folder to this branch.
3.  **Enable**: Go to your Repository Settings > Pages and ensure the source is set to the `gh-pages` branch.
4.  **Note**: GitHub Pages handles folder-based routing (`/folder/` -> `/folder/index.html`) automatically.

### 4. Docker
If you use container orchestration (like Kubernetes or Coolify), you can wrap your site in a tiny container.

1.  **Create Dockerfile**: Create a `Dockerfile` in your project root:
    ```dockerfile
    FROM nginx:alpine
    # Copy the generated site to the Nginx html directory
    COPY ./out /usr/share/nginx/html
    EXPOSE 80
    ```
2.  **Build and Run**:
    ```bash
    docker build -t my-docs .
    docker run -p 80:80 my-docs
    ```
3.  The default Nginx Alpine configuration will automatically serve `index.html` files inside your versioned folders.
