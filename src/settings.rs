use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cpp_core::CppBox;
use lazy_static::lazy_static;
use qt_core::{qs, QBox, QPtr, QSettings, QVariant};
use qt_gui::{QGuiApplication, QScreen};

lazy_static! {
    static ref THEME: Mutex<String> = Mutex::new("default".to_string());
    static ref SCREEN: Mutex<i32> = Mutex::new(-1);
    static ref DO_NOT_DISTURB: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

static mut QSETTINGS: Option<QBox<QSettings>> = None;

pub static mut SETTINGS: Settings = Settings {
    theme: Theme { name: "" },
    screen: Screen {
        id: -1,
        name: "",
        qscreen: None,
    },
    do_not_disturb: DoNotDisturb { value: false },
};

pub trait Setting {
    fn load(&mut self);
    fn set(&mut self, value: CppBox<QVariant>);
    fn save(&mut self);
}

pub struct Settings {
    pub theme: Theme,
    pub screen: Screen,
    pub do_not_disturb: DoNotDisturb,
}

pub unsafe fn load_settings() {
    unsafe {
        QSETTINGS = Some(QSettings::new());
    }

    let mut theme = Theme { name: "default" };
    let mut screen = Screen {
        name: "",
        id: -1,
        qscreen: None,
    };

    theme.load();
    screen.load();

    let do_not_disturb = DoNotDisturb { value: false };

    let this = Settings {
        theme,
        screen,
        do_not_disturb,
    };

    SETTINGS = this;
}

pub struct Theme {
    pub name: &'static str,
}

impl Setting for Theme {
    fn load(&mut self) {
        unsafe {
            let theme_setting = QSETTINGS.as_ref().unwrap().value_1a(&qs("theme"));

            self.set(theme_setting);
        }
    }

    fn set(&mut self, value: CppBox<QVariant>) {
        unsafe {
            if value.is_null() {
                self.name = "default";
            } else {
                self.name = Box::leak(value.to_string().to_std_string().into_boxed_str());
            }

            let mut _theme = THEME.lock().expect("Could not lock mutex");

            *_theme = self.name.to_string();
        }
    }

    fn save(&mut self) {
        unsafe {
            QSETTINGS.as_ref().unwrap().set_value(
                &qs("theme"),
                &QVariant::from_q_string(&qs(self.name.clone())),
            );
        }
    }
}

pub struct Screen {
    pub id: i32,
    pub name: &'static str,
    pub qscreen: Option<QPtr<QScreen>>,
}

impl Setting for Screen {
    fn load(&mut self) {
        unsafe {
            let screen_setting = QSETTINGS.as_ref().unwrap().value_1a(&qs("screen"));

            self.set(screen_setting);
        }
    }

    fn set(&mut self, value: CppBox<QVariant>) {
        unsafe {
            let screens = QGuiApplication::screens();
            let mut screen_id = -1;
            let mut screen_name = String::new();
            let mut qscreen = Some(screens.value_1a(0));

            if !value.is_null() {
                for i in 0..screens.length() {
                    let screen = screens.value_1a(i);

                    if screen.name().compare_q_string(&value.to_string()) == 0 {
                        screen_id = i;
                        screen_name = screen.name().to_std_string();
                        qscreen = Some(screen);
                    }
                }
            }

            self.name = Box::leak(screen_name.into_boxed_str());
            self.id = screen_id;
            self.qscreen = qscreen;

            let mut _screen = SCREEN.lock().expect("Could not lock screen mutex");

            *_screen = self.id.clone();
        }
    }

    fn save(&mut self) {
        unsafe {
            QSETTINGS.as_ref().unwrap().set_value(
                &qs("screen"),
                &QVariant::from_q_string(&qs(self.name.clone())),
            );
        }
    }
}

pub struct DoNotDisturb {
    pub value: bool,
}

impl Setting for DoNotDisturb {
    fn load(&mut self) {
        self.value = false;
    }

    fn set(&mut self, value: CppBox<QVariant>) {
        unsafe {
            self.value = value.to_bool();
            DO_NOT_DISTURB.store(value.to_bool(), Ordering::Relaxed);
        }
    }

    fn save(&mut self) {
        // Should I actually save the Do Not Disturb?
    }
}
