//! 用户信息模块

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    user_id: String,
    exp: SystemTime,
    sessdata: String,
}

impl Account {
    pub(crate) fn new(user_id: String, exp: SystemTime, sessdata: String) -> Self {
        Self {
            user_id,
            exp,
            sessdata,
        }
    }
    pub fn get_sessdata(&self) -> &str {
        &self.sessdata
    }
    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.exp
    }
}
