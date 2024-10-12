use sled::Db;
use std::error::Error;
use std::path::Path;

pub trait KvStoreOperations {
    fn get(&self, key: &str) -> Result<Option<String>, Box<dyn Error>>;
    fn set(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>>;
}

pub struct KvStore {
    db: Db,
}

impl KvStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, sled::Error> {
        Ok(Self {
            db: sled::open(path)?,
        })
    }
}

impl KvStoreOperations for KvStore {
    fn get(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        Ok(self
            .db
            .get(key)?
            .map(|ivec| String::from_utf8_lossy(&ivec).into_owned()))
    }

    fn set(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        self.db.insert(key, value.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }
}
