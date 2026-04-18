use anyhow::{Result, anyhow};
use std::fs;
use std::path::PathBuf;
use tiny_http::{Header, Response, Server};
use regex::Regex;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub fn start_server(out_dir: &PathBuf, port: u16, version: Arc<AtomicU64>) -> Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let server = Server::http(&addr).map_err(|e| anyhow!("Failed to start server: {}", e))?;

    println!("Preview server running at: http://{}", addr);
    println!("Press Ctrl+C to stop.");

    for request in server.incoming_requests() {
        let url = request.url();

        if url == "/__verdocs/status" {
            let v = version.load(Ordering::SeqCst);
            let response = Response::from_string(v.to_string());
            let _ = request.respond(response);
            continue;
        }

        // Remove leading slash and handle index.html
        let mut path = url.trim_start_matches('/').to_string();
        if path.is_empty() {
            path = "index.html".to_string(); // Placeholder for future root
        }

        let mut file_path = out_dir.join(&path);

        if !file_path.exists() || !file_path.is_file() {
            // Try appending .html
            let with_html = out_dir.join(format!("{}.html", path));
            if with_html.exists() && with_html.is_file() {
                file_path = with_html;
            } else {
                // Try index.html in directory
                let with_index = out_dir.join(&path).join("index.html");
                if with_index.exists() && with_index.is_file() {
                    file_path = with_index;
                }
            }
        }

        if file_path.exists() && file_path.is_file() {
            let content = fs::read(&file_path)?;
            let mime_type = get_mime_type(&file_path);
            let response = Response::from_data(content).with_header(
                Header::from_bytes(&b"Content-Type"[..], mime_type.as_bytes()).unwrap(),
            );
            let _ = request.respond(response);
        } else {
            // Check for folder index
            let index_path = file_path.join("index.html");
            if index_path.exists() {
                let content = fs::read(&index_path)?;
                let response = Response::from_data(content)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], "text/html").unwrap());
                let _ = request.respond(response);
                continue;
            }

            let response = Response::from_string(render_404(&path))
                .with_header(Header::from_bytes(&b"Content-Type"[..], "text/html").unwrap())
                .with_status_code(404);
            let _ = request.respond(response);
        }
    }

    Ok(())
}

fn render_404(path: &str) -> String {
    let re = Regex::new(r"^(v[0-9][^/]*)/").unwrap();
    let version_info = if let Some(caps) = re.captures(path) {
        format!(" on version <strong>{}</strong>", &caps[1])
    } else {
        "".to_string()
    };

    format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>404 - Not Found</title>
    <style>
        body {{ font-family: -apple-system, sans-serif; padding: 4rem; text-align: center; color: #333; }}
        h1 {{ font-size: 3rem; margin-bottom: 1rem; }}
        p {{ font-size: 1.2rem; color: #666; }}
        .home-link {{ display: inline-block; margin-top: 2rem; color: #007bff; text-decoration: none; font-weight: bold; }}
    </style>
</head>
<body>
    <h1>404</h1>
    <p>The section you're looking for doesn't exist{}.</p>
    <a href="javascript:history.back()" class="home-link">← Go back to where you were</a>
</body>
</html>"#, version_info)
}

fn get_mime_type(path: &std::path::Path) -> &str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}
