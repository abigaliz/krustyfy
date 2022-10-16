use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::time::Duration;

use errors::KrustifyError;
use qt_core::{
    qs, ConnectionType, QCoreApplication, QString, SignalOfInt, SignalOfQString, WidgetAttribute,
    WindowType,
};
use qt_widgets::{QApplication, QFrame, QMainWindow};
use tokio::{
    self,
    sync::mpsc::{self, Sender},
};
use uuid::Uuid;
use zbus::fdo::Error as ZbusFdoError;
use zbus::{dbus_interface, zvariant::Array, ConnectionBuilder};
use zvariant::Value;

use notification::{ImageData, Notification};
use notification_spawner::NotificationSpawner;

use crate::dbus_signal::{DbusMethod, DbusSignal};
use crate::settings::{load_settings, SETTINGS};
use crate::tray_menu::generate_tray;

mod dbus_signal;
mod errors;
mod image_handler;
mod notification;
mod notification_spawner;
mod notification_widget;
mod settings;
mod tray_menu;

//static
struct NotificationHandler {
    count: u32,
    dbus_method_sender: Sender<DbusMethod>,
}

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl NotificationHandler {
    #[dbus_interface(name = "CloseNotification")]
    async fn close_notification(&mut self, notification_id: u32) -> zbus::fdo::Result<()> {
        self.dbus_method_sender
            .send(DbusMethod::CloseNotification { notification_id })
            .await
            .map_err(KrustifyError::from)?;

        Ok(())
    }

    #[dbus_interface(name = "Notify")]
    async fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, Value<'_>>,
        expire_timeout: i32,
    ) -> zbus::fdo::Result<u32> {
        let desktop_entry = if hints.contains_key("desktop-entry") {
            zbus::zvariant::Str::try_from(&hints["desktop-entry"])
                .map_err(KrustifyError::from)?
                .to_string()
        } else {
            String::new()
        };

        let image_data_property_name = if hints.contains_key("image-data") {
            Some("image-data")
        } else if hints.contains_key("image_data") {
            Some("image_data")
        } else if hints.contains_key("icon-data") {
            Some("icon-data")
        } else if hints.contains_key("icon_data") {
            Some("icon_data")
        } else {
            None
        };

        let image_data = if let Some(name) = image_data_property_name {
            let image_structure =
                zbus::zvariant::Structure::try_from(&hints[name]).map_err(KrustifyError::from)?;

            let fields = image_structure.fields();
            let width_value = &fields[0];
            let height_value = &fields[1];
            let rowstride_value = &fields[2];
            let has_alpha_value = &fields[3];
            let bits_per_sample_value = &fields[4];
            let channels_value = &fields[5];
            let data_value = &fields[6];

            let image_raw_bytes_array = Array::try_from(data_value)
                .map_err(KrustifyError::from)?
                .get()
                .to_vec();

            let width = i32::try_from(width_value).map_err(KrustifyError::from)?;
            let height = i32::try_from(height_value).map_err(KrustifyError::from)?;
            let rowstride = i32::try_from(rowstride_value).map_err(KrustifyError::from)?;
            let has_alpha = bool::try_from(has_alpha_value).map_err(KrustifyError::from)?;
            let bits_per_sample =
                i32::try_from(bits_per_sample_value).map_err(KrustifyError::from)?;
            let channels = i32::try_from(channels_value).map_err(KrustifyError::from)?;

            // TODO: this one's tricky
            let data = image_raw_bytes_array
                .iter()
                .map(|value| {
                    u8::try_from(value)
                        .ok()
                        .expect("value in image data was not a u8")
                })
                .collect::<Vec<_>>();

            Some(ImageData::new(
                width,
                height,
                rowstride,
                has_alpha,
                bits_per_sample,
                channels,
                data,
            ))
        } else {
            None
        };

        let image_path = if hints.contains_key("image-path") {
            Some(
                zbus::zvariant::Str::try_from(&hints["image-path"])
                    .map_err(KrustifyError::from)?
                    .to_string(),
            )
        } else {
            None
        };

        let notification_id = if replaces_id == 0 {
            self.count += 1;
            self.count
        } else {
            replaces_id
        };

        let notification = Notification {
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
            desktop_entry,
        };

        self.dbus_method_sender
            .send(DbusMethod::Notify { notification })
            .await
            .map_err(KrustifyError::from)?;

        Ok(notification_id)
    }

    #[dbus_interface(
        out_args("name", "vendor", "version", "spec_version"),
        name = "GetServerInformation"
    )]
    fn get_server_information(&mut self) -> zbus::fdo::Result<(String, String, String, String)> {
        let name = String::from("Notification Daemon");
        let vendor = String::from(env!("CARGO_PKG_NAME"));
        let version = String::from(env!("CARGO_PKG_VERSION"));
        let specification_version = String::from("1.2");

        Ok((name, vendor, version, specification_version))
    }

    #[dbus_interface(name = "GetCapabilities")]
    fn get_capabilities(&mut self) -> zbus::fdo::Result<Vec<&str>> {
        let capabilities = [
            "action-icons",
            "actions",
            "body",
            "body-hyperlinks",
            "body-images",
            "body-markup",
            "icon-multi",
            "icon-static",
            "persistence",
            "sound",
        ]
        .to_vec();

        Ok(capabilities)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (dbus_method_sender, mut dbus_method_receiver) = mpsc::channel(5);
    let (dbus_signal_sender, mut dbus_signal_receiver) = mpsc::unbounded_channel();

    let notification_handler = NotificationHandler {
        count: 0,
        dbus_method_sender,
    };
    let connection = ConnectionBuilder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", notification_handler)?
        .build()
        .await?;

    tokio::spawn(async move {
        while let Some(signal) = dbus_signal_receiver.recv().await {
            match signal {
                DbusSignal::ActionInvoked { notification_id } => {
                    connection
                        .emit_signal(
                            None::<()>,
                            "/org/freedesktop/Notifications",
                            "org.freedesktop.Notifications",
                            "ActionInvoked",
                            &(notification_id as u32, "default"),
                        )
                        .await
                        .expect("could not emit ActionInvoked signal");
                }
                DbusSignal::NotificationClosed {
                    notification_id,
                    reason,
                } => {
                    connection
                        .emit_signal(
                            None::<()>,
                            "/org/freedesktop/Notifications",
                            "org.freedesktop.Notifications",
                            "NotificationClosed",
                            &(notification_id, reason),
                        )
                        .await
                        .expect("could not emit NotificationClosed signal");
                }
            }
        }
    });

    QApplication::init(|_app| unsafe {
        QCoreApplication::set_organization_name(&qs(env!("CARGO_PKG_NAME")));
        QCoreApplication::set_application_name(&qs(env!("CARGO_PKG_NAME")));

        load_settings();

        let main_window = QMainWindow::new_0a();

        let desktop = QApplication::desktop();

        let topleft = desktop
            .screen_geometry_int(SETTINGS.screen.id.clone())
            .top_left();

        main_window.set_window_flags(
            WindowType::WindowTransparentForInput
                | WindowType::WindowStaysOnTopHint
                | WindowType::FramelessWindowHint
                | WindowType::BypassWindowManagerHint
                | WindowType::X11BypassWindowManagerHint,
        );

        main_window.set_attribute_1a(WidgetAttribute::WATranslucentBackground);
        main_window.set_attribute_1a(WidgetAttribute::WADeleteOnClose);
        main_window.set_attribute_1a(WidgetAttribute::WANoSystemBackground);
        main_window.set_style_sheet(&qs("background-color: transparent;"));

        let main_frame = QFrame::new_1a(main_window.as_ptr());

        main_frame.set_attribute_1a(WidgetAttribute::WATranslucentBackground);
        main_frame.set_style_sheet(&qs("background-color: transparent;"));

        main_window.set_geometry_4a(topleft.x(), 0, 0, 0);

        main_window.show();

        let spawner = NotificationSpawner::new(dbus_signal_sender, main_frame);

        spawner.init();

        let notitification_signal = SignalOfQString::new();
        notitification_signal.connect_with_type(
            ConnectionType::QueuedConnection,
            &spawner.slot_on_spawn_notification(),
        );

        let closed_notification_signal = SignalOfInt::new();
        closed_notification_signal.connect_with_type(
            ConnectionType::QueuedConnection,
            &spawner.slot_on_external_close(),
        );

        let ref_notification_signal = notitification_signal
            .as_raw_ref()
            .expect("could not get a reference to notification_signal");
        let ref_closed_notification_signal = closed_notification_signal
            .as_raw_ref()
            .expect("could not get a reference to notification signal");

        tokio::spawn(async move {
            while let Some(method) = dbus_method_receiver.recv().await {
                match method {
                    DbusMethod::CloseNotification { notification_id } => {
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            ref_closed_notification_signal.emit(notification_id as i32);
                        });
                    }
                    DbusMethod::Notify { notification } => {
                        if !SETTINGS.do_not_disturb.value {
                            let guid = Uuid::new_v4().to_string();
                            let mut list = notification_spawner::NOTIFICATION_LIST
                                .lock()
                                .expect("could not acquire lock to notification list");
                            list.insert(guid.clone(), notification);
                            ref_notification_signal.emit(&QString::from_std_str(&guid));
                        }
                    }
                }
            }
        });

        let _tray_icon = generate_tray();

        QApplication::exec()
    })
}
