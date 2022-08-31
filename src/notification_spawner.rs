use std::os::unix::prelude::ExitStatusExt;
use std::{cell::RefMut, rc::Rc, slice::Iter};
use std::cell::RefCell;

use cpp_core::{Ptr, StaticUpcast, CppDeletable, };
use qt_widgets::QApplication;
use tokio::sync::mpsc::UnboundedSender;

use qt_core::{ConnectionType, QBox, QObject, qs, QString, QTimer, SignalNoArgs, SignalOfInt, SlotOfQVariant, SignalOfQString, slot, SlotNoArgs, SlotOfInt, SlotOfQString, QVariant, q_variant, QHashOfQStringQVariant};

use crate::{image_handler, notification::{ImageData, Notification}, notification_widget::notifications::{self, NotificationWidget}};

pub struct NotificationSpawner {
    widget_list: RefCell<Vec<Rc<notifications::NotificationWidget>>>,
    test_list: RefCell<Vec<Rc<notifications::TestWidget>>>,
    check_hover: QBox<SignalNoArgs>,
    qobject: QBox<QObject>,
    action_sender: UnboundedSender<i32>,
    timer: QBox<QTimer>,
    reorder_signal: QBox<SignalNoArgs>,
    action_signal: QBox<SignalOfInt>,
    close_signal: QBox<SignalOfQString>,
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
            let test_list = RefCell::new(Vec::new());

            let timer = QTimer::new_0a();
            timer.set_interval(100);

            let check_hover= SignalNoArgs::new();

            timer.timeout().connect(&check_hover);

            let reorder_signal = SignalNoArgs::new();

            let action_signal=  SignalOfInt::new();

            let close_signal=  SignalOfQString::new();

            Rc::new(Self {
                widget_list,
                test_list,
                check_hover,
                qobject,
                action_sender,
                timer,
                reorder_signal,
                action_signal,
                close_signal
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
        //let notification: Notification = serde_json::from_str(&serialized_notification.to_std_string()).unwrap();
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
            notification.expire_timeout, 
            notification.notification_id,
            notification.desktop_entry,);
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
        _expire_timeout: i32,
        notification_id: u32,
        desktop_entry: String) {

/*         let testwidget = notifications::TestWidget::new();
        
        (*self.test_list.borrow_mut()).push(testwidget); */
        
        let _notification_widget = NotificationWidget::new(&self.close_signal, &self.action_signal, notification_id);

        self.check_hover.connect(&_notification_widget.slot_check_hover());

        let icon =    if !desktop_entry.is_empty() {
                                        image_handler::find_icon(&desktop_entry)
                                    }
                                    else {
                                        image_handler::find_icon(&app_name)
                                    };

        if image_data.is_none() {
            _notification_widget.set_content(qs(app_name), qs(summary), qs(body), icon);
        }
        else {
            let pixmap = image_handler::parse_image(&image_data.unwrap());
            _notification_widget.set_content_with_image(qs(app_name), qs(summary), qs(body), pixmap, icon);
        }

        (*self.widget_list.borrow_mut()).push(_notification_widget);
        self.reorder();
    }

    unsafe fn reorder(self : &Rc<Self>) {
        self.reorder_signal.emit();
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
                widget.widget.delete();
                break;
            }
        }

        list.remove(notification_widget_index);

        self.reorder();
    }
}