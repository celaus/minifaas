use std::path::PathBuf;
use uuid::Uuid;

const TMPDIR_NAME: &str = "minifaas-test";

///
/// Creates an empty directory (using a UUID) in the current user's temp directory.
/// Guaranteed to be empty.
///
pub fn get_empty_tmp_dir() -> PathBuf {
    let d = std::env::temp_dir()
        .join(TMPDIR_NAME)
        .join(Uuid::new_v4().to_string());
    let _ = std::fs::remove_dir_all(&d);
    let _ = std::fs::create_dir_all(&d);
    d
}
