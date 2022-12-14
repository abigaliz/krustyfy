pub mod notifications {
    use std::ffi::{CStr, CString};
    use std::{cell::RefCell, rc::Rc};

    use cpp_core::{CppBox, CppDeletable, Ptr, Ref, StaticUpcast};
    use device_query::{DeviceQuery, DeviceState, Keycode};

    use crate::errors::KrustifyError;
    use crate::settings::SETTINGS;
    use qt_core::{
        q_abstract_animation, q_io_device::OpenModeFlag, qs, slot, AspectRatioMode, ConnectionType,
        GlobalColor, QBox, QByteArray, QEasingCurve, QFile, QFlags, QObject,
        QParallelAnimationGroup, QPropertyAnimation, QPtr, QRect, QSequentialAnimationGroup,
        QString, QVariant, SignalNoArgs, SignalOfInt, SignalOfQString, SlotNoArgs, SlotOfInt,
        TextElideMode, TransformationMode, WidgetAttribute, WindowType,
    };
    use qt_gui::{q_painter::RenderHint, QColor, QCursor, QPainter, QPainterPath, QPixmap};
    use qt_widgets::{
        QDialog, QFrame, QGraphicsBlurEffect, QGraphicsDropShadowEffect, QGraphicsOpacityEffect,
        QLabel, QPushButton, QStackedLayout, QWidget,
    };

    #[derive(Debug)]
    pub struct NotificationWidget {
        pub widget: QBox<QWidget>,
        // Animations
        entry_animation: QBox<QPropertyAnimation>,
        exit_animation: QBox<QPropertyAnimation>,
        blur_animation: QBox<QPropertyAnimation>,
        exit_animation_group: QBox<QSequentialAnimationGroup>,
        parallel_animation: QBox<QParallelAnimationGroup>,
        // Content
        icon_label: QPtr<QLabel>,
        app_name_label: QPtr<QLabel>,
        image_label: QPtr<QLabel>,
        title_label: QPtr<QLabel>,
        body_label: QPtr<QLabel>,
        close_signal: Ref<SignalOfQString>,
        pub animate_entry_signal: QBox<SignalOfInt>,
        blur_effect: QBox<QGraphicsBlurEffect>,
        opacity_effect: QBox<QGraphicsOpacityEffect>,
        action_button: QPtr<QPushButton>,
        pub notification_id: RefCell<u32>,
        pub overlay: QBox<QDialog>,
        frame_shadow: QBox<QGraphicsDropShadowEffect>,
        action_signal: Ref<SignalOfInt>,
        guid: String,
        parallel_hover_animation: QBox<QParallelAnimationGroup>,
        default_opacity: CppBox<QVariant>,
        default_blur: CppBox<QVariant>,
        end_blur: CppBox<QVariant>,
        notification_duration: CppBox<QVariant>,
        spawn_duration: CppBox<QVariant>,
        disappear_duration: CppBox<QVariant>,
        default_shadow_color: CppBox<QVariant>,
        focused_shadow_color: CppBox<QVariant>,
        pub close_reason: RefCell<u32>,
    }

    impl StaticUpcast<QObject> for NotificationWidget {
        unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
            ptr.widget.as_ptr().static_upcast()
        }
    }

    impl NotificationWidget {
        pub fn new(
            main_window: &QBox<QFrame>,
            close_signal: &QBox<SignalOfQString>,
            action_signal: &QBox<SignalOfInt>,
            _notification_id: u32,
            guid: String,
        ) -> Result<Rc<NotificationWidget>, KrustifyError> {
            unsafe {
                // Set the notification widget
                let widget = QWidget::new_1a(main_window);

                widget.set_object_name(&qs(&guid));

                // Set flags

                widget.set_attribute_1a(WidgetAttribute::WATranslucentBackground);
                widget.set_attribute_1a(WidgetAttribute::WADeleteOnClose);
                widget.set_attribute_1a(WidgetAttribute::WANoSystemBackground);

                let widget_layout = QStackedLayout::new();

                widget.set_layout(widget_layout.as_ptr());

                let theme = &SETTINGS.theme.name;

                let template_file =
                    QFile::from_q_string(&qs(format!("./res/themes/{theme}/template.ui")));
                template_file.open(QFlags::from(OpenModeFlag::ReadOnly));
                let loader = qt_ui_tools::QUiLoader::new_1a(&widget);
                let template = loader.load_1a(template_file.as_ptr());
                template_file.reset();
                template_file.close();
                template_file.delete();
                loader.delete();

                // Load properties
                let default_opacity =
                    template.property(CStr::as_ptr(&CString::new("defaultOpacity")?));
                let hovered_opacity =
                    template.property(CStr::as_ptr(&CString::new("hoveredOpacity")?));
                let default_blur = template.property(CStr::as_ptr(&CString::new("defaultBlur")?));
                let hovered_blur = template.property(CStr::as_ptr(&CString::new("hoveredBlur")?));
                let end_blur = template.property(CStr::as_ptr(&CString::new("endBlur")?));
                let notification_duration =
                    template.property(CStr::as_ptr(&CString::new("notificationDuration")?));
                let spawn_duration =
                    template.property(CStr::as_ptr(&CString::new("spawnDuration")?));
                let disappear_duration =
                    template.property(CStr::as_ptr(&CString::new("disappearDuration")?));
                let default_shadow_color =
                    template.property(CStr::as_ptr(&CString::new("defaultShadowColor")?));
                let focused_shadow_color =
                    template.property(CStr::as_ptr(&CString::new("focusedShadowColor")?));
                let text_shadow_color =
                    template.property(CStr::as_ptr(&CString::new("textShadowColor")?));

                let notification: QPtr<QWidget> = template.find_child("notification")?;

                widget.layout().add_widget(&notification);

                let overlay_widget: QPtr<QWidget> = template.find_child("overlay")?;
                // Set the default action overlay
                let overlay = QDialog::new_1a(&widget);
                overlay.set_object_name(&qs("overlay"));

                overlay.set_window_flags(
                    WindowType::WindowStaysOnTopHint
                        | WindowType::Tool
                        | WindowType::FramelessWindowHint
                        | WindowType::BypassWindowManagerHint,
                );

                overlay.set_attribute_1a(WidgetAttribute::WADeleteOnClose);

                overlay.set_window_opacity(0.0);

                let overlay_layout = QStackedLayout::new();
                overlay_layout.set_object_name(&qs("overlay_layout"));
                overlay.set_layout(overlay_layout.as_ptr());
                overlay_layout.add_widget(&overlay_widget);

                let action_button: QPtr<QPushButton> = overlay_widget.find_child("pushButton")?;

                let blur_effect = QGraphicsBlurEffect::new_1a(&widget);
                blur_effect.set_object_name(&qs("blur_effect"));

                let opacity_effect = QGraphicsOpacityEffect::new_1a(&notification);
                opacity_effect.set_object_name(&qs("opacity_effect"));

                widget.set_graphics_effect(&blur_effect);
                notification.set_graphics_effect(&opacity_effect);

                opacity_effect.set_opacity(default_opacity.to_double_0a());
                blur_effect.set_blur_radius(default_blur.to_double_0a());

                widget.set_geometry_4a(
                    0,
                    0 - notification.geometry().height(),
                    notification.geometry().width(),
                    notification.geometry().height(),
                );

                widget.set_window_opacity(default_opacity.to_double_0a());

                // Set animations
                let y_property = QByteArray::new();
                y_property.add_assign_q_string(&qs("geometry"));

                let blur_radius_property = QByteArray::new();
                blur_radius_property.add_assign_q_string(&qs("blurRadius"));

                let opacity_property = QByteArray::new();
                opacity_property.add_assign_q_string(&qs("opacity"));

                let entry_animation = QPropertyAnimation::new_2a(&widget, &y_property);
                entry_animation.set_object_name(&qs("entry_animation"));
                let exit_animation = QPropertyAnimation::new_2a(&opacity_effect, &opacity_property);
                exit_animation.set_object_name(&qs("exit_animation"));
                let blur_animation =
                    QPropertyAnimation::new_2a(&blur_effect, &blur_radius_property);
                blur_animation.set_object_name(&qs("blur_animation"));
                let blur_hover_animation =
                    QPropertyAnimation::new_2a(&blur_effect, &blur_radius_property);
                blur_hover_animation.set_object_name(&qs("blur_hover_animation"));
                let opacity_hover_animation =
                    QPropertyAnimation::new_2a(&opacity_effect, &opacity_property);
                opacity_hover_animation.set_object_name(&qs("opacity_hover_animation"));
                let exit_animation_group = QSequentialAnimationGroup::new_1a(&widget);
                exit_animation_group.set_object_name(&qs("exit_animation_group"));
                let parallel_animation = QParallelAnimationGroup::new_1a(&widget);
                parallel_animation.set_object_name(&qs("parallel_animation"));
                let parallel_hover_animation = QParallelAnimationGroup::new_1a(&widget);
                parallel_hover_animation.set_object_name(&qs("parallel_hover_animation"));

                blur_hover_animation.set_start_value(&default_blur);
                blur_hover_animation.set_end_value(&hovered_blur);
                blur_hover_animation.set_duration(100);
                opacity_hover_animation.set_start_value(&default_opacity);
                opacity_hover_animation.set_end_value(&hovered_opacity);
                opacity_hover_animation.set_duration(100);

                parallel_hover_animation.add_animation(&blur_hover_animation);
                parallel_hover_animation.add_animation(&opacity_hover_animation);

                let frame: QPtr<QFrame> = widget.find_child("notificationFrame")?;

                let frame_shadow = QGraphicsDropShadowEffect::new_1a(&frame);
                frame_shadow.set_object_name(&qs("frame_shadow"));

                frame_shadow.set_blur_radius(10.0);
                frame_shadow.set_x_offset(1.0);
                frame_shadow.set_y_offset(1.0);

                frame.set_graphics_effect(&frame_shadow);

                // Set up content
                let icon_label: QPtr<QLabel> =
                    widget.find_child("iconLabel").unwrap_or(QPtr::null());
                let app_name_label: QPtr<QLabel> =
                    widget.find_child("appNameLabel").unwrap_or(QPtr::null());

                if !app_name_label.is_null() {
                    let app_name_label_shadow = QGraphicsDropShadowEffect::new_1a(&app_name_label);
                    app_name_label_shadow.set_object_name(&qs("app_name_label_shadow"));

                    app_name_label_shadow.set_blur_radius(1.0);
                    app_name_label_shadow.set_x_offset(0.0);
                    app_name_label_shadow.set_y_offset(0.0);
                    app_name_label_shadow
                        .set_color(&QColor::from_q_string(&text_shadow_color.to_string()));

                    app_name_label.set_graphics_effect(&app_name_label_shadow);
                }

                let image_label: QPtr<QLabel> =
                    widget.find_child("imageLabel").unwrap_or(QPtr::null());

                let title_label: QPtr<QLabel> =
                    widget.find_child("titleLabel").unwrap_or(QPtr::null());

                if !title_label.is_null() {
                    let title_label_shadow = QGraphicsDropShadowEffect::new_1a(&title_label);

                    title_label_shadow.set_object_name(&qs("title_label_shadow"));

                    title_label_shadow.set_blur_radius(1.0);
                    title_label_shadow.set_x_offset(0.0);
                    title_label_shadow.set_y_offset(0.0);
                    title_label_shadow
                        .set_color(&QColor::from_q_string(&text_shadow_color.to_string()));

                    title_label.set_graphics_effect(&title_label_shadow);
                }

                let body_label: QPtr<QLabel> =
                    widget.find_child("bodyLabel").unwrap_or(QPtr::null());

                if !body_label.is_null() {
                    let body_label_shadow = QGraphicsDropShadowEffect::new_1a(&body_label);
                    body_label_shadow.set_object_name(&qs("body_label_shadow"));

                    body_label_shadow.set_blur_radius(1.0);
                    body_label_shadow.set_x_offset(0.0);
                    body_label_shadow.set_y_offset(0.0);
                    body_label_shadow
                        .set_color(&QColor::from_q_string(&text_shadow_color.to_string()));

                    body_label.set_graphics_effect(&body_label_shadow);
                }

                let animate_entry_signal = SignalOfInt::new();

                widget.show();
                overlay.show();
                overlay.hide();

                // not sure about these ones
                let close = close_signal
                    .as_ref()
                    .expect("couldn't get reference to close signal");
                let action = action_signal
                    .as_ref()
                    .expect("coudln't get reference to action signal");

                let notification_id = RefCell::new(_notification_id);

                template.close();
                template.delete();

                let this = Rc::new(Self {
                    widget,
                    entry_animation,
                    exit_animation,
                    blur_animation,
                    exit_animation_group,
                    parallel_animation,
                    icon_label,
                    app_name_label,
                    image_label,
                    title_label,
                    body_label,
                    close_signal: close,
                    animate_entry_signal,
                    blur_effect,
                    opacity_effect,
                    action_signal: action,
                    action_button,
                    notification_id,
                    overlay,
                    frame_shadow,
                    guid,
                    parallel_hover_animation,
                    default_opacity,
                    default_blur,
                    end_blur,
                    notification_duration,
                    spawn_duration,
                    disappear_duration,
                    default_shadow_color,
                    focused_shadow_color,
                    close_reason: RefCell::new(1),
                });
                this.init();
                this.animate_exit();
                Ok(this)
            }
        }

        #[slot(SlotNoArgs)]
        unsafe fn ellide(self: &Rc<Self>) {
            if !self.title_label.is_null() {
                let ellided_title = self.title_label.font_metrics().elided_text_3a(
                    &self.title_label.text(),
                    TextElideMode::ElideRight,
                    self.title_label.width(),
                );

                self.title_label.set_text(&ellided_title);
            }
        }

        unsafe fn set_content(
            self: &Rc<Self>,
            app_name: CppBox<QString>,
            title: CppBox<QString>,
            body: CppBox<QString>,
            icon: CppBox<QPixmap>,
        ) {
            if !self.app_name_label.is_null() {
                self.app_name_label.set_text(&app_name);
            }

            if !self.body_label.is_null() {
                self.body_label.set_text(&body);
            }

            if !self.title_label.is_null() {
                self.title_label.set_text(&title);
            }

            if !self.icon_label.is_null() {
                let scaled_icon = icon.scaled_2_int_aspect_ratio_mode_transformation_mode(
                    self.icon_label.width(),
                    self.icon_label.height(),
                    AspectRatioMode::IgnoreAspectRatio,
                    TransformationMode::SmoothTransformation,
                );

                self.icon_label.set_pixmap(&scaled_icon);
            }

            let signal = SignalNoArgs::new();
            signal.connect_with_type(ConnectionType::QueuedConnection, &self.slot_ellide());
            signal.emit();
        }

        pub unsafe fn set_content_no_image(
            self: &Rc<Self>,
            app_name: CppBox<QString>,
            title: CppBox<QString>,
            body: CppBox<QString>,
            icon: CppBox<QPixmap>,
        ) {
            self.set_content(app_name, title, body, icon);
        }

        pub unsafe fn set_content_with_image(
            self: &Rc<Self>,
            app_name: CppBox<QString>,
            title: CppBox<QString>,
            body: CppBox<QString>,
            image: CppBox<QPixmap>,
            icon: CppBox<QPixmap>,
        ) {
            if !self.image_label.is_null() {
                let scaled_image = self.resize_image(image);

                self.image_label.set_pixmap(&scaled_image);

                self.image_label.set_maximum_size_2a(
                    self.image_label.maximum_height(),
                    self.image_label.maximum_height(),
                );
                self.image_label.set_minimum_size_2a(
                    self.image_label.maximum_height(),
                    self.image_label.maximum_height(),
                );
            }

            self.set_content(app_name, title, body, icon);
        }

        unsafe fn resize_image(self: &Rc<Self>, pixmap: CppBox<QPixmap>) -> CppBox<QPixmap> {
            let target = QPixmap::from_2_int(
                self.image_label.maximum_height(),
                self.image_label.maximum_height(),
            );

            target.fill_1a(&QColor::from_global_color(GlobalColor::Transparent));

            let painter = QPainter::new_1a(&target);

            painter.set_render_hints_2a(
                RenderHint::HighQualityAntialiasing
                    | RenderHint::SmoothPixmapTransform
                    | RenderHint::Antialiasing,
                true,
            );

            let path = QPainterPath::new_0a();
            path.add_round_rect_6a(
                0.0,
                0.0,
                self.image_label.maximum_height() as f64,
                self.image_label.maximum_height() as f64,
                25,
                25,
            );

            painter.set_clip_path_1a(&path);

            let scaled_pixmap = pixmap.scaled_2_int_aspect_ratio_mode_transformation_mode(
                self.image_label.maximum_height(),
                self.image_label.maximum_height(),
                AspectRatioMode::IgnoreAspectRatio,
                TransformationMode::SmoothTransformation,
            );

            painter.draw_pixmap_q_rect_q_pixmap(&target.rect(), &scaled_pixmap);

            target
        }

        pub unsafe fn reset_timer(self: &Rc<Self>) {
            self.exit_animation_group.set_current_time(0);
            self.exit_animation_group.start_0a();
        }

        #[slot(SlotNoArgs)]
        pub unsafe fn check_hover(self: &Rc<Self>) {
            let device_state = DeviceState::new();

            let keys: Vec<Keycode> = device_state.get_keys();

            if keys.contains(&Keycode::LAlt) {
                self.freeze();
            } else {
                self.unfreeze();
            }

            let rect = QRect::new();
            rect.set_x(self.widget.x() + self.widget.window().x());
            rect.set_y(self.widget.y() + self.widget.window().y());
            rect.set_width(self.widget.geometry().width());
            rect.set_height(self.widget.geometry().height());

            let pos = QCursor::pos_0a();

            if rect.contains_q_point(pos.as_ref()) {
                self.hover();
            } else {
                self.unhover();
            }
        }

        pub unsafe fn hover(self: &Rc<Self>) {
            if self.overlay.is_visible() {
                self.blur_effect
                    .set_blur_radius(self.default_blur.to_double_0a());
                self.opacity_effect.set_opacity(0.99); // For some reason setting it to 1.0 shifts the widget slightly to the bottom-right
                self.frame_shadow.set_blur_radius(15.0);

                let color = QColor::from_q_string(&self.focused_shadow_color.to_string());

                self.frame_shadow.set_color(&color);
                self.frame_shadow.set_offset_2_double(0.0, 0.0);
            } else if self.exit_animation.state() != q_abstract_animation::State::Running {
                self.parallel_hover_animation
                    .set_direction(q_abstract_animation::Direction::Forward);

                if self.parallel_hover_animation.state() == q_abstract_animation::State::Stopped
                    && self.parallel_hover_animation.current_time() == 0
                {
                    self.parallel_hover_animation.start_0a();
                }
            }
        }

        pub unsafe fn unhover(self: &Rc<Self>) {
            if self.overlay.is_visible() {
                self.blur_effect
                    .set_blur_radius(self.default_blur.to_double_0a());
                self.opacity_effect
                    .set_opacity(self.default_opacity.to_double_0a());
            } else if self.exit_animation.state() != q_abstract_animation::State::Running {
                if self.parallel_hover_animation.state() == q_abstract_animation::State::Stopped
                    && self.parallel_hover_animation.current_time() > 0
                {
                    self.parallel_hover_animation
                        .set_direction(q_abstract_animation::Direction::Backward);
                    self.parallel_hover_animation.start_0a();
                }
            }

            self.frame_shadow.set_blur_radius(10.0);

            let color = QColor::from_q_string(&self.default_shadow_color.to_string());

            self.frame_shadow.set_color(&color);
            self.frame_shadow.set_offset_2_double(1.0, 1.0);
        }

        #[slot(SlotOfInt)]
        pub unsafe fn animate_entry(self: &Rc<Self>, height: i32) {
            self.entry_animation
                .set_duration(self.spawn_duration.to_int_0a());

            let start_value = self.widget.geometry();
            let end_value = QRect::from_4_int(
                start_value.left(),
                height,
                start_value.width(),
                start_value.height(),
            );

            self.entry_animation
                .set_start_value(&QVariant::from_q_rect(start_value));
            self.entry_animation
                .set_end_value(&QVariant::from_q_rect(&end_value));
            self.entry_animation.start_0a();
        }

        #[slot(SlotNoArgs)]
        unsafe fn animate_exit(self: &Rc<Self>) {
            self.exit_animation
                .set_duration(self.disappear_duration.to_int_0a());
            self.exit_animation.set_start_value(&self.default_opacity);
            self.exit_animation
                .set_end_value(&QVariant::from_float(0.0));
            self.exit_animation.set_easing_curve(&QEasingCurve::new_1a(
                qt_core::q_easing_curve::Type::OutCurve,
            ));

            self.blur_animation
                .set_duration(self.disappear_duration.to_int_0a());
            self.blur_animation.set_start_value(&self.default_blur);
            self.blur_animation.set_end_value(&self.end_blur);

            self.parallel_animation.add_animation(&self.blur_animation);
            self.parallel_animation.add_animation(&self.exit_animation);

            self.exit_animation_group
                .add_pause(self.notification_duration.to_int_0a())
                .finished()
                .connect(&self.slot_on_init_exit());
            self.exit_animation_group
                .add_animation(&self.parallel_animation);

            self.exit_animation_group.start_0a();

            self.exit_animation_group
                .finished()
                .connect(&self.slot_on_close());
        }

        unsafe fn init(self: &Rc<Self>) {
            self.animate_entry_signal
                .connect(&self.slot_animate_entry());
            self.action_button
                .clicked()
                .connect(&self.slot_on_button_clicked());
        }

        #[slot(SlotNoArgs)]
        pub unsafe fn on_close(self: &Rc<Self>) {
            self.close_signal.emit(&qs(&self.guid));
        }

        #[slot(SlotNoArgs)]
        unsafe fn on_init_exit(self: &Rc<Self>) {
            self.parallel_hover_animation.stop();
            self.exit_animation
                .set_start_value(&qt_core::QVariant::from_double(
                    self.opacity_effect.opacity(),
                ));
            self.blur_animation
                .set_start_value(&qt_core::QVariant::from_double(
                    self.blur_effect.blur_radius(),
                ));
        }

        unsafe fn freeze(self: &Rc<Self>) {
            let rect = QRect::new();
            rect.set_x(self.widget.x() + self.widget.window().x());
            rect.set_y(self.widget.y() + self.widget.window().y());
            rect.set_width(self.widget.geometry().width());
            rect.set_height(self.widget.geometry().height());

            self.overlay.set_geometry_1a(rect.as_ref());
            self.action_button.set_geometry_4a(
                0,
                0,
                self.widget.geometry().width(),
                self.widget.geometry().height(),
            );
            if self.overlay.is_visible() {
                return;
            }
            self.overlay.set_visible(true);
            if self.exit_animation_group.state() == q_abstract_animation::State::Paused {
                return;
            }
            self.exit_animation_group.pause();
            self.blur_effect
                .set_blur_radius(self.default_blur.to_double_0a());
            self.opacity_effect
                .set_opacity(self.default_opacity.to_double_0a());
        }

        unsafe fn unfreeze(self: &Rc<Self>) {
            if !self.overlay.is_visible() {
                return;
            }
            self.overlay.set_visible(false);
            self.frame_shadow.set_blur_radius(10.0);

            let color = QColor::from_q_string(&self.default_shadow_color.to_string());

            self.frame_shadow.set_color(&color);
            self.frame_shadow.set_offset_2_double(1.0, 1.0);
            self.parallel_hover_animation.set_current_time(0);
            if self.exit_animation_group.state() != q_abstract_animation::State::Paused {
                return;
            }
            self.exit_animation_group.resume();
        }

        #[slot(SlotNoArgs)]
        unsafe fn on_button_clicked(self: &Rc<Self>) {
            let notification_id = self.notification_id.borrow().to_owned();
            self.action_signal.emit(notification_id as i32);
            self.on_close();
        }
    }
}
