use std::{env::current_dir, path::PathBuf};

use anyhow::{bail, Result};
use inquire::{
    validator::{ErrorMessage, Validation},
    Confirm, Text,
};
use libium::{
    config::{
        self,
        structs::{Config, ProfileItem},
    },
    get_minecraft_dir,
};

use crate::file_picker::pick_file;

use super::check_output_directory;

pub async fn import(
    config: &mut Config,
    name: Option<String>,
    path: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    no_gui_mode: Option<bool>,
) -> Result<()> {
    let path = if let Some(path) = path {
        path
    } else {
        println!("Where is the profile to import?");
        if let Some(path) = pick_file(
            current_dir()?,
            "Pick the profile to import",
            "Profile",
            false,
            no_gui_mode,
        )? {
            path.canonicalize()?
        } else {
            bail!("Please select a path to a profile.");
        }
    };

    if config::read_profile(&path)?.is_none() {
        bail!("No profile was found at the given path.")
    }

    let output_dir = if let Some(output_dir) = output_dir {
        output_dir
    } else {
        let mut selected_mods_dir = get_minecraft_dir().join("mods");
        println!(
            "The default mods directory is {}",
            selected_mods_dir.display()
        );
        if Confirm::new("Would you like to specify a custom mods directory?")
            .prompt()
            .unwrap_or_default()
        {
            if let Some(dir) = pick_file(
                &selected_mods_dir,
                "Pick an output directory",
                "Output Directory",
                true,
                no_gui_mode,
            )? {
                check_output_directory(&dir, no_gui_mode).await?;
                selected_mods_dir = dir;
            };
        };
        selected_mods_dir
    };

    let name = if let Some(name) = name {
        name
    } else {
        let profiles = config.profiles.clone();

        Text::new("What should this profile be called")
            .with_validator(move |s: &str| {
                Ok(
                    if profiles
                        .iter()
                        .any(|item| item.name.eq_ignore_ascii_case(s))
                    {
                        Validation::Invalid(ErrorMessage::Custom(
                            "A profile with that name already exists".to_owned(),
                        ))
                    } else {
                        Validation::Valid
                    },
                )
            })
            .prompt()?
    };

    config.profiles.push(ProfileItem {
        path,
        name,
        output_dir,
    });

    Ok(())
}
