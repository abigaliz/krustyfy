use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::{
    self,
    sync::mpsc::{self, Sender},
};
use zbus::{dbus_interface, zvariant::Array, ConnectionBuilder};
use zvariant::Value;

use notification::{ImageData, Notification};
use notification_spawner::NotificationSpawner;
use qt_core::{qs, ConnectionType, SignalOfInt, SignalOfQVariant};
use qt_gui::QIcon;
use qt_widgets::{QApplication, QMenu, QSystemTrayIcon, SlotOfQAction};

mod image_handler;
mod notification;
mod notification_spawner;
mod notification_widget;

//static
struct NotificationHandler {
    count: u32,
    new_notification_sender: Sender<Notification>,
    closed_notification_sender: Sender<u32>,
}

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl NotificationHandler {
    #[dbus_interface(name = "CloseNotification")]
    async fn close_notification(&mut self, notification_id: u32) -> zbus::fdo::Result<()> {
        self.closed_notification_sender
            .send(notification_id)
            .await
            .unwrap();

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
                .ok()
                .unwrap()
                .to_string()
        } else {
            String::new()
        };

        let image_data = if hints.contains_key("image-data") {
            let image_structure = zbus::zvariant::Structure::try_from(&hints["image-data"])
                .ok()
                .unwrap();

            let fields = image_structure.fields();
            let width_value = &fields[0];
            let height_value = &fields[1];
            let rowstride_value = &fields[2];
            let has_alpha_value = &fields[3];
            let bits_per_sample_value = &fields[4];
            let channels_value = &fields[5];
            let data_value = &fields[6];

            let image_raw_bytes_array = Array::try_from(data_value).ok().unwrap().get().to_vec();

            let width = i32::try_from(width_value).ok().unwrap();
            let height = i32::try_from(height_value).ok().unwrap();
            let rowstride = i32::try_from(rowstride_value).ok().unwrap();
            let has_alpha = bool::try_from(has_alpha_value).ok().unwrap();
            let bits_per_sample = i32::try_from(bits_per_sample_value).ok().unwrap();
            let channels = i32::try_from(channels_value).ok().unwrap();
            let data = image_raw_bytes_array
                .iter()
                .map(|value| u8::try_from(value).ok().unwrap())
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
                    .ok()
                    .unwrap()
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

        self.new_notification_sender
            .send(notification)
            .await
            .unwrap();

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
    let (new_notification_sender, mut new_notification_receiver) = mpsc::channel(5);
    let (closed_notification_sender, mut closed_notification_receiver) = mpsc::channel(5);
    let (action_sender, mut action_receiver) = mpsc::unbounded_channel();

    let notification_handler = NotificationHandler {
        count: 0,
        new_notification_sender,
        closed_notification_sender,
    };
    let connection = ConnectionBuilder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", notification_handler)?
        .build()
        .await?;

    tokio::spawn(async move {
        while let Some(message) = action_receiver.recv().await {
            connection
                .emit_signal(
                    None::<()>,
                    "/org/freedesktop/Notifications",
                    "org.freedesktop.Notifications",
                    "ActionInvoked",
                    &(message as u32, "default"),
                )
                .await
                .unwrap();
        }
    });

    QApplication::init(|_app| unsafe {
        let spawner = NotificationSpawner::new(action_sender);

        spawner.init();

        let do_not_disturb = Arc::new(AtomicBool::new(false));

        let notitification_signal = SignalOfQVariant::new();
        notitification_signal.connect_with_type(
            ConnectionType::QueuedConnection,
            &spawner.slot_on_spawn_notification(),
        );

        let closed_notification_signal = SignalOfInt::new();
        closed_notification_signal.connect_with_type(
            ConnectionType::QueuedConnection,
            &spawner.slot_on_external_close(),
        );

        let do_not_disturb_clone = do_not_disturb.clone();
        let ref_notification_signal = notitification_signal.as_raw_ref().unwrap();
        tokio::spawn(async move {
            while let Some(message) = new_notification_receiver.recv().await {
                if !do_not_disturb_clone.load(Ordering::Relaxed) {
                    ref_notification_signal.emit(message.to_qvariant().as_ref());
                }
            }
        });

        let ref_closed_notification_signal = closed_notification_signal.as_raw_ref().unwrap();

        tokio::spawn(async move {
            while let Some(message) = closed_notification_receiver.recv().await {
                tokio::time::sleep(Duration::from_millis(100)).await; // Wait in case it's meant to be replaced;
                ref_closed_notification_signal.emit(message as i32);
            }
        });

        let tray_icon = QSystemTrayIcon::new();

        tray_icon.set_icon(&QIcon::from_theme_1a(&qs("notifications")));

        tray_icon.show();

        let tray_menu = QMenu::new();

        let do_not_disturb_action = tray_menu.add_action_q_string(&qs("Do not disturb"));
        do_not_disturb_action.set_object_name(&qs("do_not_disturb_action"));
        do_not_disturb_action.set_checkable(true);

        let quit_action = tray_menu.add_action_q_string(&qs("Quit"));
        quit_action.set_object_name(&qs("quit_action"));

        tray_icon.set_context_menu(&tray_menu);

        tray_menu
            .triggered()
            .connect(&SlotOfQAction::new(&tray_menu, move |action| {
                if action.object_name().to_std_string() == "quit_action".to_string() {
                    QApplication::close_all_windows();
                    tray_icon.hide();
                }

                if action.object_name().to_std_string() == "do_not_disturb_action".to_string() {
                    do_not_disturb.store(do_not_disturb_action.is_checked(), Ordering::Relaxed);
                }
            }));

        QApplication::exec()
    })
}
