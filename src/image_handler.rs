use std::collections::HashMap;

use cpp_core::{Ref, CppBox};
use qt_core::{QString, qs, QFileInfo, QFile};
use qt_gui::{QPixmap, QIcon};
use qt_widgets::QFileIconProvider;
use zbus::zvariant::Value;
use zbus::zvariant::{EncodingContext as Context, from_slice, to_bytes, Type};
use byteorder::LE;



pub unsafe fn find_icon(app_name: &String, hints: &HashMap<String, Value<'_>>) -> CppBox<QPixmap> {
    let app_name_lower = QString::from_std_str(app_name.to_lowercase()).as_ref();

    let mut icon_name: Ref<QString> =  QString::from_std_str(app_name.to_lowercase()).as_ref();

    let ctxt = Context::<LE>::new_dbus(0);

    if (hints.contains_key("desktop-entry")) {

        let decoded = zbus::zvariant::Str::try_from(&hints["desktop-entry"]).ok().unwrap().to_string();

        let desktop_entry = QString::from_std_str(decoded.to_lowercase()).as_ref();

        if QIcon::has_theme_icon(desktop_entry)
        {
            icon_name = desktop_entry;
        }
        else {
            let info = QFileInfo::new();

            let path = format!("/usr/share/applications/{}.desktop", decoded);

            info.set_file_q_string( QString::from_std_str(path).as_ref());

            let icon_provider = QFileIconProvider::new();

            let icon = icon_provider.icon_q_file_info(info.as_ref());

            return icon.pixmap_int(25);
        }
    }
    else {
        if QIcon::has_theme_icon(app_name_lower){
            icon_name = app_name_lower;
        }
    }

    let icon = QIcon::from_theme_1a(icon_name).pixmap_int(25);

    icon
}
/* 

def parseImage(metadata):
    image = None

    convertFrom = None

    if 'icon_data' in metadata:
        convertFrom = 'icon_data'

    if 'image-data' in metadata:
        convertFrom = 'image-data'

    if convertFrom is not None:
        pixmapArray = metadata[convertFrom][6]
        pixmap = QPixmap()

        imageFormat = QImage.Format.Format_RGBA8888 if metadata[convertFrom][3] else QImage.Format.Format_RGB888

        pixmap.convertFromImage(QImage(bytes(pixmapArray), metadata[convertFrom][0], metadata[convertFrom][1],
                                       metadata[convertFrom][2], imageFormat))
        image = pixmap.scaledToWidth(imageSize, Qt.TransformationMode.SmoothTransformation)

    if 'image-path' in metadata:
        pixmap = QPixmap()

        pixmap.convertFromImage(QImage(metadata['image-path']))
        image = pixmap.scaledToWidth(80, Qt.TransformationMode.SmoothTransformation)

    return image */