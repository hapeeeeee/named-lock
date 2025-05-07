use wasm_bindgen::prelude::*;
use web_sys::{window, Storage};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::Error;

const LOCK_PREFIX: &str = "named_lock_";
const LOCK_TIMEOUT: u64 = 30000; // 30 seconds timeout

pub(crate) struct NamedLock {
    name: String,
    storage: Storage,
}

impl NamedLock {
    pub(crate) fn new(name: &str) -> Result<Self, Error> {
        let window = window().ok_or_else(|| Error::Other("无法获取window对象".into()))?;
        let storage = window
            .local_storage()
            .map_err(|_| Error::Other("无法访问localStorage".into()))?
            .ok_or_else(|| Error::Other("localStorage不可用".into()))?;

        Ok(Self {
            name: format!("{}{}", LOCK_PREFIX, name),
            storage,
        })
    }

    pub(crate) fn try_lock(&self) -> Result<bool, Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // 检查现有锁
        if let Some(lock_time_str) = self.storage.get_item(&self.name).map_err(|_| Error::Other("无法读取锁状态".into()))? {
            if let Ok(lock_time) = lock_time_str.parse::<u64>() {
                // 如果锁未过期，返回false
                if now - lock_time < LOCK_TIMEOUT {
                    return Ok(false);
                }
            }
        }

        // 尝试获取锁
        self.storage
            .set_item(&self.name, &now.to_string())
            .map_err(|_| Error::Other("无法设置锁".into()))?;

        // 验证是否成功获取锁（处理并发情况）
        if let Ok(Some(stored_time_str)) = self.storage.get_item(&self.name) {
            if let Ok(stored_time) = stored_time_str.parse::<u64>() {
                if stored_time == now {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub(crate) fn unlock(&self) -> Result<(), Error> {
        self.storage
            .remove_item(&self.name)
            .map_err(|_| Error::Other("无法释放锁".into()))?;
        Ok(())
    }
}

impl Drop for NamedLock {
    fn drop(&mut self) {
        // 尝试清理锁，忽略可能的错误
        let _ = self.unlock();
    }
}