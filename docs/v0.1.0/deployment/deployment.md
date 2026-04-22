# Deployment Strategies

Once you have generated your documentation site using the `verdocs generate` command, you will find a production-ready version of your website inside the `out/` directory. Because Verdocs produces a completely static site, it can be hosted on virtually any platform that serves static files.

This guide provides detailed deployment instructions for the most common hosting environments.

---

## 1. Vercel (Recommended)

Vercel is a top-tier platform for hosting static sites with zero-configuration and global CDN support.

### Configuration
To support Verdocs' clean, folder-based routing, create a `vercel.json` file in your project's root:

```json
{
  "cleanUrls": true
}
```

### Deployment via CLI
1.  **Install the Vercel CLI:** `npm i -g vercel`
2.  **Deploy the Output:** From your project root, run `vercel out`.
3.  **Follow the Prompts:** Link your project to Vercel and confirm the deployment settings.

### Continuous Deployment (Git)
You can also connect your Git repository to Vercel. In the Vercel dashboard, set your **Output Directory** to `out`. If you have a custom build step to run `verdocs generate`, you may need to include the Verdocs binary in your build environment.

---

## 2. Nginx (Personal VPS)

If you are managing your own server, Nginx is a high-performance choice for serving static content efficiently.

### Preparation
1.  Generate your site: `verdocs generate`.
2.  Upload the contents of your `out/` folder to your server (e.g., using `scp` or `rsync` to `/var/www/docs`).

### Nginx Configuration
Update your Nginx site configuration (usually found in `/etc/nginx/sites-available/`) to correctly handle Verdocs' internal routing:

```nginx
server {
    listen 80;
    server_name yourdomain.com;
    root /var/www/docs;
    index index.html;

    location / {
        # This ensures /v1.0.0/home correctly serves /v1.0.0/home/index.html
        try_files $uri $uri/ $uri.html =404;
    }

    # Optional: Cache control for assets
    location /assets/ {
        expires 1y;
        add_header Cache-Control "public, no-transform";
    }
}
```

After updating the configuration, test and restart Nginx:
```bash
sudo nginx -t
sudo systemctl restart nginx
```

---

## 3. GitHub Pages

GitHub Pages is a free and reliable option for hosting documentation alongside your source code.

### Deployment via Git Branch
1.  **Create a Deployment Branch:** Use `git checkout -b gh-pages`.
2.  **Prepare the Content:** Replace the root content of this branch with the contents of your `out/` folder.
3.  **Push to GitHub:** `git push origin gh-pages`.

### Deployment via GitHub Actions (Recommended)
You can automate the entire build and deployment process using GitHub Actions. Create a `.github/workflows/deploy.yml` file to run `verdocs generate` and deploy the `out/` folder whenever you push changes to your main branch.

**Note:** GitHub Pages handles folder-based routing (`/folder/` -> `/folder/index.html`) automatically, so no additional configuration is required.

---

## 4. Docker

For teams using container orchestration like Kubernetes or services like Coolify, you can easily containerize your documentation site.

### Create a Dockerfile
Create a `Dockerfile` in your project root using a lightweight Nginx image:

```dockerfile
FROM nginx:alpine

# Copy the generated site to the default Nginx html directory
COPY ./out /usr/share/nginx/html

# Expose port 80
EXPOSE 80
```

### Build and Run
1.  **Build the Image:** `docker build -t my-docs .`
2.  **Run the Container:** `docker run -p 8080:80 my-docs`

Your documentation will now be available at `http://localhost:8080`. The default Nginx configuration on Alpine will automatically resolve `index.html` files within your versioned directories.

---

## 5. Other Static Hosting Providers

Because the output is entirely static, Verdocs is compatible with many other platforms:
-   **Netlify:** Set the Publish directory to `out`.
-   **Cloudflare Pages:** Connect your Git repository and set the Output directory to `out`.
-   **AWS S3 + CloudFront:** Sync the `out/` folder to an S3 bucket and enable Static Website Hosting.
