use std::{fs, io::{self, Write}, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::key::{KeyType, TrackedKey};


#[derive(Debug, Serialize, Deserialize)]
pub struct TrackedKeys {
    pub active: Option<String>,
    pub keys: Vec<TrackedKey>,
}

fn rename_with_context(from: &PathBuf, to: &PathBuf) -> anyhow::Result<()> {
    fs::rename(from, to)
        .context(format!(
            "Failed to move file from {} to {}",
            from.display(), to.display(),
        ))
}

impl TrackedKeys {
    pub fn load(path: &PathBuf) -> anyhow::Result<TrackedKeys> {
        if path.try_exists().is_ok_and(|exists| exists) {
            let data = fs::read_to_string(path)
                .context("Failed to read keys file")?;
            let tracked_keys: TrackedKeys = serde_json::from_str(&data)
                .context("Failed to parse tracked keys")?;
            Ok(tracked_keys)
        } else {
            Ok(TrackedKeys { active: None, keys: vec![] })
        }
    }

    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let data = serde_json::to_string(self)
            .context("Failed to serialize keys data")?;
        fs::write(path, data)
            .context("Failed to save keys data")?;
        Ok(())
    }

    pub fn get_active_key(&self) -> Option<&TrackedKey> {
        if let Some(active_name) = &self.active {
            self.find_key(active_name)
        } else {
            None
        }
    }

    fn find_key(&self, name: &str) -> Option<&TrackedKey> {
        self.keys.iter().find(|key| key.name.as_str() == name)
    }

    pub fn find_key_by_partial(&self, partial_name: &str) -> Option<&TrackedKey> {
        if let Some(key) = self.find_key(partial_name) {
            return Some(key);
        }

        self.keys.iter().find(|key| key.name.to_lowercase().starts_with(&partial_name.to_lowercase()))
    }

    pub fn activate_key(&mut self, key: &TrackedKey, keys_dir: &PathBuf, disabled_dir: &PathBuf) -> anyhow::Result<()> {
        let disabled_name = &key.name;
        let disabled_path = disabled_dir.join(disabled_name);
        let disabled_pub_path = disabled_path.with_extension("pub");

        let private_name = key.key_type.to_file_name();
        let private_path = keys_dir.join(private_name);
        let public_path = private_path.with_extension("pub");

        rename_with_context(&disabled_path, &private_path)?;
        rename_with_context(&disabled_pub_path, &public_path)?;
        self.active = Some(key.name.clone());
        Ok(())
    }

    pub fn deactivate_key(&mut self, keys_dir: &PathBuf, disabled_dir: &PathBuf) -> anyhow::Result<()> {
        if let Some(active_key) = self.get_active_key() {
            let private_name = active_key.key_type.to_file_name();
            let private_path = keys_dir.join(private_name);
            let public_path = private_path.with_extension("pub");
            
            let disabled_name = &active_key.name;
            let disabled_path = disabled_dir.join(disabled_name);
            let disabled_pub_path = disabled_path.with_extension("pub");

            rename_with_context(&private_path, &disabled_path)?;
            rename_with_context(&public_path, &disabled_pub_path)?;
            self.active = None;
        }
        Ok(())
    }

    fn prompt_for_name(&self) -> anyhow::Result<String> {
        let mut name = String::new();
        let stdin = io::stdin();

        let mut name_ok = false;
        while !name_ok {
            print!("Enter a name for this key (Leave blank to skip): ");
            io::stdout().flush()?;
            stdin.read_line(&mut name)?;
            if self.find_key(&name).is_none() {
                name_ok = true;
            } else {
                println!("This name has already been taken! Try a different one.");
            }
        }
        
        Ok(name.trim().into())
    }

    pub fn find_untracked_keys(&mut self, no_prompt: bool, keys_dir: &PathBuf, disabled_dir: &PathBuf) -> anyhow::Result<bool> {
        let files = fs::read_dir(&keys_dir)
            .context(format!("Failed to read from {}", keys_dir.display()))?;

        let mut keys_added = false;
        for entry_result in files {
            let Ok(entry) = entry_result else { continue };
            let Ok(contents) = fs::read_to_string(entry.path()) else { continue };
            let contents_split: Vec<&str> = contents.split_whitespace().collect();
            let Some(key_type_str) = contents_split.first() else { continue };

            let Ok(key_type) = KeyType::from_type(key_type_str) else { continue };
            // This seems to be a valid key, prompt user for name
            println!("Found untracked key: {}", entry.path().display());
            // TODO: Get slice from 2 to end
            let comment = contents_split.get(2);
            if let Some(comment) = comment {
                println!("  Comment: {}", comment)
            }

            let name = if no_prompt {
                "".into()
            } else {
                self.prompt_for_name()?
            };
            if name.is_empty() {
                continue;
            }

            let tracked_key = TrackedKey { name, key_type };

            let public_path = entry.path();
            let private_path = entry.path().with_extension("");
            let disabled_path = disabled_dir.join(&tracked_key.name);
            let disabled_pub_path = disabled_dir.join(&tracked_key.name).with_extension("pub");
            rename_with_context(&public_path, &disabled_pub_path)?;
            rename_with_context(&private_path, &disabled_path)?;
            self.keys.push(tracked_key);
            keys_added = true;
        }

        Ok(keys_added)
    }
}
