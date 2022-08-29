use std::{error::Error};
use notification::{Notification, ImageData};
use notification_spawner::NotificationSpawner;
use qt_core::{QTimer, SlotNoArgs};
use qt_widgets::QApplication;
use zbus::{ConnectionBuilder, dbus_interface, zvariant::Array, export::futures_util::StreamExt};
use std::collections::HashMap;
use std::convert::TryFrom;
use zvariant::{Value};
use tokio::{self, sync::mpsc::{Sender, Receiver, self}};
mod notification_widget;
mod notification_spawner;
mod notification;
mod image_handler;

//static 
struct NotificationHandler {
    count: u64,
    sender: Sender<Notification>,
    receiver: Receiver<i32>,
}

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl NotificationHandler {
    #[dbus_interface(name="Notify")]
    async fn notify(&mut self, app_name: String, replaces_id: u32, app_icon: String, summary: String, body: String, actions: Vec<String>,
        hints: HashMap<String, Value<'_>>,
        expire_timeout: i32,  ) -> zbus::fdo::Result<u32> {

        let mut desktop_entry = String::new();

        if hints.contains_key("desktop-entry") {
            desktop_entry = zbus::zvariant::Str::try_from(&hints["desktop-entry"]).ok().unwrap().to_string();
        }

        let mut image_data = ImageData::empty();

        if hints.contains_key("image-data") {
            let image_structure = zbus::zvariant::Structure::try_from(&hints["image-data"]).ok().unwrap().clone();

            let width_value = image_structure.fields()[0].clone();
            let height_value = image_structure.fields()[1].clone();
            let rowstride_value = image_structure.fields()[2].clone();
            let has_alpha_value = image_structure.fields()[3].clone();
            let bits_per_sample_value = image_structure.fields()[4].clone();
            let channels_value = image_structure.fields()[5].clone();
            let data_value = image_structure.fields()[6].clone();

            let image_raw_bytes_array = Array::try_from(data_value).ok().unwrap().get().to_vec();

            
            let width = i32::try_from(width_value).ok().unwrap();
            let height = i32::try_from(height_value).ok().unwrap();
            let rowstride = i32::try_from(rowstride_value).ok().unwrap();
            let has_alpha = bool::try_from(has_alpha_value).ok().unwrap();
            let bits_per_sample = i32::try_from(bits_per_sample_value).ok().unwrap();
            let channels = i32::try_from(channels_value).ok().unwrap();
            let mut data = Vec::new();
            (&image_raw_bytes_array).to_owned().into_iter().for_each(|f| {
                data.push(u8::try_from(f).ok().unwrap());
            });

            image_data = ImageData::new(width, height, rowstride, has_alpha, bits_per_sample, channels, data, desktop_entry);
        }
        

        let notification = notification::Notification {
            app_name, replaces_id, app_icon, summary, body, actions, image_data, expire_timeout
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


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (notification_sender, mut notification_receiver) = mpsc::channel(32);
    let (action_sender, action_receiver) = mpsc::channel(32);

    let notification_handler = NotificationHandler { count: 0, sender: notification_sender, receiver: action_receiver};
    let _ = ConnectionBuilder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", notification_handler)?
        .build()
        .await?;

/*         spawn(async move {
            while let Some(message) = action_receiver.recv().await {
                println!("GOT = {}", message);
            }
        }); */

    
    QApplication::init(|_| unsafe {

        let spawner = NotificationSpawner::new(action_sender);
        
        let timer = QTimer::new_0a();

        timer.timeout().connect(&SlotNoArgs::new(&timer, move || {
            let notification_message =  notification_receiver.try_recv();

            if notification_message.is_ok() {
                let notification = notification_message.unwrap();
                spawner.spawn_notification(
                    notification.app_name, 
                    notification.replaces_id, 
                    notification.app_icon, 
                    notification.summary, 
                    notification.body, 
                    notification.actions, 
                    notification.image_data,
                    notification.expire_timeout);
            }
        }));

        timer.start_1a(100);

        QApplication::exec()

    });
}
