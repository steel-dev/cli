use clap::Parser;
use dialoguer::Select;
use serde::Deserialize;

#[derive(Parser)]
pub struct Args {
    /// Template to run
    pub template: Option<String>,

    /// Task to run
    #[arg(short, long)]
    pub task: Option<String>,

    /// Open live session viewer
    #[arg(short = 'o', long)]
    pub view: bool,

    /// API URL for Steel API
    #[arg(short = 'a', long)]
    pub api_url: Option<String>,

    /// API Key for Steel API
    #[arg(long)]
    pub api_key: Option<String>,

    /// API Key for OpenAI
    #[arg(long)]
    pub openai_key: Option<String>,

    /// API Key for Anthropic
    #[arg(long)]
    pub anthropic_key: Option<String>,

    /// API Key for Gemini
    #[arg(long)]
    pub gemini_key: Option<String>,

    /// Skip authentication
    #[arg(long)]
    pub skip_auth: bool,
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
    #[serde(default)]
    env: Vec<ManifestEnvVar>,
    #[serde(default)]
    run: Option<String>,
}

#[derive(Deserialize)]
struct ManifestEnvVar {
    value: String,
    label: String,
    #[serde(default)]
    required: Option<bool>,
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
            .find(|e| {
                e.slug == *template_arg
                    || e.shorthand.as_deref() == Some(template_arg)
            })
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
            .with_prompt("Select a template to run")
            .items(&items)
            .default(0)
            .interact()?;

        cli_examples[selection]
    };

    // Download template
    let Some(ref template_path) = example.template else {
        anyhow::bail!("Template '{}' has no run command.", example.slug);
    };

    let download_url = format!(
        "https://raw.githubusercontent.com/steel-dev/steel-cookbook/{}/{}",
        manifest.version, template_path
    );

    println!("Downloading template '{}'...", example.title);

    let client = reqwest::Client::new();
    let response = client.get(&download_url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download template: HTTP {}", response.status());
    }

    let bytes = response.bytes().await?;

    // Extract to temp directory
    let tmp_dir = std::env::temp_dir().join(format!("steel-run-{}", example.slug));
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }
    std::fs::create_dir_all(&tmp_dir)?;

    let tmp_archive = tmp_dir.join("template.tar.gz");
    std::fs::write(&tmp_archive, &bytes)?;

    let status = std::process::Command::new("tar")
        .args(["xzf", &tmp_archive.to_string_lossy(), "--strip-components=1"])
        .current_dir(&tmp_dir)
        .status()?;

    let _ = std::fs::remove_file(&tmp_archive);

    if !status.success() {
        anyhow::bail!("Failed to extract template archive.");
    }

    // Collect environment variables
    let auth = crate::config::auth::resolve_auth();
    let mut env_vars: Vec<(String, String)> = Vec::new();

    if let Some(ref key) = auth.api_key {
        env_vars.push(("STEEL_API_KEY".to_string(), key.clone()));
    }

    if let Some(ref task) = args.task {
        env_vars.push(("TASK".to_string(), task.clone()));
    }

    // Prompt for required env vars that aren't set
    for env_var in &example.env {
        if env_var.value == "STEEL_API_KEY" || env_var.value == "TASK" {
            continue;
        }
        if std::env::var(&env_var.value).is_ok() {
            continue;
        }
        if env_var.required == Some(true) {
            let val: String = dialoguer::Input::new()
                .with_prompt(&env_var.label)
                .interact_text()?;
            env_vars.push((env_var.value.clone(), val));
        }
    }

    // Install dependencies and run
    println!("Running '{}'...", example.title);

    let run_cmd = example.run.as_deref().unwrap_or(
        match example.language.as_deref() {
            Some("python") => "pip install -r requirements.txt && python main.py",
            _ => "npm install && npm start",
        }
    );

    let mut cmd = std::process::Command::new("sh");
    cmd.args(["-c", run_cmd])
        .current_dir(&tmp_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    for (k, v) in &env_vars {
        cmd.env(k, v);
    }

    let status = cmd.status()?;

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp_dir);

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
