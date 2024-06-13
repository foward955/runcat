pub fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let image = image::open(path)
        .expect("Failed to open icon path")
        .into_rgba8();

    let (icon_width, icon_height) = image.dimensions();
    let icon_rgba = image.into_raw();

    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
