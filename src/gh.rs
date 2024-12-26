use base64::{engine::general_purpose::STANDARD, Engine as _};

pub struct Gh {
    api_token: String,
}

impl Gh {
    pub fn new(api_token: String) -> Self {
        Self { api_token }
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let api_token = std::env::var("GH_API_TOKEN")?;
        Ok(Self::new(api_token))
    }

    pub fn get_repo(self, owner: &str, name: &str, sha: &str) -> anyhow::Result<GhRepo> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github+json".parse()?,
        );
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.api_token).parse()?,
        );
        headers.insert("X-GitHub-Api-Version", "2022-11-28".parse()?);

        let client = reqwest::blocking::ClientBuilder::new()
            .user_agent("schemasync")
            .default_headers(headers)
            .build()?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
            owner, name, sha
        );

        let response = client.get(url).send()?;
        let response = match response.status() {
            reqwest::StatusCode::OK => response,
            _ => anyhow::bail!("Failed to fetch repo tree: {}", response.text()?),
        };
        let response: GhRepoResponse = response.json()?;

        Ok(GhRepo {
            owner: owner.to_string(),
            name: name.to_string(),
            token: self.api_token,
            entries: response.tree,
        })
    }
}

#[derive(serde::Deserialize)]
struct GhRepoResponse {
    tree: Vec<GhResponseEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct GhResponseEntry {
    path: String,
    r#type: GhResponseEntryKind,
}

#[derive(Debug, serde::Deserialize)]
pub enum GhResponseEntryKind {
    #[serde(rename = "blob")]
    Blob,
    #[serde(rename = "tree")]
    Tree,
}

#[derive(Debug)]
pub struct GhRepo {
    owner: String,
    name: String,
    token: String,
    entries: Vec<GhResponseEntry>,
}

impl crate::tree::Tree for GhRepo {
    fn find_schemas(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter_map(|entry| match entry.r#type {
                GhResponseEntryKind::Blob => {
                    if entry.path.ends_with("schema.json") {
                        Some(entry.path.clone())
                    } else {
                        None
                    }
                }
                GhResponseEntryKind::Tree => None,
            })
            .collect()
    }

    fn get_schema(&self, path: &str) -> anyhow::Result<String> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github+json".parse()?,
        );
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.token).parse()?,
        );
        headers.insert("X-GitHub-Api-Version", "2022-11-28".parse()?);

        let client = reqwest::blocking::ClientBuilder::new()
            .user_agent("schemasync")
            .default_headers(headers)
            .build()?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.owner, self.name, path
        );

        let response = client.get(url).send()?;
        let response = match response.status() {
            reqwest::StatusCode::OK => response,
            _ => anyhow::bail!("Failed to fetch schema file: {}", response.text()?),
        };

        #[derive(serde::Deserialize)]
        struct GhContentResponse {
            content: String,
            encoding: String,
        }

        let response: GhContentResponse = response.json()?;
        if response.encoding != "base64" {
            anyhow::bail!("Unexpected encoding: {}", response.encoding);
        }

        let content = response.content.replace("\n", "").replace("\r", "");
        let content = STANDARD.decode(content)?;

        Ok(String::from_utf8(content)?)
    }
}
