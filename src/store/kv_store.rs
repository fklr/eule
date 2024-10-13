use crate::Result;
use sled::Db;
use std::path::Path;
use std::sync::Arc;

pub struct KvStore {
    db: Arc<Db>,
}

impl KvStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            db: Arc::new(sled::open(path)?),
        })
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .db
            .get(key)?
            .map(|ivec| String::from_utf8_lossy(&ivec).into_owned()))
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        self.db.insert(key, value.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        self.db.remove(key)?;
        self.db.flush()?;
        Ok(())
    }
}
