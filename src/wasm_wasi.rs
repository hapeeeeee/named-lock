use crate::error::*;
use std::fs::{remove_file, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const LOCK_PREFIX: &str = "named_lock_";
const LOCK_TIMEOUT: u64 = 30000; // 30 seconds timeout

#[derive(Debug)]
pub(crate) struct RawNamedLock {
    name: String,
    path: PathBuf,
}

unsafe impl Send for RawNamedLock {}
unsafe impl Sync for RawNamedLock {}

impl RawNamedLock {
    pub(crate) fn create(name: &String) -> Result<RawNamedLock> {
        let mut path = PathBuf::from("/tmp"); // WASI 通常支持 /tmp
        path.push(format!("{}{}", LOCK_PREFIX, name));

        Ok(RawNamedLock {
            name: name.clone(),
            path,
        })
    }

    pub(crate) fn try_lock(&self) -> Result<()> {
        let now =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
                as u64;

        // 如果文件存在，检查时间戳
        if let Ok(content) = std::fs::read_to_string(&self.path) {
            if let Ok(lock_time) = content.parse::<u64>() {
                if now - lock_time < LOCK_TIMEOUT {
                    return Err(Error::WouldBlock);
                }
            }
        }

        // 尝试写入锁文件
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|_| Error::LockFailed)?;

        writeln!(file, "{}", now).map_err(|_| Error::LockFailed)?;

        // 验证写入是否生效
        if let Ok(stored_time_str) = std::fs::read_to_string(&self.path) {
            if let Ok(stored_time) = stored_time_str.trim().parse::<u64>() {
                if stored_time == now {
                    return Ok(());
                }
            }
        }

        Err(Error::LockFailed)
    }

    pub(crate) fn lock(&self) -> Result<()> {
        loop {
            match self.try_lock() {
                Ok(()) => return Ok(()),
                Err(Error::WouldBlock) => {
                    std::thread::yield_now();
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub(crate) fn unlock(&self) -> Result<()> {
        if self.path.exists() {
            remove_file(&self.path).map_err(|_| Error::UnlockFailed)?;
        }
        Ok(())
    }
}

impl Drop for RawNamedLock {
    fn drop(&mut self) {
        let _ = self.unlock();
    }
}
