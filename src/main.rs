use std::error::Error;
use std::collections::HashMap;
use std::convert::TryFrom;


use tokio::{self, sync::mpsc::{self, Sender}};
use zbus::{ConnectionBuilder, dbus_interface, zvariant::Array};
use zvariant::Value;

use notification::{ImageData, Notification};
use notification_spawner::NotificationSpawner;
use qt_core::{ConnectionType, SignalOfQVariant};
use qt_widgets::{QApplication};

mod notification_widget;
mod notification_spawner;
mod notification;
mod image_handler;

//static 
struct NotificationHandler {
    count: u32,
    sender: Sender<Notification>,
}

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl NotificationHandler {
    #[dbus_interface(name="Notify")]
    async fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, Value<'_>>,
        expire_timeout: i32,  ) -> zbus::fdo::Result<u32> {

        let mut desktop_entry = String::new();

        if hints.contains_key("desktop-entry") {
            desktop_entry = zbus::zvariant::Str::try_from(&hints["desktop-entry"]).ok().unwrap().to_string();
        }

        let mut image_data: Option<ImageData> = None;

        if hints.contains_key("image-data") {
            let image_structure = zbus::zvariant::Structure::try_from(&hints["image-data"]).ok().unwrap();

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
            let mut data = Vec::new();
            (&image_raw_bytes_array).iter().for_each(|f| {
                data.push(u8::try_from(f).ok().unwrap());
            });

            image_data = Some(ImageData::new(width, height, rowstride, has_alpha, bits_per_sample, channels, data));
        }
        

        let mut notification_id = replaces_id;
        self.count += 1;

        if notification_id == 0 {
            notification_id = self.count;
        }

        let notification = notification::Notification {
            app_name, replaces_id, app_icon, summary, body, actions, image_data, expire_timeout, notification_id, desktop_entry
        };
        
        self.sender.send(notification).await.unwrap();

        Ok(notification_id)
    }

    #[dbus_interface(out_args("name", "vendor", "version", "spec_version"), name="GetServerInformation")]
    fn get_server_information(&mut self) -> zbus::fdo::Result<(String, String, String, String)> {
        let name = String::from("Notification Daemon");
        let vendor = String::from("krustyfy");
        let version = String::from("v0.0.1");
        let specification_version = String::from("1.2");

        Ok((name, vendor, version, specification_version))
    }

    #[dbus_interface(name="GetCapabilities")]
    fn get_capabilities(&mut self) -> zbus::fdo::Result<Vec<&str>> {

        let capabilities = ["action-icons",
        "actions",
        "body",
        "body-hyperlinks",
        "body-images",
        "body-markup",
        "icon-multi",
        "icon-static",
        "persistence",
        "sound"].to_vec();

        Ok(capabilities)
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (notification_sender, mut notification_receiver) = mpsc::channel(32);
    let (action_sender, mut action_receiver) = mpsc::unbounded_channel();

    let notification_handler = NotificationHandler { count: 0, sender: notification_sender};
    let connection = ConnectionBuilder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", notification_handler)?
        .build()
        .await?;

    tokio::spawn(async move {
        while let Some(message) = action_receiver.recv().await {            
            connection.emit_signal(
                None::<()>, 
                "/org/freedesktop/Notifications", 
                "org.freedesktop.Notifications", 
                "ActionInvoked", 
                &(message as u32, "default")).await.unwrap();
        }
    });

    QApplication::init(|_app| unsafe {
        let spawner = NotificationSpawner::new(action_sender);

        spawner.init();

        let notitification_signal = SignalOfQVariant::new();

        notitification_signal.connect_with_type(ConnectionType::QueuedConnection, &spawner.slot_on_spawn_notification());

        let signal = notitification_signal.as_raw_ref().unwrap();
        tokio::spawn(async move {
            while let Some(message) = notification_receiver.recv().await { 
    
                signal.emit(message.to_qvariant().as_ref());
            }
        });

        QApplication::exec()

    });
}
