mod backgrounds_editor;
mod location_editor;
mod model_editor;
mod species_color_editor;

fn main() {
    let Some(path) = editor_file() else {
        return;
    };

    if aftiktuna::asset::background::DATA_FILE.matches(&path) {
        backgrounds_editor::run()
    } else if path_starts_with(&path, "assets/texture/object/") {
        model_editor::run(path)
    } else if path_starts_with(&path, "assets/location/") {
        location_editor::run(path)
    } else if path_starts_with(&path, "assets/species_color/") {
        species_color_editor::run(path)
    } else {
        eprintln!("Unknown asset file kind: {path:?}")
    }
}

fn path_starts_with(path: &std::path::Path, pattern: &str) -> bool {
    std::fs::canonicalize(path)
        .unwrap()
        .starts_with(std::fs::canonicalize(pattern).unwrap())
}

fn editor_file() -> Option<std::path::PathBuf> {
    if let [_, file, ..] = &std::env::args().collect::<Vec<_>>()[..] {
        Some(file.into())
    } else {
        let assets_directory = std::fs::canonicalize("./assets/").unwrap();
        rfd::FileDialog::new()
            .set_title("Pick an asset file")
            .add_filter("JSON", &["json"])
            .set_directory(assets_directory)
            .pick_file()
    }
}
