use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use yansi::Paint;

use crate::Recipe;

pub enum RecipeSource {
    Local(String),
    Remote(String),
}

pub struct Config {
    dir: PathBuf,
}

impl Config {
    pub fn open() -> Result<Config> {
        let dir = std::env::var("FUJINX_CONFIG").unwrap_or_else(|_| "~/.fujinx".to_string());
        let dir = PathBuf::from(shellexpand::tilde(&dir).as_ref());
        let recipes_dir = dir.join("recipes");
        std::fs::create_dir_all(&recipes_dir)?;
        std::fs::create_dir_all(dir.join("repos"))?;

        if !recipes_dir.join(".git").exists() {
            let status = Command::new("git")
                .args(["init", "-q"])
                .current_dir(&recipes_dir)
                .status()?;
            if !status.success() {
                anyhow::bail!("git init failed in recipes directory");
            }

            let status = Command::new("git")
                .args(["commit", "--allow-empty", "-m", "Initial commit.", "-q"])
                .current_dir(&recipes_dir)
                .status()?;
            if !status.success() {
                anyhow::bail!("git initial commit failed in recipes directory");
            }
        }

        Ok(Config { dir })
    }

    /// Read a recipe by name or path.
    ///
    /// If `name` ends in `.yaml`, it is treated as a file path (with ~ expansion).
    /// Otherwise, it is looked up by name in user recipes, then repos.
    pub fn read_recipe(&self, name: &str) -> Result<Recipe> {
        if name.ends_with(".yaml") {
            let path = PathBuf::from(shellexpand::tilde(name).as_ref());

            return read_recipe_file(&path);
        }

        let user_path = self.dir.join(format!("recipes/{name}.yaml"));
        if user_path.exists() {
            return read_recipe_file(&user_path);
        }

        for repo in self.repos()? {
            let path = repo.join(format!("{name}.yaml"));
            if path.exists() {
                return read_recipe_file(&path);
            }
        }

        anyhow::bail!("recipe not found: {}", name.paint(crate::BLUE))
    }

    pub fn write_recipe(&self, name: &str, recipe: &Recipe, force: bool) -> Result<PathBuf> {
        let path = self.dir.join(format!("recipes/{name}.yaml"));
        if !force && path.exists() {
            anyhow::bail!(
                "recipe already exists (use --force to overwrite): {}",
                name.paint(crate::BLUE)
            );
        }
        let yaml = serde_yaml::to_string(recipe)?;
        std::fs::write(&path, yaml)?;
        self.commit_recipe(name, "Update")?;

        Ok(path)
    }

    pub fn delete_recipe(&self, name: &str) -> Result<()> {
        let path = self.dir.join(format!("recipes/{name}.yaml"));
        if !path.exists() {
            anyhow::bail!("recipe not found: {}", name.paint(crate::BLUE));
        }
        std::fs::remove_file(&path)?;
        self.commit_recipe(name, "Delete")?;

        Ok(())
    }

    fn commit_recipe(&self, name: &str, action: &str) -> Result<()> {
        let recipes_dir = self.dir.join("recipes");
        let status = Command::new("git")
            .args(["add", "-A"])
            .current_dir(&recipes_dir)
            .status()?;
        if !status.success() {
            anyhow::bail!("git add failed in recipes directory");
        }

        let message = format!("{action} {name}.");
        let status = Command::new("git")
            .args(["commit", "-m", &message, "--allow-empty-message", "-q"])
            .current_dir(&recipes_dir)
            .status()?;
        if !status.success() {
            // Not an error — nothing to commit (e.g. saving identical content).
        }

        Ok(())
    }

    /// Add a recipe repo by cloning a git URL.
    pub fn add_repo(&self, url: &str) -> Result<String> {
        let path = repo_path(url)?;
        let dest = self.dir.join(format!("repos/{path}"));
        if dest.exists() {
            anyhow::bail!("repo already exists: {}", path.paint(crate::BLUE));
        }

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let status = Command::new("git")
            .args(["clone", "--depth", "1", url])
            .arg(&dest)
            .status()?;
        if !status.success() {
            anyhow::bail!("git clone failed");
        }

        Ok(path)
    }

    /// Remove a repo by name.
    pub fn remove_repo(&self, name: &str) -> Result<()> {
        let dest = self.dir.join(format!("repos/{name}"));
        if !dest.exists() {
            anyhow::bail!("repo not found: {}", name.paint(crate::BLUE));
        }
        std::fs::remove_dir_all(&dest)?;

        // Clean up empty parent directories.
        let repos_dir = self.dir.join("repos");
        let mut dir = dest.parent();
        while let Some(parent) = dir {
            if parent == repos_dir {
                break;
            }
            if std::fs::read_dir(parent)?.next().is_none() {
                std::fs::remove_dir(parent)?;
            } else {
                break;
            }
            dir = parent.parent();
        }

        Ok(())
    }

    /// Pull latest changes for a repo by name.
    pub fn update_repo(&self, name: &str) -> Result<()> {
        let repo = self.dir.join(format!("repos/{name}"));
        let status = Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(&repo)
            .status()?;
        if !status.success() {
            anyhow::bail!("git pull failed for repo: {}", name.paint(crate::BLUE));
        }

        Ok(())
    }

    /// List all available recipes, grouped by name with all repos listed.
    /// Repos are in priority order (local first, then repos alphabetically).
    pub fn list_recipes(&self) -> Result<Vec<(String, Vec<RecipeSource>)>> {
        let mut map: std::collections::BTreeMap<String, Vec<RecipeSource>> =
            std::collections::BTreeMap::new();

        for name in self.yaml_names_in(&self.dir.join("recipes"))? {
            map.entry(name)
                .or_default()
                .push(RecipeSource::Local("local".to_string()));
        }

        let repos_dir = self.dir.join("repos");
        for repo_dir in self.repos()? {
            let repo_name = repo_dir
                .strip_prefix(&repos_dir)
                .unwrap()
                .to_string_lossy()
                .to_string();
            for name in self.yaml_names_in(&repo_dir)? {
                map.entry(name)
                    .or_default()
                    .push(RecipeSource::Remote(repo_name.clone()));
            }
        }

        Ok(map.into_iter().collect())
    }

    fn yaml_names_in(&self, dir: &Path) -> Result<Vec<String>> {
        let mut names = Vec::new();
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(names),
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "yaml")
                && let Some(stem) = path.file_stem()
            {
                names.push(stem.to_string_lossy().to_string());
            }
        }
        names.sort();

        Ok(names)
    }

    /// List recipes grouped by source (local + each repo).
    pub fn list_recipes_per_source(&self) -> Result<Vec<(RecipeSource, Vec<String>)>> {
        let local_names = self.yaml_names_in(&self.dir.join("recipes"))?;
        let mut sources = Vec::new();
        if !local_names.is_empty() {
            sources.push((RecipeSource::Local("local".to_string()), local_names));
        }

        let repos_dir = self.dir.join("repos");
        for repo_dir in self.repos()? {
            let repo_name = repo_dir
                .strip_prefix(&repos_dir)
                .unwrap()
                .to_string_lossy()
                .to_string();
            let names = self.yaml_names_in(&repo_dir)?;
            sources.push((RecipeSource::Remote(repo_name), names));
        }

        Ok(sources)
    }

    /// List repo names (paths relative to ~/.fujinx/repos/).
    pub fn list_repos(&self) -> Result<Vec<String>> {
        let repos_dir = self.dir.join("repos");
        let repos = self.repos()?;

        Ok(repos
            .iter()
            .map(|r| {
                r.strip_prefix(&repos_dir)
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect())
    }

    /// List repo directories by finding git repos under ~/.fujinx/repos/.
    pub fn repos(&self) -> Result<Vec<PathBuf>> {
        let repos_dir = self.dir.join("repos");
        let mut repos = Vec::new();
        Self::find_repos(&repos_dir, &mut repos)?;
        repos.sort();

        Ok(repos)
    }

    fn find_repos(dir: &Path, repos: &mut Vec<PathBuf>) -> Result<()> {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(()),
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if path.join(".git").exists() {
                repos.push(path);
            } else {
                Self::find_repos(&path, repos)?;
            }
        }

        Ok(())
    }
}

fn read_recipe_file(path: &Path) -> Result<Recipe> {
    let yaml = std::fs::read_to_string(path)?;
    let recipe = serde_yaml::from_str(&yaml)?;

    Ok(recipe)
}

/// Extract a repo path from a git URL.
///
/// Examples:
///   git@github.com:calder/fujixweekly.git -> github.com/calder/fujixweekly
///   https://github.com/calder/fujixweekly.git -> github.com/calder/fujixweekly
fn repo_path(url: &str) -> Result<String> {
    let url = url.trim_end_matches('/').trim_end_matches(".git");

    // SSH: git@github.com:calder/fujixweekly
    if let Some(rest) = url.strip_prefix("git@") {
        let path = rest.replacen(':', "/", 1);

        return Ok(path);
    }

    // HTTPS: https://github.com/calder/fujixweekly
    if let Some(rest) = url.strip_prefix("https://") {
        return Ok(rest.to_string());
    }
    if let Some(rest) = url.strip_prefix("http://") {
        return Ok(rest.to_string());
    }

    anyhow::bail!("could not parse repo path from URL: {url}")
}
