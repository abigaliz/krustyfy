use crate::Notification;

#[derive(Debug)]
pub enum DbusSignal {
    ActionInvoked { notification_id: i32 },
    NotificationClosed { notification_id: u32, reason: u32 },
}

#[derive(Debug)]
pub enum DbusMethod {
    CloseNotification { notification_id: u32 },
    Notify { notification: Notification },
}
