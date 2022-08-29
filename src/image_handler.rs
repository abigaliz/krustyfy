use cpp_core::{Ref, CppBox};
use qt_core::{QString, QFileInfo};
use qt_gui::{QPixmap, QIcon, QImage};
use qt_widgets::QFileIconProvider;

use crate::notification::ImageData;

pub unsafe fn find_icon(desktop_entry: &String) -> CppBox<QPixmap> {
    let desktop_entry_lowercase = desktop_entry.as_str().to_lowercase();

    let qstr = QString::from_std_str(desktop_entry_lowercase.as_str());
    let qdesktop_entry = qstr.as_ref();

    let icon_name : Ref<QString>;
    if QIcon::has_theme_icon(qdesktop_entry)
    {
        icon_name = qdesktop_entry;
    }
    else {
        let info = QFileInfo::new();

        let path = format!("/usr/share/applications/{}.desktop", desktop_entry);

        info.set_file_q_string( QString::from_std_str(path).as_ref());

        let icon_provider = QFileIconProvider::new();

        let icon = icon_provider.icon_q_file_info(info.as_ref());

        return icon.pixmap_int(25);
    }

    let icon = QIcon::from_theme_1a(icon_name).pixmap_int(25);

    icon
}

pub unsafe fn parse_image(image_data: ImageData) -> CppBox<QPixmap> {
    let image_format: qt_gui::q_image::Format;

    let pixmap = QPixmap::new();

    if image_data.has_alpha {
        image_format = qt_gui::q_image::Format::FormatRGBA8888;
    }
    else {
        image_format = qt_gui::q_image::Format::FormatRGB888;
    }

    let data = image_data.data.as_ptr();

    let qimage = QImage::from_uchar3_int_format2(data, image_data.width, image_data.height, image_data.rowstride, image_format);

    pixmap.convert_from_image_1a(qimage.as_ref());
    pixmap
}