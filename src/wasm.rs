use web_sys::{window, Storage};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::*;

const LOCK_PREFIX: &str = "named_lock_";
const LOCK_TIMEOUT: u64 = 30000; // 30 seconds timeout

#[derive(Debug)]
pub(crate) struct RawNamedLock {
    name: String,
    storage: Storage,
}

unsafe impl Send for RawNamedLock {}
unsafe impl Sync for RawNamedLock {}

impl RawNamedLock {
    pub(crate) fn create(name: &String) -> Result<RawNamedLock> {
        let window = window()
            .ok_or(Error::CreateFailed)?;
        
        let storage = window
            .local_storage()
            .map_err(|_| Error::LockFailed)?
            .ok_or(Error::LockFailed)?;

        Ok(RawNamedLock {
            name: format!("{}{}", LOCK_PREFIX, name),
            storage,
        })
    }

    pub(crate) fn try_lock(&self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // 检查现有锁
        if let Some(lock_time_str) = self.storage
            .get_item(&self.name)
            .map_err(|_| Error::LockFailed)? 
        {
            if let Ok(lock_time) = lock_time_str.parse::<u64>() {
                // 如果锁未过期，返回WouldBlock错误
                if now - lock_time < LOCK_TIMEOUT {
                    return Err(Error::WouldBlock);
                }
            }
        }

        // 尝试获取锁
        self.storage
            .set_item(&self.name, &now.to_string())
            .map_err(|_| Error::LockFailed)?;

        // 验证是否成功获取锁（处理并发情况）
        if let Ok(Some(stored_time_str)) = self.storage.get_item(&self.name) {
            if let Ok(stored_time) = stored_time_str.parse::<u64>() {
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
                    // 短暂等待后重试
                    std::thread::yield_now();
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub(crate) fn unlock(&self) -> Result<()> {
        self.storage
            .remove_item(&self.name)
            .map_err(|_| Error::UnlockFailed)?;
        Ok(())
    }
}

impl Drop for RawNamedLock {
    fn drop(&mut self) {
        let _ = self.unlock();
    }
}