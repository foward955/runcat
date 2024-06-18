use std::path::PathBuf;

use tray_icon::Icon;

use crate::err::RunCatTrayError;

pub fn load_icon(path: PathBuf) -> Result<Icon, RunCatTrayError> {
    let image = image::open(path.clone())
        .map_err(|e| {
            let err = format!("Failed to open icon path: {:?}, error: {:?}", path, e);
            RunCatTrayError::FileError(err)
        })?
        .into_rgba8();

    let (icon_width, icon_height) = image.dimensions();
    let icon_rgba = image.into_raw();

    Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .map_err(|_| RunCatTrayError::FileError("Failed to open icon".to_string()))
}

pub fn current_exe_dir() -> Result<PathBuf, RunCatTrayError> {
    let exe = std::env::current_exe()
        .map_err(|_| RunCatTrayError::PathError("Get current exe path failed.".to_string()))?;

    if let Some(dir) = exe.parent() {
        Ok(dir.to_path_buf())
    } else {
        Err(RunCatTrayError::PathError(
            "Get current exe dir failed.".to_string(),
        ))
    }
}
