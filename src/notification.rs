use serde::{Serialize, Deserialize};

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct ImageData {
    pub is_empty: bool,
    pub width: i32,
    pub height: i32,
    pub rowstride: i32,
    pub has_alpha: bool,
    pub bits_per_sample: i32,
    pub channels: i32,
    pub data: Vec<u8>,
    pub desktop_entry: String,
}


impl ImageData {
    pub fn empty() -> ImageData {
        ImageData {
            is_empty: true,
            width: 0,
            height: 0,
            rowstride: 0,
            has_alpha: false,
            bits_per_sample: 0,
            channels: 0,
            data: Vec::new(),
            desktop_entry: String::new(),
        }
    }

    pub fn new(
        width: i32,
        height: i32,
        rowstride: i32,
        has_alpha: bool,
        bits_per_sample: i32,
        channels: i32,
        data: Vec<u8>,
        desktop_entry: String) -> ImageData {
        ImageData {
            is_empty: false,
            width,
            height,
            rowstride,
            has_alpha,
            bits_per_sample,
            channels,
            data,
            desktop_entry
        }
    }
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub app_name: String, 
    pub replaces_id: u32, 
    pub app_icon: String, 
    pub summary: String, 
    pub body: String, 
    pub actions: Vec<String>,
    pub image_data: ImageData,
    pub expire_timeout: i32,
    pub notification_id: u32,
}