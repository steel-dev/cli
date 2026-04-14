use std::io::Write;

use clap::Parser;
use dialoguer::{Input, Select};
use serde::Deserialize;

use crate::status;

#[derive(Parser)]
pub struct Args {
    /// Template to start from
    pub template: Option<String>,

    /// Project name
    #[arg(short, long)]
    pub name: Option<String>,
}

#[derive(Deserialize)]
struct Manifest {
    examples: Vec<ManifestExample>,
    version: String,
}

#[derive(Deserialize)]
struct ManifestExample {
    slug: String,
    title: String,
    #[serde(default)]
    shorthand: Option<String>,
    #[serde(default)]
    template: Option<String>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    flags: Vec<String>,
}

fn load_manifest() -> anyhow::Result<Manifest> {
    let manifest_bytes = include_bytes!("../../manifest.json");
    let manifest: Manifest = serde_json::from_slice(manifest_bytes)?;
    Ok(manifest)
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let manifest = load_manifest()?;

    // Filter to CLI-available templates
    let cli_examples: Vec<&ManifestExample> = manifest
        .examples
        .iter()
        .filter(|e| e.flags.contains(&"cli".to_string()))
        .collect();

    if cli_examples.is_empty() {
        anyhow::bail!("No templates available.");
    }

    // Select template
    let example = if let Some(ref template_arg) = args.template {
        cli_examples
            .iter()
            .find(|e| e.slug == *template_arg || e.shorthand.as_deref() == Some(template_arg))
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Template not found: {template_arg}"))?
    } else {
        let items: Vec<String> = cli_examples
            .iter()
            .map(|e| {
                let lang = e.language.as_deref().unwrap_or("unknown");
                format!("{} [{}]", e.title, lang)
            })
            .collect();

        let selection = Select::new()
            .with_prompt("Select a template")
            .items(&items)
            .default(0)
            .interact()?;

        cli_examples[selection]
    };

    // Get project name
    let project_name = if let Some(ref name) = args.name {
        name.clone()
    } else {
        Input::new()
            .with_prompt("Project name")
            .default(example.slug.clone())
            .interact_text()?
    };

    // Download template
    let Some(ref template_path) = example.template else {
        anyhow::bail!("Template '{}' has no download path.", example.slug);
    };

    let download_url = format!(
        "https://registry.steel-edge.net/versions/{}/{}",
        manifest.version, template_path
    );

    status!("Downloading template '{}'...", example.title);

    let client = reqwest::Client::new();
    let response = client.get(&download_url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download template: HTTP {}", response.status());
    }

    let bytes = response.bytes().await?;

    // Extract tarball using system tar
    let target_dir = std::env::current_dir()?.join(&project_name);
    if target_dir.exists() {
        anyhow::bail!("Directory '{}' already exists.", project_name);
    }

    std::fs::create_dir_all(&target_dir)?;

    // Write to temp file then extract
    let tmp = target_dir.join(".template.tar.gz");
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(&bytes)?;
    }

    let status = std::process::Command::new("tar")
        .args(["xzf", &tmp.to_string_lossy(), "--strip-components=1"])
        .current_dir(&target_dir)
        .status()?;

    let _ = std::fs::remove_file(&tmp);

    if !status.success() {
        anyhow::bail!("Failed to extract template archive.");
    }

    status!(
        "Project '{}' created at {}",
        project_name,
        target_dir.display()
    );
    status!("\nNext steps:");
    status!("  cd {project_name}");

    match example.language.as_deref() {
        Some("python") => {
            status!("  pip install -r requirements.txt");
        }
        _ => {
            status!("  npm install");
        }
    }

    Ok(())
}
