use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use crate::config::Config;

pub fn init_project(path: &PathBuf) -> Result<()> {
    // Check if project is already initialized
    if path.join("config.yml").exists() {
        return Err(anyhow::anyhow!(
            "Project already initialized at {:?}. Use 'verdocs clean' first if you want to reset.",
            path
        ));
    }

    fs::create_dir_all(path.join("assets"))?;
    fs::create_dir_all(path.join("search-index"))?;

    // Embed and write the sample image
    let sample_image = include_bytes!("../resources/sample-image.png");
    fs::write(path.join("assets/main.go.png"), sample_image)?;

    // Create v1.0.0
    create_version_folder(path, "v1.0.0", false)?;
    
    // Create v1.1.0 with some changes
    create_version_folder(path, "v1.1.0", true)?;

    let config = Config::default();
    let config_yaml = serde_yaml::to_string(&config)?;
    fs::write(path.join("config.yml"), config_yaml)?;

    Ok(())
}

fn create_version_folder(root: &PathBuf, version: &str, is_updated: bool) -> Result<()> {
    let version_path = root.join(version);
    
    // Base folders
    let folders = vec![
        "home",
        "quick-start",
        "api-integrations",
        "reference",
    ];

    for folder in folders {
        let folder_path = version_path.join(folder);
        fs::create_dir_all(&folder_path)?;
        
        let folder_name = folder_path.file_name().unwrap().to_str().unwrap();
        let index_file = folder_path.join(format!("{}.md", folder_name));
        
        let mut content = format!("# {}\n\nThis is the {} page for version {}.\n", 
            folder_name.to_uppercase(), folder_name, version);
            
        if folder == "home" {
            content.push_str(r#"
### 1. Linking and Assets
You can easily link to other parts of your documentation or external sites.

*   **Internal Link:** Check out the [Quick Start Guide](../quick-start/quick-start.md) to get moving.
*   **External Link:** Visit the [Verdocs GitHub](https://github.com) for more info.
*   **Image Support:**
    ![Main Go Sample](../../assets/main.go.png)

"#);
            if is_updated {
                content.push_str("\n{TIP type=\"admonition\" title=\"New in this version\"}\nWe have added a new Features section!\n{/TIP}\n");
            }
        }
        
        if folder == "quick-start" {
            content.push_str(r#"
## Introduction
This is an introduction to the quick start guide.

## Installation
How to install the project.

### Step 1: Download
Download the binary from the releases page.

### Step 2: Extract
Extract the archive to your desired location.

## Configuration
Basic configuration steps.

## Next Steps
Where to go from here.
"#);
        }

        fs::write(index_file, content)?;
    }

    // Version specific files
    if !is_updated {
        // v1.0.0 specific
        fs::write(version_path.join("api-integrations/node-js.md"), "# Node.js (Legacy)\n")?;
    } else {
        // v1.1.0 specific
        let features_path = version_path.join("features");
        fs::create_dir_all(&features_path)?;
        fs::write(features_path.join("features.md"), "# New Features\n\nCheck out our latest updates!\n")?;
        
        fs::write(version_path.join("api-integrations/node-js.md"), "# Node.js (Updated)\n")?;
        fs::write(version_path.join("api-integrations/coolify.md"), "# Coolify Integration\n")?;
    }

    Ok(())
}
