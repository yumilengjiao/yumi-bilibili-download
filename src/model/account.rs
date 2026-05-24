use std::time::SystemTime;

#[derive(Debug)]
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
