use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use qt_core::q_dir::Filter;
use qt_core::{qs, ConnectionType, QString, SignalOfInt, SignalOfQString, QDirIterator, QDir, QVariant, QSettings, QCoreApplication};
use qt_gui::QIcon;
use qt_widgets::{QApplication, QMenu, QSystemTrayIcon, SlotOfQAction, QActionGroup};
use tokio::{
    self,
    sync::mpsc::{self, Sender},
};
use uuid::Uuid;
use zbus::{dbus_interface, zvariant::Array, ConnectionBuilder};
use zvariant::Value;

use crate::dbus_signal::{DbusMethod, DbusSignal};
use notification::{ImageData, Notification};
use notification_spawner::NotificationSpawner;

use lazy_static::lazy_static;

mod dbus_signal;
mod image_handler;
mod notification;
mod notification_spawner;
mod notification_widget;

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

        self.dbus_method_sender
            .send(DbusMethod::Notify { notification })
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

lazy_static! {
    pub static ref THEME : Mutex<String> = Mutex::new("default".to_string());
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
                        .unwrap();
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
                        .unwrap();
                }
            }
        }
    });

    QApplication::init(|_app| unsafe {
        let settings = QSettings::new();

        QCoreApplication::set_organization_name(&qs("abigaliz"));
        QCoreApplication::set_application_name(&qs(env!("CARGO_PKG_NAME")));

        let theme_setting = settings.value_1a(&qs("theme"));

        if theme_setting.is_null() {
            theme_setting.set_value(&QVariant::from_q_string(&qs("default")));
        } else {
            let mut _theme = THEME.lock().expect("Could not lock mutex");
            *_theme = theme_setting.to_string().to_std_string();
        }

        let spawner = NotificationSpawner::new(dbus_signal_sender);

        spawner.init();

        let do_not_disturb = Arc::new(AtomicBool::new(false));

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

        let do_not_disturb_clone = do_not_disturb.clone();
        let ref_notification_signal = notitification_signal.as_raw_ref().unwrap();
        let ref_closed_notification_signal = closed_notification_signal.as_raw_ref().unwrap();

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
                        if !do_not_disturb_clone.load(Ordering::Relaxed) {
                            let guid = Uuid::new_v4().to_string();
                            let mut list = notification_spawner::NOTIFICATION_LIST.lock().unwrap();
                            list.insert(guid.clone(), notification);
                            ref_notification_signal.emit(&QString::from_std_str(&guid));
                        }
                    }
                }
            }
        });

        let tray_icon = QSystemTrayIcon::new();

        tray_icon.set_icon(&QIcon::from_theme_1a(&qs("notifications")));

        tray_icon.show();

        let tray_menu = QMenu::new();

        let theme_menu = tray_menu.add_menu_q_string(&qs("Themes"));

        let theme_action_group = QActionGroup::new(&theme_menu);
        theme_action_group.set_exclusive(true);

        let theme_directories = QDirIterator::from_q_string_q_flags_filter(&qs("./res/themes"), Filter::AllDirs | Filter::NoDotAndDotDot);

        while theme_directories.has_next() {
            let theme = QDir::new_1a(&theme_directories.next());

            let theme_action = theme_menu.add_action_q_string(&theme.dir_name());

            theme_action.set_object_name(&qs("set_theme"));
            theme_action.set_checkable(true);
            theme_action.set_data(&QVariant::from_q_string(&theme.dir_name()));

            if theme_setting.to_string().compare_q_string(&theme.dir_name())  == 0 {
                theme_action.set_checked(true);
            }

            theme_action_group.add_action_q_action(theme_action.as_ptr());
        }

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

                if action.object_name().to_std_string() == "set_theme".to_string() {
                    let mut _theme = THEME.lock().expect("Could not lock mutex");

                    let theme_name = action.data().to_string().to_std_string();

                    *_theme = theme_name.clone();

                    settings.set_value(&qs("theme"), &QVariant::from_q_string(&qs(theme_name)));
                }
            }));

        QApplication::exec()
    })
}
