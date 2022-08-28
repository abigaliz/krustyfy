use std::collections::HashMap;

use zbus::zvariant::Value;
use zvariant::Array;

pub struct Notification {
    pub app_name: String, 
    pub replaces_id: u32, 
    pub app_icon: String, 
    pub summary: String, 
    pub body: String, 
    pub actions: Vec<String>,
    pub image_data: Vec<u8>,
    pub image_has_alpha: bool,
    pub expire_timeout: i32,
}

/* impl<'a> Notification<'a> {
    pub fn new(app_name: String, replaces_id: u32, app_icon: String, summary: String, body: String, actions: Vec<String>,
        image_data: Array<'a>, image_has_alpha: bool,
        expire_timeout: i32) -> Notification<'a> {

        let _image_data = image_data.to_owned();
        Notification {
            app_name, replaces_id, app_icon, summary, body, actions, image_data: _image_data, image_has_alpha, expire_timeout
        }
    }
} */