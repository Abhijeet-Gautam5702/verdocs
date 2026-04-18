use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use crate::config::Config;

pub fn init_project(path: &PathBuf) -> Result<()> {
    fs::create_dir_all(path.join("assets"))?;
    fs::create_dir_all(path.join("search-index"))?;

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
            
        if folder == "home" && is_updated {
            content.push_str("\n{TIP type=\"admonition\" title=\"New in this version\"}\nWe have added a new Features section!\n{/TIP}\n");
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
