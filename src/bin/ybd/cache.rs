use std::{fs::File, path::Path};

use yumi_bilibili_download::{
        error::{Error, Result},
        model::account::Account,
};

/// 从缓存信息中加载用户信息(包含sessdata)
///
/// * `source`: io源
pub fn load_user_from_file(source: &Path) -> Result<Account> {
        let file = File::open(source)
                .map_err(|e| Error::Normal(format!("不存在用户认证信息文件: {}", e)))?;
        let account: Account = serde_json::from_reader(file)?;
        Ok(account)
}

/// 保存认证信息
///
/// * `account`: 账户
/// * `dest`: 保存目的地
pub fn save_user_info(
        account: Account,
        dest: &Path,
) -> Result<bool> {
        if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
        }
        let file = File::create(dest)?;
        serde_json::to_writer(file, &account)?;
        Ok(true)
}
