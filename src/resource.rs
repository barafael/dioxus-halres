use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Resource {
    pub(crate) uri_uuid: String,
    pub(crate) url: String,
    pub(crate) scheme: String,
    pub(crate) host: String,
    pub(crate) path: String,
    pub(crate) live_status: String,
    pub(crate) title: String,
    pub(crate) auto_descr: String,
    pub(crate) man_descr: String,
    pub(crate) crea_user: String,
    pub(crate) crea_time: String,
    pub(crate) modi_user: String,
    pub(crate) modi_time: String,
}

impl Default for Resource {
    fn default() -> Self {
        Self {
            uri_uuid: String::new(),
            url: "-".to_string(),
            scheme: "-".to_string(),
            host: "-".to_string(),
            path: "-".to_string(),
            live_status: "1".to_string(),
            title: "-".to_string(),
            auto_descr: "-".to_string(),
            man_descr: String::new(),
            crea_time: String::new(),
            crea_user: "api".to_string(),
            modi_time: String::new(),
            modi_user: "api".to_string(),
        }
    }
}
