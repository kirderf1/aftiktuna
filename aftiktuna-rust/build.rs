use std::path::Path;

fn main() {
    let data = lookup_assets();
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("builtin_assets.rs");
    write_module(std::fs::File::create(dest_path).unwrap(), &data);
    println!("cargo::rerun-if-changed=asset/");
}

#[derive(Debug, Default)]
struct AssetsData {
    text_file_list: Vec<String>,
    binary_file_list: Vec<String>,
}

fn lookup_assets() -> AssetsData {
    let mut data = AssetsData::default();
    scan_dir("./assets".as_ref(), &mut data);
    data
}

fn sanitize_path(path: &Path) -> String {
    let path = std::fs::canonicalize(path).unwrap();
    let path = path
        .strip_prefix(std::fs::canonicalize(".").unwrap())
        .unwrap();
    path.to_str().unwrap().replace('\\', "/")
}

fn create_dir_in_out(dir_path: &Path) {
    let dir_path = std::fs::canonicalize(dir_path).unwrap();
    let relative_path = dir_path
        .strip_prefix(std::fs::canonicalize(".").unwrap())
        .unwrap();
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_dir: &Path = out_dir.as_ref();
    let target_path = out_dir.join(relative_path);
    if !target_path.exists() {
        std::fs::create_dir(target_path).unwrap();
    }
}

fn copy_to_out(file_path: &Path) {
    let file_path = std::fs::canonicalize(file_path).unwrap();
    let relative_path = file_path
        .strip_prefix(std::fs::canonicalize(".").unwrap())
        .unwrap();
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_dir: &Path = out_dir.as_ref();
    let dest_path = out_dir.join(relative_path);
    std::fs::copy(file_path, dest_path).unwrap();
}

fn scan_dir(dir: &Path, data: &mut AssetsData) {
    create_dir_in_out(dir);
    let read_dir = std::fs::read_dir(dir).unwrap();
    for entry in read_dir.filter_map(Result::ok) {
        if let Ok(file_type) = entry.file_type() {
            let path = entry.path();
            if file_type.is_dir() {
                scan_dir(&path, data);
            } else if file_type.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == "json")
            {
                data.text_file_list.push(sanitize_path(&path));
                copy_to_out(&path);
            } else if file_type.is_file()
                && path.extension().is_some_and(|extension| extension == "png")
            {
                data.binary_file_list.push(sanitize_path(&path));
                copy_to_out(&path);
            }
        }
    }
}

fn write_module(file: std::fs::File, data: &AssetsData) {
    let mut writer = std::io::BufWriter::new(file);
    use std::io::Write;

    writeln!(
        writer,
        "pub fn text_data(path: &str) -> Option<&'static str> {{"
    )
    .unwrap();
    writeln!(writer, "    match path {{").unwrap();
    for text_file in &data.text_file_list {
        writeln!(
            writer,
            "        \"{text_file}\" => Some(include_str!(\"{text_file}\")),"
        )
        .unwrap();
    }
    writeln!(writer, "        _ => None,").unwrap();
    writeln!(writer, "    }}").unwrap();
    writeln!(writer, "}}").unwrap();

    writeln!(
        writer,
        "pub fn binary_data(path: &str) -> Option<&'static [u8]> {{"
    )
    .unwrap();
    writeln!(writer, "    match path {{").unwrap();
    for binary_file in &data.binary_file_list {
        writeln!(
            writer,
            "        \"{binary_file}\" => Some(include_bytes!(\"{binary_file}\")),"
        )
        .unwrap();
    }
    writeln!(writer, "        _ => None,").unwrap();
    writeln!(writer, "    }}").unwrap();
    writeln!(writer, "}}").unwrap();

    writeln!(writer, "pub fn text_files() -> &'static [&'static str] {{").unwrap();
    writeln!(writer, "    &[").unwrap();
    for text_file in &data.text_file_list {
        writeln!(writer, "        \"{text_file}\",").unwrap();
    }
    writeln!(writer, "    ]").unwrap();
    writeln!(writer, "}}").unwrap();

    writer.flush().unwrap()
}
