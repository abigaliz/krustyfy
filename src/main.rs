use std::{error::Error};
use async_std::channel::Sender;
use notification::Notification;
use notification_spawner::NotificationSpawner;
use qt_core::{QTimer, SlotNoArgs};
use qt_widgets::QApplication;
use zbus::{ConnectionBuilder, dbus_interface, zvariant::Array};
use std::collections::HashMap;
use std::convert::TryFrom;
use zvariant::{Value};
mod notification_widget;
mod notification_spawner;
mod notification;
mod image_handler;

//static 
struct NotificationHandler {
    count: u64,
    sender: Sender<Notification>,
}

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl NotificationHandler {
    #[dbus_interface(name="Notify")]
    async fn notify(&mut self, app_name: String, replaces_id: u32, app_icon: String, summary: String, body: String, actions: Vec<String>,
        hints: HashMap<String, Value<'_>>,
        expire_timeout: i32,  ) -> zbus::fdo::Result<u32> {

        let mut icon: String;

        if hints.contains_key("desktop-entry") {
            icon = zbus::zvariant::Str::try_from(&hints["desktop-entry"]).ok().unwrap().to_string();
        }

        let mut image = Vec::new();

        let mut image_has_alpha: bool = false;


        if hints.contains_key("image-data") {
        
            let image_data = zbus::zvariant::Structure::try_from(&hints["image-data"]).ok().unwrap().clone();

            let image_raw_bytes = image_data.fields()[6].clone();
            let image_raw_alpha = image_data.fields()[3].clone();

            let image_raw_bytes_array = Array::try_from(image_raw_bytes).ok().unwrap().get().to_vec();

            (&image_raw_bytes_array).to_owned().into_iter().for_each(|f| {
                image.push(u8::try_from(f).ok().unwrap());
            });

            println!("{}", image.len().to_string());

            image_has_alpha = bool::try_from(image_raw_alpha).ok().unwrap();
        }
        

        let notification = notification::Notification {
            app_name, replaces_id, app_icon, summary, body, actions, image_data: image, image_has_alpha, expire_timeout
        };
        
        self.sender.send(notification).await;

        self.count += 1;
        return Ok(0);
    }

    #[dbus_interface(out_args("name", "vendor", "version", "spec_version"), name="GetServerInformation")]
    fn get_server_information(&mut self) -> zbus::fdo::Result<(String, String, String, String)> {
        let name = String::from("");
        let vendor = String::from("notif");
        let version = String::from("1");
        let specification_version = String::from("1.2");

        Ok((name, vendor, version, specification_version))
    }
}


#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = async_std::channel::unbounded();

    let notification_handler = NotificationHandler { count: 0, sender: tx};
    let _ = ConnectionBuilder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", notification_handler)?
        .build()
        .await?;


    
    QApplication::init(|_| unsafe {

        let spawner = NotificationSpawner::new();
        
        let timer = QTimer::new_0a();

        timer.timeout().connect(&SlotNoArgs::new(&timer, move || {
            let message =  rx.try_recv();

            if message.is_ok() {
                let notification = message.unwrap();
                spawner.spawn_notification(notification.app_name, notification.replaces_id, notification.app_icon, notification.summary, notification.body, notification.actions, notification.expire_timeout);
            }
        }));

        timer.start_1a(100);

        QApplication::exec()

    });
}
