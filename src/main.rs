use std::{error::Error, future::pending};
use async_std::channel::Sender;
use notification::Notification;
use notification_widget::notifications::NotificationWidget;
use notification_spawner::NotificationSpawner;
use qt_core::{QTimer, SlotNoArgs};
use qt_widgets::QApplication;
use signals2::{Signal, Emit7, Connect7};
use zbus::{ConnectionBuilder, dbus_interface, MessageBuilder, Message, MessageField};
use std::collections::HashMap;
use zbus::zvariant::Value;
mod notification_widget;
mod notification_spawner;
mod notification;

//static 
struct NotificationHandler {
    count: u64,
    sender: Sender<Notification>,
    //callback: fn(String, u32, String, String, String, Vec<String>, HashMap<String, Value<'_>>, i32),
}

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl NotificationHandler {
    #[dbus_interface(name="Notify")]
    async fn notify(&mut self, app_name: String, replaces_id: u32, app_icon: String, summary: String, body: String, actions: Vec<String>,
        hints: HashMap<String, Value<'_>>,
        expire_timeout: i32,  ) -> zbus::fdo::Result<u32> {

        let notification = notification::Notification::new(app_name, replaces_id, app_icon, summary, body, actions, expire_timeout);

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


    
    QApplication::init(|app| unsafe {

        let spawner = NotificationSpawner::new();
        
        let timer = QTimer::new_0a();

        timer.timeout().connect(&SlotNoArgs::new(&timer, move || {
            let message =  rx.try_recv();

            if (message.is_ok()) {
                let notification = message.unwrap();
                spawner.spawn_notification(notification.app_name, notification.replaces_id, notification.app_icon, notification.summary, notification.body, notification.actions, notification.expire_timeout);
            }
        }));

        timer.start_1a(100);

        QApplication::exec()

    });

    pending::<()>().await;

    Ok(())
}
