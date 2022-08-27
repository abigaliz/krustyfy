use std::collections::HashMap;

use zbus::zvariant::Value;

pub struct Notification {
    pub app_name: String, 
    pub replaces_id: u32, 
    pub app_icon: String, 
    pub summary: String, 
    pub body: String, 
    pub actions: Vec<String>,
    //hints: HashMap<String, Value<'a>>,
    pub expire_timeout: i32,
}

impl Notification {
    pub fn new(app_name: String, replaces_id: u32, app_icon: String, summary: String, body: String, actions: Vec<String>,
        expire_timeout: i32) -> Notification {
        Notification {
            app_name, replaces_id, app_icon, summary, body, actions, expire_timeout
        }
    }
}