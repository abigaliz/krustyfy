use std::{cell::RefCell, rc::Rc};
use std::sync::{Mutex, MutexGuard};

use cpp_core::{Ptr, StaticUpcast, Ref };

use linked_hash_map::LinkedHashMap;

use tokio::sync::mpsc::UnboundedSender;

use qt_core::{ConnectionType, QBox, QObject, qs, QString, QTimer, SignalNoArgs, SignalOfBool, SlotOfBool, SignalOfInt, SlotOfQVariant, SignalOfQString, slot, SlotNoArgs, SlotOfInt, SlotOfQString, QVariant};
use uuid::Uuid;

use crate::{image_handler, notification::{ImageData, Notification}, notification_widget::notifications::{NotificationWidget}};

pub struct NotificationSpawner {
    widget_list: Mutex<LinkedHashMap<String, Rc<NotificationWidget>>>,
    check_hover: QBox<SignalNoArgs>,
    action_sender: UnboundedSender<i32>,
    timer: QBox<QTimer>,
    reorder_signal: QBox<SignalNoArgs>,
    action_signal: QBox<SignalOfInt>,
    close_signal: QBox<SignalOfQString>,
    qobject: QBox<QObject>,
    do_not_disturb: RefCell<bool>,
    do_not_disturb_signal: QBox<SignalOfBool>
}

impl StaticUpcast<QObject> for NotificationSpawner {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.qobject.as_ptr().static_upcast()
    }
}

impl NotificationSpawner {
    pub fn new(action_sender: UnboundedSender<i32>) -> Rc<NotificationSpawner> {
        unsafe {
            let widget_list = Mutex::new(LinkedHashMap::new());

            let timer = QTimer::new_0a();
            timer.set_interval(100);

            let check_hover= SignalNoArgs::new();

            timer.timeout().connect(&check_hover);

            let reorder_signal = SignalNoArgs::new();

            let action_signal=  SignalOfInt::new();

            let close_signal=  SignalOfQString::new();

            let qobject = QObject::new_0a();

            let do_not_disturb = RefCell::new(false);

            let do_not_disturb_signal= SignalOfBool::new();

            Rc::new(Self {
                widget_list,
                check_hover,
                action_sender,
                timer,
                reorder_signal,
                action_signal,
                close_signal,
                qobject,
                do_not_disturb,
                do_not_disturb_signal
            })
        }
    }

    pub unsafe fn init(self: &Rc<Self>)
    {
        self.timer.start_0a();

        self.reorder_signal.connect_with_type(ConnectionType::QueuedConnection,&self.slot_on_reorder());

        self.close_signal.connect_with_type(ConnectionType::QueuedConnection, &self.slot_on_widget_close());

        self.action_signal.connect_with_type(ConnectionType::QueuedConnection, &self.slot_on_action());

        self.do_not_disturb_signal.connect_with_type(ConnectionType::QueuedConnection, &self.slot_on_do_not_disturb());
    }

    pub unsafe fn do_not_disturb(self: &Rc<Self>, do_not_disturb: bool) {
        self.do_not_disturb_signal.emit(do_not_disturb);
    }

    #[slot(SlotOfBool)]
    unsafe fn on_do_not_disturb(self: &Rc<Self>, do_not_disturb: bool) {
        self.do_not_disturb.replace(do_not_disturb);
    }

    #[slot(SlotOfQVariant)]
    pub unsafe fn on_spawn_notification(self: &Rc<Self>, serialized_notification: Ref<QVariant>) {
        let do_not_disturb = self.do_not_disturb.borrow();

        if do_not_disturb.eq(&true) { return ;}

        let notification_hash = serialized_notification.to_hash();

        let notification = Notification::from_qvariant(&notification_hash);

        self.spawn_notification(
            notification.app_name, 
            notification.replaces_id, 
            notification.app_icon, 
            notification.summary, 
            notification.body, 
            notification.actions, 
            notification.image_data, 
            notification.image_path,
            notification.expire_timeout, 
            notification.notification_id,
            notification.desktop_entry);
    }

    pub unsafe fn get_already_existing_notification<'a>(
        self: &Rc<Self>, list: &'a MutexGuard<LinkedHashMap<String, Rc<NotificationWidget>>>, 
        app_name: &String, 
        replaces_id: u32) -> Option<&'a Rc<NotificationWidget>> {
        for widget in list.values() {
            let _replaces_id = widget.notification_id.borrow().to_owned();

            if _replaces_id == replaces_id
            {
                return Some(widget);
            }

            if app_name.eq("discord") && _replaces_id == replaces_id - 1 // Fuck you Discord
            {
                widget.notification_id.replace(replaces_id);
                return Some(widget);
            }
        }

        None
    }

    pub unsafe fn spawn_notification(
        self: &Rc<Self>, 
        app_name: String, 
        replaces_id: u32, 
        _app_icon: String, 
        summary: String, 
        body: String, 
        _actions: Vec<String>,
        image_data: Option<ImageData>,
        image_path: Option<String>,
        _expire_timeout: i32,
        notification_id: u32,
        desktop_entry: String) {

        let mut list = self.widget_list.lock().unwrap();

        let already_existing_notification = self.get_already_existing_notification(&list, &app_name, replaces_id);

        if already_existing_notification.is_none() {
            let guid = Uuid::new_v4().to_string();

            let _notification_widget = NotificationWidget::new(
                &self.close_signal, 
                &self.action_signal, 
                notification_id, guid.clone());

            self.set_notification_contents(app_name, image_data, image_path, desktop_entry, summary, body, &_notification_widget);

            self.check_hover.connect(&_notification_widget.slot_check_hover());
            
            list.insert(guid, _notification_widget);

            self.reorder(); 
        } else {
            let notification_widget = already_existing_notification.unwrap();

            notification_widget.reset_timer();

            self.set_notification_contents(app_name, image_data, image_path, desktop_entry, summary, body, notification_widget);
        };
    } 

    unsafe fn set_notification_contents(
        self: &Rc<Self>,
        app_name: String, 
        image_data: Option<ImageData>,
        image_path: Option<String>,
        desktop_entry: String,
        summary: String, 
        body: String,
        notification_widget: &Rc<NotificationWidget>
        ) {
        let icon = if !desktop_entry.is_empty() {
            image_handler::find_icon(&desktop_entry)
         }
         else {
             image_handler::find_icon(&app_name)
         };

        if image_data.is_none() && image_path.is_none() {
            notification_widget.set_content_no_image(qs(app_name), qs(summary), qs(body), icon);
        }
        else {
            let pixmap = if image_data.is_some() {
                image_handler::parse_image(&image_data.unwrap())
            }
            else {
                image_handler::load_image(image_path.unwrap())
            };

            notification_widget.set_content_with_image(qs(app_name), qs(summary), qs(body), pixmap, icon);
        }
    }

    unsafe fn reorder(self : &Rc<Self>) {
        self.reorder_signal.emit();
    }

    #[slot(SlotNoArgs)]
    unsafe fn on_reorder(self : &Rc<Self>) {
        let list = self.widget_list.lock().unwrap();

        let mut height_accumulator = 0;
        for widget in list.values() {
            widget.animate_entry_signal.emit(height_accumulator);
            height_accumulator += widget.widget.height();
        }
    }

    #[slot(SlotOfInt)]
    unsafe fn on_action(self: &Rc<Self>, notifcation_id: i32) {
        self.action_sender.send(notifcation_id).unwrap();
    }

    #[slot(SlotOfQString)]
    unsafe fn on_widget_close(self : &Rc<Self>, closed_widget: cpp_core::Ref<QString>) {    
        let mut list = self.widget_list.lock().unwrap();

        let widget = &list[&closed_widget.to_std_string()];
        widget.widget.close();
        widget.overlay.close();

        list.remove(&closed_widget.to_std_string());

        self.reorder(); 
    }

    #[slot(SlotOfInt)]
    pub unsafe fn on_external_close(self : &Rc<Self>, notification_id: i32) {    
        let list = self.widget_list.lock().unwrap();

        for widget in list.values() {
            let _notification_id = widget.notification_id.borrow().to_owned();
            if _notification_id as i32 == notification_id {
                widget.on_close();
                break;
            }
        }
    }
}