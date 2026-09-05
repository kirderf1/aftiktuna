use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

trait AssetSourceImpl {
    fn load_from_path<V: DeserializeOwned>(name: &str) -> Result<V, super::Error>;
    fn load_from_dir_path<K, V>(dir: &str) -> Result<HashMap<K, V>, super::Error>
    where
        K: Eq + Hash + for<'a> From<&'a str>,
        V: DeserializeOwned;
}

#[cfg(not(target_arch = "wasm32"))]
struct FsAssetSource;

#[cfg(not(target_arch = "wasm32"))]
impl AssetSourceImpl for FsAssetSource {
    fn load_from_path<V: DeserializeOwned>(name: &str) -> Result<V, super::Error> {
        let file = std::fs::File::open(name)
            .map_err(|error| super::Error::IO(PathBuf::from(name), error))?;
        serde_json::from_reader(file)
            .map_err(|error| super::Error::Json(PathBuf::from(name), error))
    }

    fn load_from_dir_path<K, V>(dir: &str) -> Result<HashMap<K, V>, super::Error>
    where
        K: Eq + Hash + for<'a> From<&'a str>,
        V: DeserializeOwned,
    {
        let mut map = HashMap::new();
        for entry in
            std::fs::read_dir(dir).map_err(|error| super::Error::IO(PathBuf::from(dir), error))?
        {
            if let Ok(entry) = entry
                && let Ok(file_name) = entry.file_name().into_string()
                && let [file_name, "json"] = file_name.split('.').collect::<Vec<_>>()[..]
            {
                let file = std::fs::File::open(entry.path())
                    .map_err(|error| super::Error::IO(entry.path(), error))?;
                let asset_data: V = serde_json::from_reader(file)
                    .map_err(|error| super::Error::Json(entry.path(), error))?;
                map.insert(K::from(file_name), asset_data);
            }
        }
        Ok(map)
    }
}

#[cfg(target_arch = "wasm32")]
struct BuiltinAssetSource;

#[cfg(target_arch = "wasm32")]
impl AssetSourceImpl for BuiltinAssetSource {
    fn load_from_path<V: DeserializeOwned>(name: &str) -> Result<V, super::Error> {
        let Some(data) = crate::builtin_assets::text_data(name) else {
            return Err(super::Error::IO(
                PathBuf::from(name),
                std::io::ErrorKind::NotFound.into(),
            ));
        };
        serde_json::from_str(data).map_err(|error| super::Error::Json(PathBuf::from(name), error))
    }

    fn load_from_dir_path<K, V>(dir: &str) -> Result<HashMap<K, V>, super::Error>
    where
        K: Eq + Hash + for<'a> From<&'a str>,
        V: DeserializeOwned,
    {
        let mut map = HashMap::new();
        for file in crate::builtin_assets::text_files() {
            if file.starts_with(dir) {
                let stripped_file = &file[(dir.len() + 1)..];
                let dot_pos = stripped_file.find('.').unwrap();
                let name = &stripped_file[..dot_pos];

                let asset_data = Self::load_from_path::<V>(file)?;
                map.insert(K::from(name), asset_data);
            }
        }
        Ok(map)
    }
}

#[cfg(not(target_arch = "wasm32"))]
type AssetSource = FsAssetSource;
#[cfg(target_arch = "wasm32")]
type AssetSource = BuiltinAssetSource;

pub struct AssetFile<T> {
    path: &'static str,
    data_type: PhantomData<T>,
}

impl<T: DeserializeOwned> AssetFile<T> {
    pub(crate) const fn new(path: &'static str) -> Self {
        Self {
            path,
            data_type: PhantomData,
        }
    }
    pub fn matches(&self, path: &Path) -> bool {
        path.ends_with(self.path)
    }
    pub fn file_path(&self) -> PathBuf {
        format!("assets/{}", self.path).into()
    }
    pub fn load(&self) -> Result<T, super::Error> {
        AssetSource::load_from_path::<T>(&format!("assets/{}", self.path))
    }
}

impl<K: Eq + Hash + DeserializeOwned, V: DeserializeOwned> AssetFile<HashMap<K, V>> {
    /// Loads asset as order-preserved map.
    pub fn load_index_map(&self) -> Result<IndexMap<K, V>, super::Error> {
        AssetSource::load_from_path::<IndexMap<K, V>>(&format!("assets/{}", self.path))
    }
}

pub struct AssetDirectory<T> {
    path: &'static str,
    data_type: PhantomData<T>,
}

impl<T: DeserializeOwned> AssetDirectory<T> {
    pub(crate) const fn new(path: &'static str) -> Self {
        Self {
            path,
            data_type: PhantomData,
        }
    }
    pub fn dir_path(&self) -> PathBuf {
        format!("assets/{}", self.path).into()
    }
    pub fn file_path(&self, id: impl Display) -> PathBuf {
        format!("assets/{}/{id}.json", self.path).into()
    }
    pub fn load(&self, id: impl Display) -> Result<T, super::Error> {
        AssetSource::load_from_path::<T>(&format!("assets/{}/{id}.json", self.path))
    }
    pub fn load_all<K>(&self) -> Result<HashMap<K, T>, super::Error>
    where
        K: Eq + Hash + for<'a> From<&'a str>,
    {
        AssetSource::load_from_dir_path(&format!("assets/{}", self.path))
    }
}

impl<K: Eq + Hash + DeserializeOwned, V: DeserializeOwned> AssetDirectory<HashMap<K, V>> {
    /// Loads asset as order-preserved map.
    pub fn load_index_map(&self, id: impl Display) -> Result<IndexMap<K, V>, super::Error> {
        AssetSource::load_from_path::<IndexMap<K, V>>(&format!("assets/{}/{id}.json", self.path))
    }
}
