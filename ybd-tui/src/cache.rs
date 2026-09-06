//! 认证凭据缓存模块
//!
//! 负责从磁盘加载与保存 Bilibili 登录凭据（SESSDATA 等）。

use std::{fs::File, path::Path};

use ybd_core::{
        error::{Error, Result},
        model::account::Account,
};

pub fn load_user_from_file(source: &Path) -> Result<Account> {
        let file = File::open(source)
                .map_err(|e| Error::Normal(format!("不存在用户认证信息文件: {}", e)))?;
        let account: Account = serde_json::from_reader(file)?;
        Ok(account)
}

pub fn save_user_info(
        account: &Account,
        dest: &Path,
) -> Result<bool> {
        if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
        }
        let file = File::create(dest)?;
        serde_json::to_writer(file, account)?;
        Ok(true)
}

pub fn remove_user_info(dest: &Path) -> Result<()> {
        if dest.exists() {
                std::fs::remove_file(dest)?;
        }
        Ok(())
}
