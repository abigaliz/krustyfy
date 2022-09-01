pub mod notifications {
    use std::rc::Rc;

    use cpp_core::{CppBox, Ptr, StaticUpcast, Ref};
    use device_query::{DeviceQuery, DeviceState, Keycode};

    use qt_core::{CursorShape, q_abstract_animation, QBox, QByteArray,
                  QObject, QParallelAnimationGroup, QPropertyAnimation, QRect, qs, QSequentialAnimationGroup,
                  QString, SignalNoArgs, SignalOfInt, SignalOfQString, slot, SlotNoArgs, SlotOfInt, WidgetAttribute, WindowType
    };
    use qt_gui::{QColor, QCursor, QPixmap};
    use qt_widgets::{QFrame,
                     QGraphicsBlurEffect, QGraphicsDropShadowEffect,
                     QHBoxLayout, QLabel, QPushButton, QStackedLayout, QVBoxLayout, QMainWindow, QDialog
    };

    const NOTIFICATION_HEIGHT: i32 = 143;
    const NOTIFICATION_WIDTH: i32 = 318;
    const ICON_SIZE: i32 = 25;
    const IMAGE_SIZE: i32 = 80;
    
    const NOTIFICATION_SPAWN_DURATION: i32 = 200;
    const NOTIFICATION_DURATION: i32 = 6500;
    const NOTIFICATION_EXIT_DURATION: i32 = 600;

    const DEFAULT_NOTIFICATION_BLUR_RADIUS: f64 = 1.0;
    const DISAPPEARING_NOTIFICATION_BLUR_RADIUS: i32 = 15;

    const DEFAULT_NOTIFICATION_OPACITY: f32 = 0.8;

    const HOVERED_NOTIFICATION_OPACITY: f32 = 0.2;
    const HOVERED_NOTIFICATION_BLUR_RADIUS: f64 = 10.0;
    
    #[derive(Debug)]
    pub struct NotificationWidget {
        pub widget: QBox<QDialog>,
        // Animations
        entry_animation: QBox<QPropertyAnimation>,
        exit_animation: QBox<QPropertyAnimation>,
        blur_animation: QBox<QPropertyAnimation>,
        exit_animation_group: QBox<QSequentialAnimationGroup>,
        parallel_animation: QBox<QParallelAnimationGroup>,
        // Content
        icon_label: QBox<QLabel>,
        app_name_label: QBox<QLabel>,
        image_label: QBox<QLabel>,
        title_label: QBox<QLabel>,
        body_label: QBox<QLabel>,
        close_signal: Ref<SignalOfQString>,
        pub animate_entry_signal: QBox<SignalOfInt>,
        blur_effect: QBox<QGraphicsBlurEffect>,
        action_button: QBox<QPushButton>,
        notification_id: u32,
        pub freeze_signal: QBox<SignalNoArgs>,
        pub unfreeze_signal: QBox<SignalNoArgs>,
        pub overlay: QBox<QDialog>,
        frame_shadow: QBox<QGraphicsDropShadowEffect>,
        action_signal: Ref<SignalOfInt>,
        guid: String,
    }

    impl StaticUpcast<QObject> for NotificationWidget {
        unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
            ptr.widget.as_ptr().static_upcast()
        }
    }

    impl NotificationWidget {
        pub fn new(
            close_signal: &QBox<SignalOfQString>, 
            action_signal: &QBox<SignalOfInt>, 
            notification_id: u32, 
            main_window: &QBox<QMainWindow>,
            guid: String) -> Rc<NotificationWidget> {
            unsafe {

                let topleft = main_window.screen().geometry().top_left();
                // Set the notification widget
                let widget = QDialog::new_1a(main_window);
                widget.set_object_name(&qs(&guid));

                // Set the default action overlay
                let overlay = QDialog::new_1a(&widget);
                overlay.set_object_name(&qs("overlay"));

                overlay.set_geometry_4a(topleft.x(), 0, 400, 400);

                overlay.set_window_flags(
                    WindowType::WindowStaysOnTopHint | 
                    WindowType::Tool | 
                    WindowType::FramelessWindowHint |
                    WindowType::X11BypassWindowManagerHint);

                overlay.set_attribute_1a( WidgetAttribute::WADeleteOnClose);

                overlay.set_window_opacity(0.0);

                let cursor = QCursor::new();
                cursor.set_shape(CursorShape::PointingHandCursor);

                overlay.set_cursor(cursor.as_ref());

                let overlay_layout = QStackedLayout::new();
                overlay_layout.set_object_name(&qs("overlay_layout"));
                overlay_layout.set_geometry(&QRect::from_4_int(0, 0, 400, 300));
                overlay_layout.set_margin(0);
                overlay_layout.set_spacing(0);
                overlay.set_layout(overlay_layout.as_ptr());
        
                let action_button = QPushButton::new();
                action_button.set_object_name(&qs("action_button"));
                action_button.set_geometry_4a(0, 0, 400, 300);
                
                overlay_layout.add_widget(&action_button);
                
                // Set flags
                widget.set_window_flags(
                    WindowType::WindowTransparentForInput |
                    WindowType::WindowStaysOnTopHint | 
                    WindowType::Tool | 
                    WindowType::FramelessWindowHint |
                    WindowType::X11BypassWindowManagerHint);

                widget.set_attribute_1a(WidgetAttribute::WATranslucentBackground);
                widget.set_attribute_1a( WidgetAttribute::WADeleteOnClose);

                let blur_effect = qt_widgets::QGraphicsBlurEffect::new_1a(&widget);
                blur_effect.set_object_name(&qs("blur_effect"));

                widget.set_graphics_effect(&blur_effect);
                blur_effect.set_blur_radius(0.0);


                widget.set_geometry_4a(topleft.x(), 0 - NOTIFICATION_HEIGHT, NOTIFICATION_WIDTH, NOTIFICATION_HEIGHT);

                widget.set_window_opacity(DEFAULT_NOTIFICATION_OPACITY as f64);

                // Set animations
                let y_property = QByteArray::new();
                y_property.add_assign_q_string(&qs("geometry"));

                let blur_radius_property = QByteArray::new();
                blur_radius_property.add_assign_q_string(&qs("blurRadius"));

                let opacity_property = QByteArray::new();
                opacity_property.add_assign_q_string(&qs("windowOpacity"));

                let entry_animation = QPropertyAnimation::new_2a(&widget, &y_property);
                entry_animation.set_object_name(&qs("entry_animation"));
                let exit_animation = QPropertyAnimation::new_2a(&widget, &opacity_property);
                exit_animation.set_object_name(&qs("exit_animation"));
                let blur_animation = QPropertyAnimation::new_2a(&blur_effect, &blur_radius_property);
                blur_animation.set_object_name(&qs("blur_animation"));
                let exit_animation_group = QSequentialAnimationGroup::new_1a(&widget);
                exit_animation_group.set_object_name(&qs("exit_animation_group"));
                let parallel_animation = QParallelAnimationGroup::new_1a(&widget);
                parallel_animation.set_object_name(&qs("parallel_animation"));


                // Set up layout
                let frame = QFrame::new_1a(&widget);
                frame.set_object_name(&qs("frame"));

                frame.set_geometry_4a(10, 10, 301, 121);
                frame.set_style_sheet(&qs("border-radius: 15px;background-color: black;border-style:none;"));

                frame.set_frame_shape(qt_widgets::q_frame::Shape::StyledPanel);
                frame.set_frame_shadow(qt_widgets::q_frame::Shadow::Raised);

                let frame_shadow = QGraphicsDropShadowEffect::new_1a(&frame);
                frame_shadow.set_object_name(&qs("frame_shadow"));

                frame_shadow.set_blur_radius(10.0);
                frame_shadow.set_x_offset(1.0);
                frame_shadow.set_y_offset(1.0);

                frame.set_graphics_effect(&frame_shadow);

                frame.set_line_width(6);
                frame.set_mid_line_width(3);

                let vertical_layout = QVBoxLayout::new_1a(&frame);
                vertical_layout.set_object_name(&qs("vertical_layout"));

                vertical_layout.set_geometry(&QRect::from_4_int(0, 0, 301, 121));
                vertical_layout.set_size_constraint(qt_widgets::q_layout::SizeConstraint::SetMaximumSize);
                vertical_layout.set_contents_margins_4a(0, 0, 0, 0);
                vertical_layout.set_stretch(1, 3);


                let title_layout = QHBoxLayout::new_0a();
                title_layout.set_object_name(&qs("title_layout"));

                title_layout.set_spacing(6);
                title_layout.set_size_constraint(qt_widgets::q_layout::SizeConstraint::SetMinAndMaxSize);
                title_layout.set_contents_margins_4a(2, 2, -1, -1);

                let body_layout = QHBoxLayout::new_0a();
                body_layout.set_object_name(&qs("body_layout"));

                body_layout.set_spacing(5);
                body_layout.set_size_constraint(qt_widgets::q_layout::SizeConstraint::SetNoConstraint);
                body_layout.set_contents_margins_4a(5, 2, -1, 5);
                body_layout.set_stretch(0, 1);

                let vertical_body_layout = QVBoxLayout::new_0a();
                vertical_body_layout.set_object_name(&qs("vertical_body_layout"));

                vertical_body_layout.set_stretch(1, 1);
                vertical_body_layout.set_spacing(2);
                vertical_body_layout.set_size_constraint(qt_widgets::q_layout::SizeConstraint::SetMinAndMaxSize);


                vertical_layout.add_layout_1a(&title_layout);
                vertical_layout.add_layout_1a(&body_layout);


                // Set up content
                let icon_label = QLabel::new();
                icon_label.set_object_name(&qs("icon_label"));
                icon_label.set_parent(&widget);

                icon_label.set_maximum_size_2a(ICON_SIZE, ICON_SIZE);

                icon_label.set_style_sheet(&qs("background-color:rgba(255, 255, 255, 0);border-style: none;"));

                let app_name_label = QLabel::new();
                app_name_label.set_object_name(&qs("app_name_label"));
                app_name_label.set_parent(&widget);

                app_name_label.set_text_format(qt_core::TextFormat::MarkdownText);
                app_name_label.set_maximum_size_2a(350, 30);
                app_name_label.set_style_sheet(&qs("background-color:rgba(255, 255, 255, 0);border-style: none;"));

                let image_label = QLabel::new();
                image_label.set_object_name(&qs("image_label"));
                image_label.set_parent(&widget);

                image_label.set_maximum_size_2a(IMAGE_SIZE, IMAGE_SIZE);
                image_label.set_minimum_size_2a(IMAGE_SIZE, IMAGE_SIZE);
                image_label.set_style_sheet(&qs("background-color:rgba(255, 255, 255, 0); border-style: none; border-radius: 20px;"));

                let title_label = QLabel::new();
                title_label.set_object_name(&qs("title_label"));
                title_label.set_parent(&widget);

                title_label.set_maximum_size_2a(300, 20);
                title_label.set_text_format(qt_core::TextFormat::MarkdownText);


                let body_label = QLabel::new();
                body_label.set_object_name(&qs("body_label"));
                body_label.set_parent(&widget);

                body_label.set_alignment(
                    qt_core::AlignmentFlag::AlignLeading | 
                    qt_core::AlignmentFlag::AlignLeft | 
                    qt_core::AlignmentFlag::AlignTop);

                body_label.set_minimum_size_2a(0, 50);
                body_label.set_maximum_size_2a(350, 50);
                body_label.set_text_format(qt_core::TextFormat::MarkdownText);
                body_label.set_scaled_contents(false);
                body_label.set_word_wrap(true);

                title_layout.add_widget(&icon_label);
                title_layout.add_widget(&app_name_label);

                body_layout.add_widget(&image_label);
                body_layout.add_layout_1a(&vertical_body_layout);


                vertical_body_layout.add_widget(&title_label);
                vertical_body_layout.add_widget(&body_label);

                let animate_entry_signal = SignalOfInt::new();
                let freeze_signal = SignalNoArgs::new();
                let unfreeze_signal = SignalNoArgs::new();
            
                widget.show();
                overlay.show();

                let close = close_signal.as_ref().unwrap();
                let action = action_signal.as_ref().unwrap();

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
                    action_signal: action,
                    action_button,
                    notification_id,
                    freeze_signal,
                    unfreeze_signal,
                    overlay,
                    frame_shadow,
                    guid
                });
                this.init();
                this.animate_exit();
                this
            }
        }

        pub unsafe fn set_content(self: &Rc<Self>, app_name: CppBox<QString>, title: CppBox<QString>, body: CppBox<QString>, icon: CppBox<QPixmap>) {
            self.app_name_label.set_text(&app_name);
            self.body_label.set_text(&body);
            self.title_label.set_text(&title); 
            self.icon_label.set_pixmap(&icon);   
            
            self.image_label.set_maximum_size_2a(0, IMAGE_SIZE);
            self.image_label.set_minimum_size_2a(0, IMAGE_SIZE);
        }

        pub unsafe fn set_content_with_image(self: &Rc<Self>, app_name: CppBox<QString>, title: CppBox<QString>, body: CppBox<QString>, image: CppBox<QPixmap>, icon: CppBox<QPixmap>) {
            self.app_name_label.set_text(&app_name);
            self.body_label.set_text(&body);
            self.title_label.set_text(&title); 

            let scaled_image = image.scaled_2_int(IMAGE_SIZE, IMAGE_SIZE);

            self.image_label.set_pixmap(&scaled_image);  
            self.icon_label.set_pixmap(&icon);       
        }

        #[slot(SlotNoArgs)]
        pub unsafe fn check_hover(self: &Rc<Self>) {
            let device_state = DeviceState::new();

            let keys: Vec<Keycode> = device_state.get_keys();

            if keys.contains(&Keycode::LAlt) {
                self.freeze_signal.emit();
            } else {
                self.unfreeze_signal.emit();
            }

            let pos = QCursor::pos_0a();

            if self.widget.geometry().contains_q_point(pos.as_ref()) {
                self.hover();
            }
            else {
                self.unhover();
            }
        }
        
        pub unsafe fn hover(self: &Rc<Self>) {
            if self.overlay.is_visible() {
                self.blur_effect.set_blur_radius(DEFAULT_NOTIFICATION_BLUR_RADIUS);
                self.widget.set_window_opacity(1.0);
                self.frame_shadow.set_blur_radius(15.0);
                self.frame_shadow.set_color(QColor::from_3_int(255, 255, 255).as_ref());
                self.frame_shadow.set_offset_2_double(0.0, 0.0);
            }
            else if self.exit_animation.state() != q_abstract_animation::State::Running {
                self.widget.set_window_opacity(HOVERED_NOTIFICATION_OPACITY as f64);
                self.exit_animation.set_start_value(&qt_core::QVariant::from_float(HOVERED_NOTIFICATION_OPACITY));
                self.blur_effect.set_blur_radius(HOVERED_NOTIFICATION_BLUR_RADIUS);
                self.blur_animation.set_start_value(&qt_core::QVariant::from_double(HOVERED_NOTIFICATION_BLUR_RADIUS));
            }
            
        }

        pub unsafe fn unhover(self: &Rc<Self>) {
            if self.overlay.is_visible() {
                self.blur_effect.set_blur_radius(DEFAULT_NOTIFICATION_BLUR_RADIUS);
                self.widget.set_window_opacity(DEFAULT_NOTIFICATION_OPACITY as f64);
                
            } else if self.exit_animation.state() != q_abstract_animation::State::Running {
                self.widget.set_window_opacity(DEFAULT_NOTIFICATION_OPACITY as f64);
                self.exit_animation.set_start_value(&qt_core::QVariant::from_float(DEFAULT_NOTIFICATION_OPACITY));
                self.blur_effect.set_blur_radius(DEFAULT_NOTIFICATION_BLUR_RADIUS);
                self.blur_animation.set_start_value(&qt_core::QVariant::from_double(DEFAULT_NOTIFICATION_BLUR_RADIUS));
            }

            self.frame_shadow.set_blur_radius(10.0);
            self.frame_shadow.set_color(QColor::from_3_int(0, 0, 0).as_ref());
            self.frame_shadow.set_offset_2_double(1.0, 1.0);
            
        }

        #[slot(SlotOfInt)]
        pub unsafe fn animate_entry(self: &Rc<Self>, i: i32) {
            self.entry_animation.set_duration(NOTIFICATION_SPAWN_DURATION);

            let g = self.widget.geometry();

            let start_value = QRect::from_4_int(g.left(), g.top(), g.width(), g.height());
            let end_value = QRect::from_4_int(g.left(), NOTIFICATION_HEIGHT * i, g.width(), g.height());

            self.entry_animation.set_start_value(&qt_core::QVariant::from_q_rect(&start_value));
            self.entry_animation.set_end_value(&qt_core::QVariant::from_q_rect(&end_value));
            self.entry_animation.start_0a();
        }

        #[slot(SlotNoArgs)]
        unsafe fn animate_exit(self: &Rc<Self>) {
            self.exit_animation.set_duration(NOTIFICATION_EXIT_DURATION);
            self.exit_animation.set_start_value(&qt_core::QVariant::from_float(DEFAULT_NOTIFICATION_OPACITY));
            self.exit_animation.set_end_value(&qt_core::QVariant::from_float(0.0));
            self.exit_animation.set_easing_curve(&qt_core::QEasingCurve::new_1a(qt_core::q_easing_curve::Type::OutCurve));

            self.blur_animation.set_duration(NOTIFICATION_EXIT_DURATION);
            self.blur_animation.set_start_value(&qt_core::QVariant::from_double(DEFAULT_NOTIFICATION_BLUR_RADIUS));
            self.blur_animation.set_end_value(&qt_core::QVariant::from_int(DISAPPEARING_NOTIFICATION_BLUR_RADIUS));

            self.parallel_animation.add_animation(&self.blur_animation);
            self.parallel_animation.add_animation(&self.exit_animation);

            self.exit_animation_group.add_pause(NOTIFICATION_DURATION);
            self.exit_animation_group.add_animation(&self.parallel_animation);

            self.exit_animation_group.start_0a();

            self.exit_animation_group.finished().connect(&self.slot_on_close());
        }

        unsafe fn init(self: &Rc<Self>) {
            self.animate_entry_signal.connect(&self.slot_animate_entry());
            self.action_button.clicked().connect(&self.slot_on_button_clicked());
            self.freeze_signal.connect(&self.slot_on_freeze());
            self.unfreeze_signal.connect(&self.slot_on_unfreeze());
        }

        #[slot(SlotNoArgs)]
        unsafe fn on_close(self: &Rc<Self>) {
            self.close_signal.emit(&qs(&self.guid));
        }

        #[slot(SlotNoArgs)]
        unsafe fn on_freeze(self: &Rc<Self>) {
            self.overlay.set_geometry_1a(self.widget.geometry());
            self.overlay.set_visible(true);
            if self.exit_animation_group.state() == q_abstract_animation::State::Paused {
                return;
            }
            self.exit_animation_group.pause();
            self.blur_effect.set_blur_radius(DEFAULT_NOTIFICATION_BLUR_RADIUS);
            self.widget.set_window_opacity(DEFAULT_NOTIFICATION_OPACITY as f64);
        }

        #[slot(SlotNoArgs)]
        unsafe fn on_unfreeze(self: &Rc<Self>) {
            self.overlay.set_visible(false);
            if self.exit_animation_group.state() != q_abstract_animation::State::Paused {
                return;
            }
            self.exit_animation_group.resume();
        }

        #[slot(SlotNoArgs)]
        unsafe fn on_button_clicked(self: &Rc<Self>) {
            println!("Clicked");

            self.action_signal.emit(self.notification_id as i32);
            self.on_close();
            print!("wanting to close {}", self.widget.win_id().to_string());
        }
    }
}