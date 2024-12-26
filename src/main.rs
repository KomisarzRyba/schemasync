use clap::{Parser, Subcommand};
use tree::Tree;

mod gh;
mod store;
mod tree;

#[derive(Parser)]
#[command(name = "schemastore")]
struct App {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pull schema from GitHub repository
    Pull {
        /// Owner of the repository
        owner: String,
        /// Name of the repository
        repo: String,
        /// Branch to pull from
        #[arg(short, long, default_value = "main")]
        branch: String,
    },
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();

    match app.command {
        Commands::Pull {
            owner,
            repo,
            branch,
        } => pull_schema(&owner, &repo, &branch),
    }
}

fn pull_schema(owner: &str, repo_name: &str, branch: &str) -> anyhow::Result<()> {
    let gh = gh::Gh::from_env()?;
    let repo = gh.get_repo(owner, repo_name, branch)?;
    let schema_paths = repo.find_schemas();

    let selected_schema_index = dialoguer::Select::new()
        .with_prompt("Found following schemas. Select one to pull")
        .items(&schema_paths)
        .interact_opt()?
        .ok_or(anyhow::anyhow!("No schema selected"))?;

    let selected_schema_path = &schema_paths[selected_schema_index];

    let schema_content = repo.get_schema(selected_schema_path)?;

    let schema_name = selected_schema_path.split('/').last().unwrap();
    store::Store::save_schema(owner, repo_name, schema_name, &schema_content)?;

    println!("Schema saved to ~/.schemastore/{owner}/{repo_name}/{schema_name}");

    Ok(())
}
