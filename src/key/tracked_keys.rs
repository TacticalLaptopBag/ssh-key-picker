use std::{fs, io::{self, Write}, path::PathBuf};

use anyhow::{Context, anyhow, bail};
use serde::{Deserialize, Serialize};
use colored::Colorize;

use crate::key::{KeyType, TrackedKey};


#[derive(Debug, Serialize, Deserialize)]
pub struct TrackedKeys {
    pub active: Option<String>,
    pub keys: Vec<TrackedKey>,

    #[serde(skip)]
    keys_dir: PathBuf,
    #[serde(skip)]
    disabled_dir: PathBuf,
}

fn rename_with_context(from: &PathBuf, to: &PathBuf) -> anyhow::Result<()> {
    fs::rename(from, to)
        .context(format!(
            "Failed to move file from {} to {}",
            from.display(), to.display(),
        ))
}

fn remove_file_with_context(file: &PathBuf) -> anyhow::Result<()> {
    fs::remove_file(file)
        .context(format!(
            "Failed to remove file {}",
            file.display(),
        ))
}

impl TrackedKeys {
    pub fn load(path: &PathBuf, keys_dir: PathBuf, disabled_dir: PathBuf) -> anyhow::Result<TrackedKeys> {
        if path.try_exists().is_ok_and(|exists| exists) {
            let data = fs::read_to_string(path)
                .context("Failed to read keys file")?;
            let mut tracked_keys: TrackedKeys = serde_json::from_str(&data)
                .context("Failed to parse tracked keys")?;
            tracked_keys.keys_dir = keys_dir;
            tracked_keys.disabled_dir = disabled_dir;
            Ok(tracked_keys)
        } else {
            Ok(TrackedKeys {
                active: None,
                keys: vec![],
                keys_dir,
                disabled_dir,
            })
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

    pub fn activate_key(&mut self, key: &TrackedKey) -> anyhow::Result<()> {
        let disabled_paths = key.get_disabled_paths(&self.disabled_dir);
        let enabled_paths = key.get_enabled_paths(&self.keys_dir);
        rename_with_context(&disabled_paths.private, &enabled_paths.private)?;
        rename_with_context(&disabled_paths.public, &enabled_paths.public)?;
        self.active = Some(key.name.clone());
        Ok(())
    }

    pub fn deactivate_key(&mut self) -> anyhow::Result<()> {
        if let Some(active_key) = self.get_active_key() {
            let disabled_paths = active_key.get_disabled_paths(&self.disabled_dir);
            let enabled_paths = active_key.get_enabled_paths(&self.keys_dir);

            rename_with_context(&enabled_paths.private, &disabled_paths.private)?;
            rename_with_context(&enabled_paths.public, &disabled_paths.public)?;
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

    fn update_active_state(&mut self) -> anyhow::Result<bool> {
        let mut dirty = false;

        if let Some(active_key) = self.get_active_key() {
            let enabled_paths = active_key.get_enabled_paths(&self.keys_dir);
            let disabled_paths = active_key.get_disabled_paths(&self.disabled_dir);
            let does_private_exist = fs::exists(&enabled_paths.private)?;
            let does_public_exist = fs::exists(&enabled_paths.public)?;
            if !does_private_exist || !does_public_exist {
                // Private or public active key is missing. Did we try to disable it?
                let is_private_disabled = fs::exists(&disabled_paths.private)?;
                let is_public_disabled = fs::exists(&disabled_paths.public)?;
                if does_private_exist && !is_private_disabled {
                    rename_with_context(&enabled_paths.private, &disabled_paths.private)?;
                }
                if does_public_exist && !is_public_disabled {
                    rename_with_context(&enabled_paths.public, &disabled_paths.public)?;
                }

                self.active = None;
                dirty = true;
            }
        }

        Ok(dirty)
    }

    fn update_tracked_keys(&mut self) -> anyhow::Result<bool> {
        let mut dirty = false;

        let mut keys_to_remove = vec![];
        for (i, key) in self.keys.iter().enumerate() {
            if Some(&key.name) == self.active.as_ref() {
                continue;
            }
            let disabled_paths = key.get_disabled_paths(&self.disabled_dir);
            let does_private_exist = fs::exists(&disabled_paths.private)?;
            let does_public_exist = fs::exists(&disabled_paths.public)?;
            let lost_and_found_dir = self.disabled_dir.parent()
                .ok_or(anyhow!("Disabled keys directory is in an invalid location!"))?
                .join("lost-and-found");
            if !does_private_exist || !does_public_exist {
                println!("{}", format!("Tracked key {} appears to be missing!", key.name.red()).bold());
                println!("This key will be removed from the list of tracked keys.");
                if does_private_exist || does_public_exist {
                    fs::create_dir_all(&lost_and_found_dir)
                        .context(format!("Failed to create directory {}", lost_and_found_dir.display()))?;
                    println!(
                        "Remnants of this key were found, and will be moved to {}",
                        lost_and_found_dir.display().to_string().red(),
                    )
                }
                
                if does_private_exist {
                    rename_with_context(&disabled_paths.private, &lost_and_found_dir.join(&key.name))?;
                }
                if does_public_exist {
                    rename_with_context(&disabled_paths.public, &lost_and_found_dir.join(&key.name).with_extension("pub"))?;
                }

                keys_to_remove.push(i);
                dirty = true;
            }
        }

        for i in keys_to_remove {
            self.keys.remove(i);
        }

        Ok(dirty)
    }

    /// Checks actual locations of key files and updates the internal state to reflect them
    pub fn update_state(&mut self) -> anyhow::Result<bool> {
        let mut changes_made = self.update_active_state()?;
        changes_made = self.update_tracked_keys()? || changes_made;
        Ok(changes_made)
    }

    pub fn find_untracked_keys(&mut self, no_prompt: bool) -> anyhow::Result<bool> {
        let files = fs::read_dir(&self.keys_dir)
            .context(format!("Failed to read from {}", self.keys_dir.display()))?;

        let active_key_paths = self.get_active_key().map(|k| k.get_enabled_paths(&self.keys_dir));

        let mut keys_added = false;
        for entry_result in files {
            let Ok(entry) = entry_result else { continue };
            // Check this key isn't the currently active key
            if let Some(active_paths) = &active_key_paths {
                if entry.path() == active_paths.private || entry.path() == active_paths.public {
                    continue;
                }
            }

            let Ok(contents) = fs::read_to_string(entry.path()) else { continue };
            let contents_split: Vec<&str> = contents.split_whitespace().collect();
            let Some(key_type_str) = contents_split.first() else { continue };

            let Ok(key_type) = KeyType::from_type(key_type_str) else { continue };
            // This seems to be a valid key, prompt user for name
            println!("Found untracked key: {}", entry.path().display());
            let comment = contents_split.get(2..).map(|s| s.join(" ")).filter(|s| !s.is_empty());
            if let Some(ref comment) = comment {
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

            let public_path = entry.path().with_extension("pub");
            let private_path = entry.path().with_extension("");
            let disabled_paths = tracked_key.get_disabled_paths(&self.disabled_dir);
            rename_with_context(&private_path, &disabled_paths.private)?;
            rename_with_context(&public_path, &disabled_paths.public)?;
            self.keys.push(tracked_key);
            keys_added = true;
        }

        Ok(keys_added)
    }

    pub fn rename(&mut self, key: &TrackedKey, new_name: String) -> anyhow::Result<()> {
        let old_name = key.name.clone();

        if self.find_key(&new_name).is_some() {
            bail!("This name has already been taken!");
        }

        if let Some(internal_key) = self.keys.iter_mut().find(|k| k.name == old_name) {
            if self.active.as_deref() == Some(&internal_key.name) {
                self.active = Some(new_name.clone());
                // Key is currently active, don't need to rename it
            } else {
                let disabled_path = self.disabled_dir.join(&old_name);
                let disabled_pub_path = disabled_path.with_extension("pub");
                let new_path = disabled_path.with_file_name(&new_name);
                let new_pub_path = new_path.with_extension("pub");
                rename_with_context(&disabled_path, &new_path)?;
                rename_with_context(&disabled_pub_path, &new_pub_path)?;
            }
            
            internal_key.name = new_name;
        } else {
            bail!("Provided key is not tracked!")
        }

        Ok(())
    }

    pub fn delete(&mut self, key: &TrackedKey, no_prompt: bool) -> anyhow::Result<bool> {
        if !no_prompt {
            let mut response = String::new();
            print!("Are you sure you want to delete key {}? (y/N): ", key.name.red());
            io::stdout().flush()?;
            io::stdin().read_line(&mut response)?;
            if !response.to_lowercase().starts_with("y") {
                return Ok(false)
            }
        }
        
        let key_index = self.keys.iter().position(|k| k.name == key.name).ok_or(anyhow!("Provided key is not tracked!"))?;
        // key_index must be in keys, so this is a safe unwrap
        let internal_key = self.keys.get_mut(key_index).unwrap();
        if self.active.as_ref() == Some(&internal_key.name) {
            self.active = None;

            let private_name = internal_key.key_type.to_file_name();
            let private_path = self.keys_dir.join(private_name);
            let public_path = private_path.with_extension("pub");
            remove_file_with_context(&private_path)?;
            remove_file_with_context(&public_path)?;
        } else {
            let disabled_path = self.disabled_dir.join(&internal_key.name);
            let disabled_pub_path = disabled_path.with_extension("pub");
            remove_file_with_context(&disabled_path)?;
            remove_file_with_context(&disabled_pub_path)?;
        }

        self.keys.remove(key_index);
        Ok(true)
    }
}
