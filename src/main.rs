use std::{env::home_dir, fs, io::{self, Write}};

use clap::Parser;
use platform_dirs::AppDirs;
use anyhow::{Context, bail};
use colored::Colorize;

use crate::key::{TrackedKey, TrackedKeys};

mod key;


#[derive(Parser)]
#[command(
    about,
    version,
    propagate_version = true,
)]
struct Cli {
    /// The partial or complete name of a tracked SSH key
    #[arg()]
    key_name: Option<String>,

    /// Exit non-zero on missing key, rather than prompt for key
    #[arg(long, short, action)]
    no_prompt: bool,

    /// Rename the selected key to the provided name
    #[arg(long, short)]
    rename: Option<String>,

    /// Delete the selected key
    #[arg(long, short, action)]
    delete: bool,
}

fn prompt_for_key(tracked_keys: &TrackedKeys) -> anyhow::Result<&TrackedKey> {
    println!("Available keys:");
    for (i, key) in tracked_keys.keys.iter().enumerate() {
        let active_mark = if Some(&key.name) == tracked_keys.active.as_ref() {
            " (active)".green()
        } else {
            "".into()
        };
        println!("  {}: {}{}", i+1, key.name, active_mark);
    }

    loop {
        let mut selection = String::new();
        let stdin = io::stdin();
        print!("Select key (number/name): ");
        io::stdout().flush()?;
        stdin.read_line(&mut selection)?;
        selection = selection.trim().into();

        let index_result = selection.parse::<usize>();
        if let Ok(index) = index_result {
            if index > 0 && let Some(some_key) = tracked_keys.keys.get(index-1) {
                return Ok(some_key)
            } else {
                println!("{}", "Selection out of range!".yellow());
            }
        } else {
            if let Some(key) = tracked_keys.find_key_by_partial(&selection) {
                return Ok(key)
            } else {
                println!("{}", "Could not find key with that name!".yellow());
            }
        }
    }
}

fn get_key(key_name: Option<String>, no_prompt: bool, tracked_keys: &TrackedKeys) -> anyhow::Result<&TrackedKey> {
    if let Some(key_name) = key_name {
        if let Some(key) = tracked_keys.find_key_by_partial(&key_name) {
            return Ok(key)
        } else {
            println!("{}", "No key found with provided name!".yellow());
        }
    }

    if no_prompt {
        bail!("No key found, and no-prompt flag is present")
    }

    prompt_for_key(tracked_keys)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.delete && cli.rename.is_some() {
        bail!("Cannot delete and rename a key at the same time!")
    }

    let app_dirs = AppDirs::new(Some("ssh-key-picker"), false)
        .context("Failed to determine data path")?;
    fs::create_dir_all(&app_dirs.data_dir)
        .context(format!("Failed to create data directory: {}", &app_dirs.data_dir.display()))?;

    let tracked_keys_path = app_dirs.data_dir.join("keys.json");
    let ssh_dir = home_dir().unwrap().join(".ssh");
    let disabled_dir = ssh_dir.join("disabled");
    fs::create_dir_all(&disabled_dir)
        .context(format!("Failed to create disabled keys directory: {}", &disabled_dir.display()))?;
    let mut tracked_keys = TrackedKeys::load(&tracked_keys_path, ssh_dir, disabled_dir)?;

    // Index any untracked keys and deactivate them as well
    if tracked_keys.find_untracked_keys(cli.no_prompt)? {
        tracked_keys.save(&tracked_keys_path)?;
    }

    // Select key to activate, or prompt user
    if tracked_keys.keys.is_empty() {
        println!("{}", "No SSH keys found.".yellow());
        return Ok(())
    }
    let key = get_key(cli.key_name, cli.no_prompt, &tracked_keys)?.clone();

    let msg: String;
    if cli.delete {
        let had_active = tracked_keys.active.is_some();
        let deleted = tracked_keys.delete(&key, cli.no_prompt)?;
        msg = if deleted {
            if had_active != tracked_keys.active.is_some() {
                format!("Deleted {} key {}", "previously active".bold(), key.name.red())
            } else {
                format!("Deleted key {}", key.name.red())
            }
        } else {
            format!("{}", "Canceled.".yellow())
        };
    } else if let Some(new_name) = cli.rename {
        let old_name = key.name.clone();
        tracked_keys.rename(&key, new_name.clone())?;
        msg = format!("Renamed key {} to {}", old_name.red(), new_name.green());
    } else {
        if let Some(active_key) = curr_active_key {
            if active_key.name == key.name {
                println!("{}", format!("Key {} is already active!", key.name.italic()).yellow());
                return Ok(())
            }
            // Deactivate current key, if one is active
            tracked_keys.deactivate_key()?;
            tracked_keys.save(&tracked_keys_path)?;
        }

        tracked_keys.activate_key(&key)?;
        msg = format!("Activated key {} as {}", key.name.green(), key.key_type.to_file_name().cyan());
    }
    tracked_keys.save(&tracked_keys_path)?;
    println!("{}", msg);

    Ok(())
}
