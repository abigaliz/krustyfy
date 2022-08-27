use std::{rc::Rc, collections::HashMap, slice::Iter, borrow::Borrow, cell::RefMut};

use async_std::channel::Receiver;
use cpp_core::{CppBox, StaticUpcast, Ptr, };
use qt_core::{QBox, SignalNoArgs, QTimer, qs, QString, slot, SlotNoArgs, QEvent, QObject, SlotOfQString, QPtr, SignalOfQString};
use qt_gui::{QCloseEvent, SlotOfQWindow, QWindow, SignalOfQWindow};
use qt_widgets::{QWidget, QDialog, SlotOfQWidgetQWidget};
use signals2::Connect12;
use zbus::zvariant::Value;
use std::cell::{RefCell, Ref};

use crate::{notification_widget::notifications::{self, NotificationWidget}, notification::Notification};

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
    pub fn new() -> Rc<NotificationSpawner> {
        unsafe {
            let qobject = QObject::new_0a();

            let widget_list = RefCell::new(Vec::new());
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

    pub unsafe fn spawn_notification(self: &Rc<Self>, app_name: String, replaces_id: u32, app_icon: String, summary: String, body: String, actions: Vec<String>,
        expire_timeout: i32,) {

        let mut iter = self.iter();

        let close_signal = SignalOfQString::new();

        close_signal.connect(&self.slot_on_widget_close());
        
        let _notification_widget = NotificationWidget::new(close_signal);

        _notification_widget.set_content(qs(app_name), qs(summary), qs(body));
        _notification_widget.animate_entry(iter.r.len() as i32);
        

        iter.r.push(_notification_widget);
        self.reorder(iter.into_iter().enumerate());
    }

    unsafe fn reorder(self : &Rc<Self>, iter: std::iter::Enumerate<std::slice::Iter<'_, Rc<NotificationWidget>>>) {
        //let iter = self.iter();

        for (i, widget) in iter {
            let topleft = widget.widget.screen().geometry().top_left();

            widget.widget.set_geometry_4a(topleft.x(), widget.widget.y(), widget.widget.width(), widget.widget.height());
            widget.animate_entry(i as i32);
        }
    }

    #[slot(SlotOfQString)]
    unsafe fn on_widget_close(self : &Rc<Self>, closed_widget: cpp_core::Ref<QString>) {

        let mut iter = self.iter();

        let enumerator = iter.into_iter().enumerate();
        
        let mut notification_widget_index: usize = 0;

        for (i, widget) in enumerator {
            if (widget.widget.win_id().to_string() == closed_widget.to_std_string()) {
                notification_widget_index = i;
            }
        }

        iter.r.remove(notification_widget_index);
        self.reorder(iter.into_iter().enumerate());
    }

    pub fn iter(&self) -> VecRefWrapper<Rc<notifications::NotificationWidget>> {
        VecRefWrapper { r: self.widget_list.borrow_mut() }
    }
}