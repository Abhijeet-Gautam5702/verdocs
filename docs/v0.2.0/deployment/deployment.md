# Deployment Strategies

Verdocs makes deployment seamless by offering host-specific optimizations. Once you've written your documentation, you can generate a production-ready `out/` directory tailored for your hosting provider.

## The `--host` Flag

The `verdocs generate` command includes a `--host` flag that automatically configures the output directory for your target platform:

```bash
verdocs generate --host <vps | vercel | gh-pages>
```

---

## 1. Personal VPS (Nginx/Apache)

Hosting on a VPS gives you full control over your documentation. By using `--host vps`, Verdocs ensures that visiting your root domain automatically redirects to the latest documentation version.

### Steps to Deploy

1.  **Generate the site:**
    ```bash
    verdocs generate --host vps
    ```
2.  **Upload the files:**
    Copy the contents of the `out/` directory to your server's web root (e.g., `/var/www/docs`):
    ```bash
    scp -r out/* user@your-vps-ip:/var/www/docs
    ```
3.  **Configure Nginx:**
    Ensure your site configuration handles folder-based routing:
    ```nginx
    server {
        listen 80;
        server_name docs.yourdomain.com;
        root /var/www/docs;
        index index.html;

        location / {
            # Standard static file serving with .html fallback
            try_files $uri $uri/ $uri.html =404;
        }
    }
    ```
3. **Restart Nginx:**
    ```bash
    sudo systemctl restart nginx
    ```

---

## 2. Caddy (Modern VPS)

Caddy is a powerful, enterprise-ready open source web server with automatic HTTPS. Serving a Verdocs site with Caddy requires minimal configuration.

### Steps to Deploy

1.  **Generate the site:**
    ```bash
    verdocs generate --host vps
    ```
2.  **Upload the files:**
    Transfer the `out/` directory content to your server (e.g., `/var/www/docs`).
3.  **Configure Caddy:**
    Create or update your `Caddyfile`:
    ```caddy
    docs.yourdomain.com {
        root * /var/www/docs
        file_server

        # Handle clean URLs (e.g., /v1.0.0/home -> /v1.0.0/home.html)
        try_files {path} {path}/ {path}.html
    }
    ```
4.  **Restart Caddy:**
    ```bash
    sudo systemctl reload caddy
    ```

---

## 3. Vercel

Vercel is optimized for speed and global delivery. Using `--host vercel` automatically generates the required configuration for clean routing.

### Steps to Deploy

1.  **Generate the site:**
    ```bash
    verdocs generate --host vercel
    ```
2.  **Deploy via CLI:**
    ```bash
    vercel out --prod
    ```
    *Verdocs automatically creates a `vercel.json` file inside `out/` with `cleanUrls: true`, so no manual configuration is needed.*

---

## 3. GitHub Pages

Perfect for hosting documentation directly from your repository. Using `--host gh-pages` handles the specific requirements of GitHub's environment.

### Steps to Deploy

1.  **Configure Base Path:**
    If your site is at `username.github.io/repo-name/`, set `base_path: /repo-name/` in your `config.yml`.
2.  **Generate the site:**
    ```bash
    verdocs generate --host gh-pages
    ```
3.  **Push to Branch:**
    Push the contents of the `out/` folder to your `gh-pages` branch.
    *Verdocs automatically creates a `.nojekyll` file to ensure all documentation folders are served correctly.*

---

## 4. Docker

If you prefer containerized deployment, you can wrap the output in a lightweight Nginx container.

1.  **Generate:** `verdocs generate --host vps`
2.  **Dockerfile:**
    ```dockerfile
    FROM nginx:alpine
    COPY ./out /usr/share/nginx/html
    EXPOSE 80
    ```
3.  **Build and Run:**
    ```bash
    docker build -t my-docs .
    docker run -p 80:80 my-docs
    ```
