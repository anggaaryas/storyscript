#![allow(dead_code)]
use std::{fs, path::Path};
pub fn project() -> tempfile::TempDir {
    fn copy(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }
    let temp = tempfile::tempdir().unwrap();
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/localized_project"),
        temp.path(),
    );
    temp
}
pub fn replace(root: &Path, path: &str, from: &str, to: &str) {
    let path = root.join(path);
    let source = fs::read_to_string(&path).unwrap();
    assert!(source.contains(from));
    fs::write(path, source.replace(from, to)).unwrap();
}
