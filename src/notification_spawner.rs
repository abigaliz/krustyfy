use std::{cell::RefMut, rc::Rc, slice::Iter};
use std::cell::RefCell;

use cpp_core::{Ptr, StaticUpcast, };
use tokio::sync::mpsc::UnboundedSender;

use qt_core::{ConnectionType, QBox, QObject, qs, QString, QTimer, SignalNoArgs, SignalOfInt, SignalOfQString, slot, SlotNoArgs, SlotOfInt, SlotOfQString};

use crate::{image_handler, notification::{ImageData, Notification}, notification_widget::notifications::{self, NotificationWidget}};

struct VecRefWrapper<'a, T: 'a> {
    r: RefMut<'a, Vec<T>>
}

impl<'a, 'b: 'a, T: 'a> IntoIterator for &'b VecRefWrapper<'a, T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.r.iter()
    }
}

pub struct NotificationSpawner {
    widget_list: RefCell<Vec<Rc<notifications::NotificationWidget>>>,
    check_hover: QBox<SignalNoArgs>,
    qobject: QBox<QObject>,
    action_sender: UnboundedSender<i32>,
    timer: QBox<QTimer>
}

impl StaticUpcast<QObject> for NotificationSpawner {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.qobject.as_ptr().static_upcast()
    }
}

impl NotificationSpawner {
    pub fn new(action_sender: UnboundedSender<i32>) -> Rc<NotificationSpawner> {
        unsafe {
            let qobject = QObject::new_0a();

            let widget_list = RefCell::new(Vec::new());
            let timer = QTimer::new_0a();
            timer.set_interval(100);

            let check_hover= SignalNoArgs::new();

            timer.timeout().connect(&check_hover);

            Rc::new(Self {
                widget_list,
                check_hover,
                qobject,
                action_sender,
                timer
            })
        }
    }

    pub unsafe fn int_timer(self: &Rc<Self>)
    {
        self.timer.start_0a();
    }

    #[slot(SlotOfQString)]
    pub unsafe fn on_spawn_notification(self: &Rc<Self>, serialized_notification: cpp_core::Ref<QString>) {
        let notification: Notification = serde_json::from_str(&serialized_notification.to_std_string()).unwrap();

        self.spawn_notification(
            notification.app_name, 
            notification.replaces_id, 
            notification.app_icon, 
            notification.summary, 
            notification.body, 
            notification.actions, 
            notification.image_data, 
            notification.expire_timeout, 
            notification.notification_id);
    }

    pub unsafe fn spawn_notification(
        self: &Rc<Self>, 
        app_name: String, 
        _replaces_id: u32, 
        _app_icon: String, 
        summary: String, 
        body: String, 
        _actions: Vec<String>,
        image_data: ImageData,
        _expire_timeout: i32,
        notification_id: u32) {

        let close_signal = SignalOfQString::new();

        close_signal.connect_with_type( ConnectionType::QueuedConnection, &self.slot_on_widget_close());

        let action_signal = SignalOfInt::new();

        action_signal.connect_with_type( ConnectionType::QueuedConnection, &self.slot_on_action());
        
        let _notification_widget = NotificationWidget::new(close_signal, action_signal, notification_id);

        self.check_hover.connect(&_notification_widget.slot_check_hover());

        let icon =    if !image_data.desktop_entry.is_empty() {
                                        image_handler::find_icon(&image_data.desktop_entry)
                                    }
                                    else {
                                        image_handler::find_icon(&app_name)
                                    };

        if image_data.is_empty {
            _notification_widget.set_content(qs(app_name), qs(summary), qs(body), icon);
        }
        else {
            let pixmap = image_handler::parse_image(image_data);
            _notification_widget.set_content_with_image(qs(app_name), qs(summary), qs(body), pixmap, icon);
        }

        _notification_widget.animate_entry((*self.widget_list.borrow()).len() as i32);


        (*self.widget_list.borrow_mut()).push(_notification_widget);
        self.reorder();
    }

    unsafe fn reorder(self : &Rc<Self>) {
        let signal = SignalNoArgs::new();

        signal.connect_with_type(ConnectionType::QueuedConnection,&self.slot_on_reorder());

        signal.emit();
    }

    #[slot(SlotNoArgs)]
    unsafe fn on_reorder(self : &Rc<Self>) {
        let list = self.widget_list.borrow();

        for i in 0..list.len() {
            let widget = &list[i];

            let topleft = widget.widget.screen().geometry().top_left();

            widget.widget.set_geometry_4a(topleft.x(), widget.widget.y(), widget.widget.width(), widget.widget.height());
            widget.animate_entry_signal.emit(i as i32);
        }
    }

    #[slot(SlotOfInt)]
    unsafe fn on_action(self: &Rc<Self>, notifcation_id: i32) {
        self.action_sender.send(notifcation_id).unwrap();
    }

    #[slot(SlotOfQString)]
    unsafe fn on_widget_close(self : &Rc<Self>, closed_widget: cpp_core::Ref<QString>) {    
        let mut notification_widget_index: usize = 0;

        let mut list = self.widget_list.borrow_mut();

        for i in 0..&list.len() - 1 {
            let widget = &list[i];
            if widget.widget.win_id().to_string() == closed_widget.to_std_string() {
                notification_widget_index = i;
                break;
            }
        }

        list.remove(notification_widget_index);

        self.reorder();
    }
}