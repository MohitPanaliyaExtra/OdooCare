use std::path::{Path, PathBuf};
use std::fs;
use thiserror::Error;
use walkdir::WalkDir;
use anyhow::Result;
use tempfile::TempDir;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum ZipError {
    #[error("failed to read zip file: {0}")]
    ReadFailed(#[from] zip::result::ZipError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Represents one discovered Odoo module inside a zip.
#[derive(Clone, Debug)]
pub struct FoundModule {
    pub name: String,
    pub manifest_path: PathBuf,
    pub dir_path: PathBuf,
}

/// Extracts a zip file into a temporary directory and discovers all
/// `__manifest__.py` files inside it.
///
/// Returns `(Vec<FoundModule>, TempDir)` — the TempDir **must** be kept alive
/// by the caller, otherwise the extracted files are deleted and all paths become invalid.
pub fn extract_and_discover(zip_path: &Path) -> Result<(Vec<FoundModule>, TempDir), anyhow::Error> {
    let tmp = tempfile::tempdir()?;
    let file = fs::File::open(zip_path)?;
    let mut zip_archive = zip::ZipArchive::new(file)?;

    for i in 0..zip_archive.len() {
        let mut entry = zip_archive.by_index(i)?;
        let entry_path = entry.mangled_name();
        let target = tmp.path().join(&entry_path);

        if entry_path.to_string_lossy().contains('*') {
            continue;
        }

        if entry.is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out = fs::File::create(&target)?;
            std::io::copy(&mut entry, &mut out)?;
        }
    }
    drop(zip_archive);

    let mut modules = Vec::new();

    for entry in WalkDir::new(tmp.path()).into_iter().filter_map(|e| e.ok()) {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name == "__manifest__.py" {
            let manifest_path = entry.path().to_path_buf();
            let dir_path = manifest_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("__manifest__.py has no parent directory"))?
                .to_path_buf();

            let name = dir_path
                .file_stem()
                .or_else(|| dir_path.file_name())
                .ok_or_else(|| anyhow::anyhow!("cannot derive module name from {:?}", dir_path))?
                .to_string_lossy()
                .to_string();

            modules.push(FoundModule {
                name,
                manifest_path: manifest_path.clone(),
                dir_path,
            });
        }
    }

    modules.sort_by(|a, b| a.name.cmp(&b.name));
    Ok((modules, tmp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_extract_single_module_zip() {
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("my_module");
        fs::create_dir_all(&folder.join("models")).unwrap();
        fs::write(
            folder.join("__manifest__.py"),
            r#"{"name": "My Module", "depends": ["base"]}"#,
        )
        .unwrap();
        let zip_path = tmp.path().join("test.zip");

        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default();
        zip.start_file("my_module/__manifest__.py", options).unwrap();
        let manifest = br#"{"name": "My Module", "depends": ["base"]}"#;
        zip.write_all(manifest).unwrap();
        zip.finish().unwrap();

        let (found, _kept_alive) = extract_and_discover(&zip_path).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "my_module");
    }
}
