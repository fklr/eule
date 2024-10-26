use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Creates a unique test directory path.
///
/// Uses an atomic counter to ensure each test gets a unique path,
/// even when tests are run concurrently.
pub fn unique_test_path() -> PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    PathBuf::from(format!("test_db_{}", id))
}

/// A guard struct that cleans up test directories on drop.
pub struct TestCleanup {
    path: PathBuf,
}

impl TestCleanup {
    /// Creates a new TestCleanup instance for the given path.
    ///
    /// Also ensures the directory exists and is empty.
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TestCleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
