use std::rc::Rc;
use std::sync::Mutex;

use cpp_core::{Ptr, StaticUpcast };

use linked_hash_map::LinkedHashMap;

use tokio::sync::mpsc::UnboundedSender;

use qt_core::{ConnectionType, QBox, QObject, qs, QString, QTimer, SignalNoArgs, SignalOfInt, SlotOfQVariant, SignalOfQString, slot, SlotNoArgs, SlotOfInt, SlotOfQString, QVariant, QFile};
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
    template_file: QBox<QFile>,
}

impl StaticUpcast<QObject> for NotificationSpawner {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.qobject.as_ptr().static_upcast()
    }
}

impl NotificationSpawner {
    pub fn new(action_sender: UnboundedSender<i32>, template_file: QBox<QFile>) -> Rc<NotificationSpawner> {
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

            Rc::new(Self {
                widget_list,
                check_hover,
                action_sender,
                timer,
                reorder_signal,
                action_signal,
                close_signal,
                qobject,
                template_file
            })
        }
    }

    pub unsafe fn init(self: &Rc<Self>)
    {
        self.timer.start_0a();

        self.reorder_signal.connect_with_type(ConnectionType::QueuedConnection,&self.slot_on_reorder());

        self.close_signal.connect_with_type(ConnectionType::QueuedConnection, &self.slot_on_widget_close());

        self.action_signal.connect_with_type(ConnectionType::QueuedConnection, &self.slot_on_action());
    }

    #[slot(SlotOfQVariant)]
    pub unsafe fn on_spawn_notification(self: &Rc<Self>, serialized_notification: cpp_core::Ref<QVariant>) {
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

    pub unsafe fn spawn_notification(
        self: &Rc<Self>, 
        app_name: String, 
        _replaces_id: u32, 
        _app_icon: String, 
        summary: String, 
        body: String, 
        _actions: Vec<String>,
        image_data: Option<ImageData>,
        image_path: Option<String>,
        _expire_timeout: i32,
        notification_id: u32,
        desktop_entry: String) {

        let guid = Uuid::new_v4().to_string();

        let _notification_widget = NotificationWidget::new(
            &self.template_file,
            &self.close_signal, 
            &self.action_signal, 
            notification_id, guid.clone());
 
        self.check_hover.connect(&_notification_widget.slot_check_hover());

        let icon =    if !desktop_entry.is_empty() {
                                        image_handler::find_icon(&desktop_entry)
                                    }
                                    else {
                                        image_handler::find_icon(&app_name)
                                    };

        if image_data.is_none() && image_path.is_none() {
            _notification_widget.set_content_no_image(qs(app_name), qs(summary), qs(body), icon);
        }
        else {
            let pixmap = if image_data.is_some() {
                image_handler::parse_image(&image_data.unwrap())
            }
            else {
                image_handler::load_image(image_path.unwrap())
            };
            
            _notification_widget.set_content_with_image(qs(app_name), qs(summary), qs(body), pixmap, icon);
        }

        self.widget_list.lock().unwrap().insert(guid, _notification_widget);

        self.reorder();
    } 

    unsafe fn reorder(self : &Rc<Self>) {
        self.reorder_signal.emit();
    }

    #[slot(SlotNoArgs)]
    unsafe fn on_reorder(self : &Rc<Self>) {
        let list = self.widget_list.lock().unwrap();

        let mut counter = 0;
        for widget in list.values() {
            widget.animate_entry_signal.emit(counter);
            counter += 1;
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

        list.remove(&closed_widget.to_std_string());

        self.reorder(); 
    }
}