use core::slice;

use cpp_core::CppBox;
use qt_core::{QVariant, QHashOfQStringQVariant, QByteArray, qs};

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
        data: Vec<u8>,) -> ImageData {
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

    pub unsafe fn from_qvariant(hash : &CppBox<QHashOfQStringQVariant>) -> Self {
        let width = hash.value_1a(&qs("width")).to_int_0a();
        let height = hash.value_1a(&qs("height")).to_int_0a();
        let rowstride = hash.value_1a(&qs("rowstride")).to_int_0a();
        let has_alpha = hash.value_1a(&qs("has_alpha")).to_bool();
        let bits_per_sample = hash.value_1a(&qs("bits_per_sample")).to_int_0a();
        let channels = hash.value_1a(&qs("channels")).to_int_0a();
        let data_slice_i8 = hash.value_1a(&qs("data")).to_byte_array().as_slice();

        let data_slice = slice::from_raw_parts(data_slice_i8.as_ptr() as *const u8, data_slice_i8.len());

        let data = data_slice.to_vec();


        ImageData::new(width, height, rowstride, has_alpha, bits_per_sample, channels, data)
    }

    pub unsafe fn to_qvariant(&self) -> CppBox<QHashOfQStringQVariant> {
        let hash = QHashOfQStringQVariant::new();

        let width = QVariant::from_int(self.width);
        let height = QVariant::from_int(self.height);
        let rowstride = QVariant::from_int(self.rowstride);
        let has_alpha = QVariant::from_bool(self.has_alpha);
        let bits_per_sample = QVariant::from_int(self.bits_per_sample);
        let channels = QVariant::from_int(self.channels);
        let data = QVariant::from_q_byte_array(&QByteArray::from_slice(self.data.as_slice()));

        hash.insert(&qs("width"), &width);
        hash.insert(&qs("height"), &height);
        hash.insert(&qs("rowstride"), &rowstride);
        hash.insert(&qs("has_alpha"), &has_alpha);
        hash.insert(&qs("bits_per_sample"), &bits_per_sample);
        hash.insert(&qs("channels"), &channels);
        hash.insert(&qs("data"), &data);

        hash
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

impl Notification {
    pub unsafe fn from_qvariant(hash : &CppBox<QHashOfQStringQVariant>) -> Self {
        let app_name = hash.value_1a(&qs("app_name")).to_string().to_std_string();
        let replaces_id = hash.value_1a(&qs("replaces_id")).to_u_int_0a();
        let app_icon = hash.value_1a(&qs("app_icon")).to_string().to_std_string();
        let summary = hash.value_1a(&qs("summary")).to_string().to_std_string();
        let body = hash.value_1a(&qs("body")).to_string().to_std_string();
        let expire_timeout = hash.value_1a(&qs("expire_timeout")).to_int_0a();
        let notification_id = hash.value_1a(&qs("notification_id")).to_u_int_0a();
        let actions = Vec::new();

        let desktop_entry = hash.value_1a(&qs("desktop_entry")).to_string().to_std_string();

        let mut image_data: Option<ImageData> = None;

        if hash.contains(&qs("image_data")) {
            let image_data_hash = hash.value_1a(&qs("image_data")).to_hash();
            image_data = Some(ImageData::from_qvariant(&image_data_hash));
        }

        let mut image_path: Option<String> = None;

        if hash.contains(&qs("image_path")) {
            image_path = Some(hash.value_1a(&qs("image_path")).to_string().to_std_string());
        }

        Notification {
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            image_data,
            image_path,
            expire_timeout,
            notification_id,
            desktop_entry
        }
    }

    pub unsafe fn to_qvariant(&self) -> CppBox<QVariant> {
        let hash = QHashOfQStringQVariant::new();

        let app_name = QVariant::from_q_string(&qs(&self.app_name));
        let replaces_id = QVariant::from_uint(self.replaces_id);
        let app_icon = QVariant::from_q_string(&qs(&self.app_icon));
        let summary = QVariant::from_q_string(&qs(&self.summary));
        let body = QVariant::from_q_string(&qs(&self.body));
        let expire_timeout = QVariant::from_int(self.expire_timeout);
        let notification_id = QVariant::from_uint(self.notification_id);
        let desktop_entry = QVariant::from_q_string(&qs(&self.desktop_entry));

        hash.insert(&qs("app_name"), &app_name);
        hash.insert(&qs("replaces_id"), &replaces_id);
        hash.insert(&qs("app_icon"), &app_icon);
        hash.insert(&qs("summary"), &summary);
        hash.insert(&qs("body"), &body);
        hash.insert(&qs("expire_timeout"), &expire_timeout);
        hash.insert(&qs("notification_id"), &notification_id);
        hash.insert(&qs("desktop_entry"), &desktop_entry);

        if self.image_data.is_some() {
            let image_data = QVariant::from_q_hash_of_q_string_q_variant(&self.image_data.as_ref().unwrap().to_qvariant());
            hash.insert(&qs("image_data"), &image_data);
        }

        if self.image_path.is_some() {
            let image_path = QVariant::from_q_string(&qs(&self.image_path.as_ref().unwrap()));
            hash.insert(&qs("image_path"), &image_path);
        }

        QVariant::from_q_hash_of_q_string_q_variant(&hash)
    }
}