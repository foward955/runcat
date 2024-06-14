use std::path::PathBuf;

use tray_icon::Icon;

use crate::err::RunCatTrayError;

pub fn load_icon(path: std::path::PathBuf) -> Result<Icon, RunCatTrayError> {
    let image = image::open(path)
        .map_err(|_| RunCatTrayError::FileError("Failed to open icon path"))?
        .into_rgba8();

    let (icon_width, icon_height) = image.dimensions();
    let icon_rgba = image.into_raw();

    Ok(Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .map_err(|_| RunCatTrayError::FileError("Failed to open icon"))?)
}

pub fn current_exe_dir() -> Result<PathBuf, RunCatTrayError> {
    let exe = std::env::current_exe()
        .map_err(|e| RunCatTrayError::PathError("Get current exe path failed."))?;

    if let Some(dir) = exe.parent() {
        Ok(dir.to_path_buf())
    } else {
        Err(RunCatTrayError::PathError("Get current exe dir failed."))
    }
}
