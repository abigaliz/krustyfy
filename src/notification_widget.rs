pub mod notifications {

    use cpp_core::{Ptr, Ref, StaticUpcast, CppBox, NullPtr};
    use qt_core::{qs, slot, ContextMenuPolicy, QBox, QObject, 
        QPoint, SlotNoArgs, SlotOfInt, QPropertyAnimation, QSequentialAnimationGroup, QParallelAnimationGroup,
        WindowType, QByteArray, QRect, QString, WidgetAttribute, SlotOfQString, SignalOfQString, SignalOfInt
    };
    use qt_gui::{QPixmap, SignalOfQWindow};
    use qt_widgets::{
        QAction, QApplication, QLineEdit, QMenu, QMessageBox, QPushButton, QTableWidget,
        QTableWidgetItem, QVBoxLayout, QWidget, SlotOfQPoint, SlotOfQTableWidgetItemQTableWidgetItem,
        QFrame, QHBoxLayout, QLabel, QLayout, QGraphicsDropShadowEffect
    };
    
    use std::rc::Rc;

    const NOTIFICATION_HEIGHT: i32 = 143;
    const NOTIFICATION_WIDTH: i32 = 318;
    const ICON_SIZE: i32 = 25;
    const IMAGE_SIZE: i32 = 80;
    
    const NOTIFICATION_SPAWN_DURATION: i32 = 200;
    const NOTIFICATION_DURATION: i32 = 6500;
    const NOTIFICATION_EXIT_DURATION: i32 = 600;

    const DEFAULT_NOTIFICATION_BLUR_RADIUS: i32 = 1;
    const DISAPPEARING_NOTIFICATION_BLUR_RADIUS: i32 = 15;

    const DEFAULT_NOTIFICATION_OPACITY: f32 = 0.8;

    pub struct NotificationWidget {
        pub widget: QBox<QWidget>,
        // Animations
        entry_animation: QBox<QPropertyAnimation>,
        exit_animation: QBox<QPropertyAnimation>,
        blur_animation: QBox<QPropertyAnimation>,
        exit_animation_group: QBox<QSequentialAnimationGroup>,
        parallel_animation: QBox<QParallelAnimationGroup>,
        // Layout
        frame: QBox<QFrame>,
        vertical_layout: QBox<QVBoxLayout>,
        title_layout: QBox<QHBoxLayout>,
        body_layout: QBox<QHBoxLayout>,
        vertical_body_layout: QBox<QVBoxLayout>,
        // Content
        icon_label: QBox<QLabel>,
        app_name_label: QBox<QLabel>,
        image_label: QBox<QLabel>,
        title_label: QBox<QLabel>,
        body_label: QBox<QLabel>,
        close_signal: QBox<SignalOfQString>,
        pub animate_entry_signal: QBox<SignalOfInt>,
    }

    impl StaticUpcast<QObject> for NotificationWidget {
        unsafe fn static_upcast(ptr: Ptr<Self>) -> Ptr<QObject> {
            ptr.widget.as_ptr().static_upcast()
        }
    }

    impl NotificationWidget {
        pub fn new(close_signal: QBox<SignalOfQString>) -> Rc<NotificationWidget> {
            unsafe {
                let widget = QWidget::new_0a();
                
                // Set flags
                widget.set_window_flags(
                    WindowType::FramelessWindowHint |
                    WindowType::WindowTransparentForInput |
                    WindowType::WindowStaysOnTopHint |
                    WindowType::Tool |
                    WindowType::BypassWindowManagerHint);

                let blur_effect = qt_widgets::QGraphicsBlurEffect::new_1a(&widget);

                widget.set_graphics_effect(&blur_effect);
                blur_effect.set_blur_radius(1.0);

                widget.set_attribute_1a(WidgetAttribute::WANoSystemBackground);
                widget.set_attribute_1a(WidgetAttribute::WATranslucentBackground);

                widget.set_geometry_4a(0, 0 - NOTIFICATION_HEIGHT, NOTIFICATION_WIDTH, NOTIFICATION_HEIGHT);

                widget.set_window_opacity(DEFAULT_NOTIFICATION_OPACITY as f64);

                // Set animations
                let y_property = QByteArray::new();
                y_property.add_assign_q_string(&qs("geometry"));

                let blur_radius_property = QByteArray::new();
                blur_radius_property.add_assign_q_string(&qs("blurRadius"));

                let opacity_property = QByteArray::new();
                opacity_property.add_assign_q_string(&qs("windowOpacity"));

                let entry_animation = QPropertyAnimation::new_2a(&widget, &y_property);
                let exit_animation = QPropertyAnimation::new_2a(&widget, &opacity_property);
                let blur_animation = QPropertyAnimation::new_2a(&blur_effect, &blur_radius_property);
                let exit_animation_group = QSequentialAnimationGroup::new_1a(&widget);
                let parallel_animation = QParallelAnimationGroup::new_1a(&widget);


                // Set up layout
                let frame = QFrame::new_1a(&widget);

                frame.set_geometry_4a(10, 10, 301, 121);
                frame.set_style_sheet(&qs("border-radius: 15px;background-color: black;border-style:none;"));

                frame.set_frame_shape(qt_widgets::q_frame::Shape::StyledPanel);
                frame.set_frame_shadow(qt_widgets::q_frame::Shadow::Raised);

                let frame_shadow = QGraphicsDropShadowEffect::new_0a();

                frame_shadow.set_blur_radius(10.0);
                frame_shadow.set_x_offset(1.0);
                frame_shadow.set_y_offset(1.0);

                frame.set_graphics_effect(&frame_shadow);

                frame.set_line_width(6);
                frame.set_mid_line_width(3);

                let vertical_layout = QVBoxLayout::new_1a(&frame);

                vertical_layout.set_geometry(&QRect::from_4_int(0, 0, 301, 121));
                vertical_layout.set_size_constraint(qt_widgets::q_layout::SizeConstraint::SetMaximumSize);
                vertical_layout.set_contents_margins_4a(0, 0, 0, 0);
                vertical_layout.set_stretch(1, 3);


                let title_layout = QHBoxLayout::new_0a();

                title_layout.set_spacing(6);
                title_layout.set_size_constraint(qt_widgets::q_layout::SizeConstraint::SetMinAndMaxSize);
                title_layout.set_contents_margins_4a(2, 2, -1, -1);

                let body_layout = QHBoxLayout::new_0a();

                body_layout.set_spacing(5);
                body_layout.set_size_constraint(qt_widgets::q_layout::SizeConstraint::SetNoConstraint);
                body_layout.set_contents_margins_4a(5, 2, -1, 5);
                body_layout.set_stretch(0, 1);

                let vertical_body_layout = QVBoxLayout::new_0a();

                vertical_body_layout.set_stretch(1, 1);
                vertical_body_layout.set_spacing(2);
                vertical_body_layout.set_size_constraint(qt_widgets::q_layout::SizeConstraint::SetMinAndMaxSize);


                vertical_layout.add_layout_1a(&title_layout);
                vertical_layout.add_layout_1a(&body_layout);


                // Set up content
                let icon_label = QLabel::new();

                icon_label.set_maximum_size_2a(ICON_SIZE, ICON_SIZE);

                icon_label.set_style_sheet(&qs("background-color:rgba(255, 255, 255, 0);border-style: none;"));

                let app_name_label = QLabel::new();

                app_name_label.set_text_format(qt_core::TextFormat::MarkdownText);
                app_name_label.set_maximum_size_2a(350, 30);
                app_name_label.set_style_sheet(&qs("background-color:rgba(255, 255, 255, 0);border-style: none;"));

                let image_label = QLabel::new();

                image_label.set_maximum_size_2a(IMAGE_SIZE, IMAGE_SIZE);
                image_label.set_minimum_size_2a(0, IMAGE_SIZE);
                image_label.set_style_sheet(&qs("background-color:rgba(255, 255, 255, 0); border-style: none; border-radius: 20px;"));

                let title_label = QLabel::new();

                title_label.set_maximum_size_2a(300, 20);
                title_label.set_text_format(qt_core::TextFormat::MarkdownText);


                let body_label = QLabel::new();

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
            
                widget.show();

                let this = Rc::new(Self {
                    widget,
                    entry_animation,
                    exit_animation,
                    blur_animation,
                    exit_animation_group,
                    parallel_animation,
                    frame,
                    vertical_layout,
                    title_layout,
                    body_layout,
                    vertical_body_layout,
                    icon_label,
                    app_name_label,
                    image_label,
                    title_label,
                    body_label,
                    close_signal,
                    animate_entry_signal,
                });
                this.init();
                this.animate_exit();
                this
            }
        }

        pub unsafe fn set_content(self: &Rc<Self>, app_name: CppBox<QString>, title: CppBox<QString>, body: CppBox<QString>) {
            self.app_name_label.set_text(&app_name);
            self.body_label.set_text(&body);
            self.title_label.set_text(&title);            
        }

        pub unsafe fn set_content_with_image(self: &Rc<Self>, app_name: CppBox<QString>, title: CppBox<QString>, body: CppBox<QString>, icon: CppBox<QPixmap>, image: CppBox<QPixmap>) {
            self.app_name_label.set_text(&app_name);
            self.body_label.set_text(&body);
            self.title_label.set_text(&title);            
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
            self.blur_animation.set_start_value(&qt_core::QVariant::from_int(DEFAULT_NOTIFICATION_BLUR_RADIUS));
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
            //self.entry_animation.finished().connect(&self.slot_animate_exit());
        }

        #[slot(SlotNoArgs)]
        unsafe fn on_close(self: &Rc<Self>) {
            println!("emitting on_close signal for {}", self.widget.win_id().to_string());
            self.close_signal.emit(&qs(self.widget.win_id().to_string()));
            println!("emitted on_close signal for {}", self.widget.win_id().to_string());
            self.widget.close();
        }

        #[slot(SlotNoArgs)]
        unsafe fn on_button_clicked(self: &Rc<Self>) {

        }

        #[slot(SlotOfQTableWidgetItemQTableWidgetItem)]
        unsafe fn on_table_current_item_changed(
            self: &Rc<Self>,
            current: Ptr<QTableWidgetItem>,
            previous: Ptr<QTableWidgetItem>,
        ) {
            if !previous.is_null() {
                let font = previous.font();
                font.set_bold(false);
                previous.set_font(&font);
            }
            if !current.is_null() {
                let font = current.font();
                font.set_bold(true);
                current.set_font(&font);
            }
        }

        #[slot(SlotOfQPoint)]
        unsafe fn on_table_context_menu_requested(self: &Rc<Self>, pos: Ref<QPoint>) {
            
        }
    }
}