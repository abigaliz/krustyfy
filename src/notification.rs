#[derive(Debug)]
pub struct ImageData {
    pub width: i32,
    pub height: i32,
    pub rowstride: i32,
    pub has_alpha: bool,
    pub bits_per_sample: i32,
    pub channels: i32,
    pub data: Vec<u8>,
}

impl ImageData {
    pub fn new(
        width: i32,
        height: i32,
        rowstride: i32,
        has_alpha: bool,
        bits_per_sample: i32,
        channels: i32,
        data: Vec<u8>,
    ) -> ImageData {
        ImageData {
            width,
            height,
            rowstride,
            has_alpha,
            bits_per_sample,
            channels,
            data,
        }
    }
}

#[derive(Debug)]
pub struct Notification {
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub image_data: Option<ImageData>,
    pub image_path: Option<String>,
    pub expire_timeout: i32,
    pub notification_id: u32,
    pub desktop_entry: String,
}
