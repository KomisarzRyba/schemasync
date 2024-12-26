pub struct Store {}

impl Store {
    pub fn save_schema(
        repo_owner: &str,
        repo_name: &str,
        schema_name: &str,
        schema_content: &str,
    ) -> anyhow::Result<()> {
        let home_dir = std::env::var("HOME")?;
        let schemastore_dir = std::path::Path::new(&home_dir).join(".schemastore");

        let repo_dir = schemastore_dir.join(repo_owner).join(repo_name);
        std::fs::create_dir_all(&repo_dir)?;

        let schema_path = repo_dir.join(schema_name);
        std::fs::write(&schema_path, schema_content)?;

        Ok(())
    }
}
