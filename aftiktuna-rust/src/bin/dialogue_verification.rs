use aftiktuna::asset::dialogue::DIALOGUE_DIR;

fn main() {
    let failure_count = verify_files_in_dir(DIALOGUE_DIR.dir_path().as_ref());

    if failure_count == 0 {
        println!("All dialogue files are OK!");
    }
}

fn verify_files_in_dir(path: &std::path::Path) -> u32 {
    let mut failure_count = 0;
    let read_dir = std::fs::read_dir(path).unwrap();
    for entry in read_dir.filter_map(Result::ok) {
        if let Ok(file_type) = entry.file_type() {
            let path = entry.path();
            if file_type.is_dir() {
                failure_count += verify_files_in_dir(&path);
            } else if file_type.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == "json")
            {
                let mut path = std::fs::canonicalize(path).unwrap();
                path.set_extension("");
                let path = path
                    .strip_prefix(std::fs::canonicalize(DIALOGUE_DIR.dir_path()).unwrap())
                    .unwrap();
                let dialogue_name = path.to_str().unwrap();

                if let Err(error) = DIALOGUE_DIR.load(dialogue_name) {
                    eprintln!("Failed to load dialogue \"{dialogue_name}\":");
                    eprintln!("{error}");
                    failure_count += 1;
                }
            }
        }
    }
    failure_count
}
