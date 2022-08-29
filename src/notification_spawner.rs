use std::{rc::Rc, collections::HashMap, slice::Iter, borrow::Borrow, cell::RefMut};
use cpp_core::{CppBox, StaticUpcast, Ptr, };
use qt_core::{QBox, SignalNoArgs, QTimer, qs, QString, slot, SlotNoArgs, SlotOfInt, QEvent, QObject, SlotOfQString, QPtr, SignalOfQString, QListOfQObject, ConnectionType, SignalOfInt};
use qt_gui::{QCloseEvent, SlotOfQWindow, QWindow, SignalOfQWindow, QPixmap};
use qt_widgets::{QWidget, QDialog, SlotOfQWidgetQWidget};
use signals2::Connect12;
use tokio::sync::mpsc::Sender;
use zbus::zvariant::Value;
use std::cell::{RefCell, Ref};

use crate::{notification_widget::notifications::{self, NotificationWidget}, notification::{Notification, ImageData}, image_handler};

struct VecRefWrapper<'a, T: 'a> {
    r: RefMut<'a, Vec<T>>
}

impl<'a, 'b: 'a, T: 'a> IntoIterator for &'b VecRefWrapper<'a, T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Iter<'a, T> {
        self.r.iter()
    }
}

pub struct NotificationSpawner {
    widget_list: RefCell<Vec<Rc<notifications::NotificationWidget>>>,
    check_hover: QBox<SignalNoArgs>,
    reset_anim_time: QBox<SignalNoArgs>,
    timer: QBox<QTimer>,
    qobject: QBox<QObject>,
}

impl StaticUpcast<QObject> for NotificationSpawner {
    unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
        ptr.qobject.as_ptr().static_upcast()
    }
}

impl NotificationSpawner {
    pub fn new(action_sender: Sender<i32>) -> Rc<NotificationSpawner> {
        unsafe {
            let qobject = QObject::new_0a();

            let widget_list = RefCell::new(Vec::new());
            let qwidget_list = QListOfQObject::new();
            let timer = QTimer::new_0a();
            timer.set_interval(100);

            let reset_anim_time = SignalNoArgs::new();
            let check_hover= SignalNoArgs::new();

            timer.timeout().connect(&check_hover);

            timer.start_0a();

            let this = Rc::new(Self {
                widget_list,
                check_hover,
                reset_anim_time,
                timer,
                qobject,
            });
            
            this
        }
    }

    pub unsafe fn init_timer(&mut self) {
        self.timer.start_0a()
    }

    pub unsafe fn spawn_notification(
        self: &Rc<Self>, 
        app_name: String, 
        replaces_id: u32, 
        app_icon: String, 
        summary: String, 
        body: String, 
        actions: Vec<String>,
        image_data: ImageData,
        expire_timeout: i32,) {

        let close_signal = SignalOfQString::new();

        close_signal.connect_with_type( ConnectionType::QueuedConnection, &self.slot_on_widget_close());

        let action_signal = SignalOfInt::new();

        action_signal.connect_with_type( ConnectionType::QueuedConnection, &self.slot_on_action());
        
        let _notification_widget = NotificationWidget::new(close_signal, action_signal, replaces_id);

        self.check_hover.connect(&_notification_widget.slot_check_hover());

        let mut icon = QPixmap::new();

        if !image_data.desktop_entry.is_empty() {
            icon = image_handler::find_icon(&image_data.desktop_entry);
        }
        else {
            icon = image_handler::find_icon(&app_name);
        }

        if image_data.is_empty {
            _notification_widget.set_content(qs(app_name), qs(summary), qs(body), icon);
        }
        else {
            let pixmap = image_handler::parse_image(image_data);
            _notification_widget.set_content_with_image(qs(app_name), qs(summary), qs(body), pixmap, icon);
        }

        _notification_widget.animate_entry((*self.widget_list.borrow()).len() as i32);
        
        println!("pushing new widget with id {}", _notification_widget.widget.win_id().to_string());

        (*self.widget_list.borrow_mut()).push(_notification_widget);
        self.reorder();
    }

    unsafe fn reorder(self : &Rc<Self>) {
        let signal = SignalNoArgs::new();

        signal.connect_with_type(ConnectionType::DirectConnection,&self.slot_on_reorder());

        println!("Emitting reorder signal");
        signal.emit();
    }

    #[slot(SlotNoArgs)]
    unsafe fn on_reorder(self : &Rc<Self>) {
        println!("reordering");

        let list = &(*self.widget_list.borrow());

        println!("there are {} items", list.len().to_string());

        if list.len() == 0 {
            return;
        }

        for i in 0..list.len() {
            println!("looping through item {} ", i.to_string());
            let widget = &list[i];

            println!("obtained widget {} ", i.to_string());

            let topleft = widget.widget.screen().geometry().top_left();

            println!("setting geometry for {}", i.to_string());

            widget.widget.set_geometry_4a(topleft.x(), widget.widget.y(), widget.widget.width(), widget.widget.height());

            println!("animating entry for {}", i.to_string());
            widget.animate_entry_signal.emit(i as i32);

            println!("animated entry for {}", i.to_string());
        }
    }

    #[slot(SlotOfInt)]
    unsafe fn on_action(self: &Rc<Self>, notifcation_id: i32) {

    }

    #[slot(SlotOfQString)]
    unsafe fn on_widget_close(self : &Rc<Self>, closed_widget: cpp_core::Ref<QString>) {    
        let mut notification_widget_index: usize = 0;

        println!("trying to close widget {}", closed_widget.to_std_string());

        let list = (*self.widget_list.borrow()).clone();

        println!("cloned list successfully");

        for i in 0..&list.len() - 1 {
            let widget = &list[i];
            if widget.widget.win_id().to_string() == closed_widget.to_std_string() {
                notification_widget_index = i;
                break;
            }
        }


        println!("removing {} - {}", notification_widget_index.to_string(), closed_widget.to_std_string());

        (self.widget_list.borrow_mut()).remove(notification_widget_index);

        println!("successfully removed {} - {}, no panic!", notification_widget_index.to_string(), closed_widget.to_std_string());
        self.reorder();
    }

    pub fn iter(&self) -> VecRefWrapper<Rc<notifications::NotificationWidget>> {
        VecRefWrapper { r: self.widget_list.borrow_mut() }
    }
}