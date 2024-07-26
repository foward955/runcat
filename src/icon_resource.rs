use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use tray_icon::Icon;

use crate::{
    err::RunCatTrayError,
    util::{current_exe_dir, load_icon},
};

pub(crate) const MAX_RUN_ICON_INDEX: usize = 4;

#[derive(Clone)]
pub struct IconResource {
    pub dark: Vec<Icon>,
    pub light: Vec<Icon>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IconResourcePath {
    pub dark: Vec<String>,
    pub light: Vec<String>,
}

impl IconResourcePath {
    pub(crate) fn load(
        path: PathBuf,
    ) -> Result<HashMap<String, IconResourcePath>, RunCatTrayError> {
        if let Some(path) = path.to_str() {
            let config = config::Config::builder()
                .add_source(config::File::with_name(path))
                .build()
                .map_err(|_| {
                    RunCatTrayError::FileError(
                        "File \"resource.toml\" is not found/invalid toml file. Please check."
                            .to_string(),
                    )
                })?;

            Ok(config
                .get::<HashMap<String, IconResourcePath>>("resource")
                .map_err(|_| {
                    RunCatTrayError::FileError(
                        "Invalid resource file. Please check it out.".to_string(),
                    )
                })?)
        } else {
            Err(RunCatTrayError::PathError(
                "Can't load resource.".to_string(),
            ))
        }
    }
}

impl IconResource {
    pub fn load(
        light_paths: &[String],
        dark_paths: &[String],
    ) -> Result<IconResource, RunCatTrayError> {
        let base = current_exe_dir()?;

        let mut light_icon = vec![];
        let mut dark_icon = vec![];

        if light_paths.len() != dark_paths.len() && light_paths.len() != MAX_RUN_ICON_INDEX {
            return Err(RunCatTrayError::Other(format!(
                "light/dark icon must greater than or equal to {}.",
                MAX_RUN_ICON_INDEX
            )));
        }

        for p in light_paths {
            let icon = load_icon(base.join(p))?;
            light_icon.push(icon);
        }

        for p in dark_paths {
            let icon = load_icon(base.join(p))?;
            dark_icon.push(icon);
        }

        Ok(IconResource {
            light: light_icon,
            dark: dark_icon,
        })
    }
}
