use cpp_core::CppBox;
use qt_core::q_dir::Filter;
use qt_core::{qs, QBox, QDir, QDirIterator, QString, QVariant};
use qt_gui::{QGuiApplication, QIcon};
use qt_widgets::{QActionGroup, QApplication, QMenu, QSystemTrayIcon, SlotOfQAction};

use crate::settings::Setting;
use crate::SETTINGS;

pub struct MenuItem {
    label: CppBox<QString>,
    value: CppBox<QVariant>,
}

pub fn get_available_themes() -> Vec<MenuItem> {
    let mut values: Vec<MenuItem> = Vec::new();
    unsafe {
        let theme_directories = QDirIterator::from_q_string_q_flags_filter(
            &qs("./res/themes"),
            Filter::AllDirs | Filter::NoDotAndDotDot,
        );

        while theme_directories.has_next() {
            let theme = QDir::new_1a(&theme_directories.next());

            values.push(MenuItem {
                label: theme.dir_name(),
                value: QVariant::from_q_string(theme.dir_name().as_ref()),
            })
        }
    }

    values
}

pub fn get_available_screens() -> Vec<MenuItem> {
    let mut values: Vec<MenuItem> = Vec::new();
    unsafe {
        let screens = QGuiApplication::screens();

        for i in 0..screens.length() {
            values.push(MenuItem {
                label: screens.value_1a(i).name(),
                value: QVariant::from_int(i),
            })
        }
    }

    values
}

pub unsafe fn generate_tray() -> (QBox<QSystemTrayIcon>, QBox<QMenu>) {
    let tray_icon = QSystemTrayIcon::new();

    tray_icon.set_icon(&QIcon::from_theme_1a(&qs("notifications")));

    tray_icon.show();

    let tray_menu = QMenu::new();

    let theme_menu = tray_menu.add_menu_q_string(&qs("Themes"));

    let theme_action_group = QActionGroup::new(&theme_menu);
    theme_action_group.set_exclusive(true);

    let theme_directories = get_available_themes();

    for theme in theme_directories {
        let theme = theme.label;

        let theme_action = theme_menu.add_action_q_string(&theme);

        theme_action.set_object_name(&qs("set_theme"));
        theme_action.set_checkable(true);
        theme_action.set_data(&QVariant::from_q_string(&theme));

        if SETTINGS.theme.name == theme.to_std_string() {
            theme_action.set_checked(true);
        }

        theme_action_group.add_action_q_action(theme_action.as_ptr());
    }

    let screens_menu = tray_menu.add_menu_q_string(&qs("Screen"));

    let screens_action_group = QActionGroup::new(&screens_menu);
    screens_action_group.set_exclusive(true);

    for screen in get_available_screens() {
        let screen_action = screens_menu.add_action_q_string(&screen.label);

        screen_action.set_object_name(&qs("set_screen"));
        screen_action.set_checkable(true);
        screen_action.set_data(&screen.value);

        if SETTINGS.screen.name == screen.label.to_std_string() {
            screen_action.set_checked(true);
        }

        screens_action_group.add_action_q_action(screen_action.as_ptr());
    }

    if screens_action_group.checked_action().is_null() {
        screens_action_group.actions().value_1a(0).set_checked(true);
    }

    let do_not_disturb_action = tray_menu.add_action_q_string(&qs("Do not disturb"));
    do_not_disturb_action.set_object_name(&qs("do_not_disturb_action"));
    do_not_disturb_action.set_checkable(true);

    let quit_action = tray_menu.add_action_q_string(&qs("Quit"));
    quit_action.set_object_name(&qs("quit_action"));

    tray_icon.set_context_menu(&tray_menu);

    let settings = &mut SETTINGS;

    let tray_icon_ptr = tray_icon.as_ptr();

    tray_menu
        .triggered()
        .connect(&SlotOfQAction::new(&tray_menu, move |action| {
            if action.object_name().to_std_string() == "quit_action".to_string() {
                QApplication::close_all_windows();
                tray_icon_ptr.hide();
            }

            if action.object_name().to_std_string() == "do_not_disturb_action".to_string() {
                settings
                    .do_not_disturb
                    .set(QVariant::from_bool(action.is_checked()));
            }

            if action.object_name().to_std_string() == "set_theme".to_string() {
                settings.theme.set(action.data());
                settings.theme.save();
            }

            if action.object_name().to_std_string() == "set_screen".to_string() {
                settings.screen.set(action.data());
                settings.screen.save();
            }
        }));

    (tray_icon, tray_menu)
}
