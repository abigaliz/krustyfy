use cpp_core::{CppBox, Ref};

use qt_core::{qs, QFileInfo, QString};
use qt_gui::{QIcon, QImage, QPixmap};
use qt_widgets::QFileIconProvider;

use crate::notification::ImageData;

const DEFAULT_ICON: &str = "notifications";

pub unsafe fn find_icon(desktop_entry: &String) -> CppBox<QPixmap> {
    let desktop_entry_lowercase = desktop_entry.as_str().to_lowercase();

    let qstr = QString::from_std_str(desktop_entry_lowercase.as_str());
    let qdesktop_entry = qstr.as_ref();

    let icon_name: Ref<QString>;
    if QIcon::has_theme_icon(qdesktop_entry) {
        icon_name = qdesktop_entry;
    } else {
        let info = QFileInfo::new();

        let path = format!("/usr/share/applications/{}.desktop", desktop_entry);

        info.set_file_q_string(QString::from_std_str(path).as_ref());

        let icon_provider = QFileIconProvider::new();

        if info.exists_0a() {
            let icon = icon_provider.icon_q_file_info(info.as_ref());

            let pixmap = icon.pixmap_int(64);

            return pixmap;
        }

        return QIcon::from_theme_1a(QString::from_std_str(DEFAULT_ICON).as_ref()).pixmap_int(64);
    }

    QIcon::from_theme_1a(icon_name).pixmap_int(64)
}

pub unsafe fn parse_image(image_data: ImageData) -> CppBox<QPixmap> {
    let pixmap = QPixmap::new();

    let image_format = if image_data.has_alpha {
        qt_gui::q_image::Format::FormatRGBA8888
    } else {
        qt_gui::q_image::Format::FormatRGB888
    };

    let data = image_data.data.as_ptr();

    let qimage = QImage::from_uchar3_int_format2(
        data,
        image_data.width,
        image_data.height,
        image_data.rowstride,
        image_format,
    );

    pixmap.convert_from_image_1a(qimage.as_ref());

    pixmap
}

pub unsafe fn load_image(image_path: String) -> CppBox<QPixmap> {
    let pixmap = QPixmap::new();

    let qimage = QImage::from_q_string(&qs(image_path));

    pixmap.convert_from_image_1a(qimage.as_ref());

    pixmap
}
