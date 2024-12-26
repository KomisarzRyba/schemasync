pub trait Tree {
    fn find_schemas(&self) -> Vec<String>;
    fn get_schema(&self, path: &str) -> anyhow::Result<String>;
}
