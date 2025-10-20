use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, Box, Button, ColorButton, Label, Orientation, Scale, ScrolledWindow};
use std::rc::Rc;
use std::cell::RefCell;

// External C functions from our bridge
unsafe extern "C" {
    fn macos_blur_init() -> i32;
    fn macos_blur_apply_to_gtk_window(window: *mut gtk4::ffi::GtkWindow, radius: u32) -> i32;
    fn macos_set_titlebar_opaque(window: *mut gtk4::ffi::GtkWindow) -> i32;
}

struct WindowBlurManager {
    initialized: bool,
}

impl WindowBlurManager {
    fn new() -> Self {
        let result = unsafe { macos_blur_init() };
        let initialized = result == 0;
        
        if initialized {
            println!("✅ WindowBlurManager initialized successfully");
        } else {
            eprintln!("❌ WindowBlurManager failed to initialize");
        }
        
        Self { initialized }
    }

    fn set_blur(&self, window: &ApplicationWindow, radius: u32) {
        if !self.initialized {
            eprintln!("❌ Cannot apply blur - manager not initialized");
            return;
        }

        let window_ptr = window.as_ptr() as *mut gtk4::ffi::GtkWindow;
        let result = unsafe { macos_blur_apply_to_gtk_window(window_ptr, radius) };
        
        if result == 0 {
            println!("✅ Blur applied successfully: radius={}", radius);
        } else {
            eprintln!("❌ Failed to apply blur: result={}", result);
        }
    }
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        "window {
            background: transparent;
        }
        .terminal-viewport {
            border: 1px solid rgba(0, 255, 255, 0.3);
        }
        .terminal-background {
            /* This will handle the background with opacity and color */
            background: rgba(0, 0, 0, 0.7); /* Start with 70% opacity (30% transparency) */
        }
        .terminal-text {
            font-family: 'SF Mono', Monaco, Menlo, 'DejaVu Sans Mono', 'Bitstream Vera Sans Mono', monospace;
            font-size: 14px;
        }
        .terminal-text.gray {
            color: #888;
        }
        .terminal-text.white {
            color: #fff;
        }
        .terminal-text.cyan {
            color: #00ffff;
        }
        .terminal-text.green {
            color: #00ff00;
        }
        .controls-panel {
            background: rgba(38, 38, 51, 1.0);
            padding: 12px;
        }
        .controls-title {
            color: white;
            font-family: 'SF Mono', Monaco, Menlo, 'DejaVu Sans Mono', 'Bitstream Vera Sans Mono', monospace;
            font-size: 16px;
            font-weight: bold;
        }
        .control-label {
            color: white;
            font-family: 'SF Mono', Monaco, Menlo, 'DejaVu Sans Mono', 'Bitstream Vera Sans Mono', monospace;
            font-size: 12px;
        }
        .control-value {
            color: #00ff00;
            font-family: 'SF Mono', Monaco, Menlo, 'DejaVu Sans Mono', 'Bitstream Vera Sans Mono', monospace;
            font-size: 12px;
        }
        .preset-label {
            color: #888;
            font-family: 'SF Mono', Monaco, Menlo, 'DejaVu Sans Mono', 'Bitstream Vera Sans Mono', monospace;
            font-size: 10px;
        }
        .info-title {
            color: #00ffff;
            font-family: 'SF Mono', Monaco, Menlo, 'DejaVu Sans Mono', 'Bitstream Vera Sans Mono', monospace;
            font-size: 11px;
            font-weight: bold;
        }
        .info-text {
            color: rgba(255, 255, 255, 0.8);
            font-family: 'SF Mono', Monaco, Menlo, 'DejaVu Sans Mono', 'Bitstream Vera Sans Mono', monospace;
            font-size: 10px;
        }
        .warning-text {
            color: #ff0000;
            font-family: 'SF Mono', Monaco, Menlo, 'DejaVu Sans Mono', 'Bitstream Vera Sans Mono', monospace;
            font-size: 10px;
            font-weight: 600;
            margin-top: 6px;
        }",
    );

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn create_terminal_line(text: &str, color: &str) -> Label {
    let label = Label::new(Some(text));
    label.set_halign(gtk4::Align::Start);
    label.add_css_class("terminal-text");
    label.add_css_class(color);
    label
}

fn create_terminal_prompt() -> Box {
    let prompt_box = Box::new(Orientation::Horizontal, 6);
    
    let tilde = Label::new(Some("~"));
    tilde.add_css_class("terminal-text");
    tilde.add_css_class("cyan");
    
    let arrow = Label::new(Some("$"));
    arrow.add_css_class("terminal-text");
    arrow.add_css_class("green");
    
    prompt_box.append(&tilde);
    prompt_box.append(&arrow);
    
    prompt_box
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Terminal - Custom Blur API")
        .default_width(800)
        .default_height(700)
        .build();

    // Ensure window supports transparency
    if let Some(surface) = window.surface() {
        let display = surface.display();
        if display.is_rgba() {
            println!("✅ Display supports RGBA");
        }
    }

    let blur_manager = Rc::new(WindowBlurManager::new());
    let main_box = Box::new(Orientation::Vertical, 0);
    
    // Store current opacity and color
    let current_opacity = Rc::new(RefCell::new(0.7)); // Start with 70% opacity
    let current_color = Rc::new(RefCell::new(gtk4::gdk::RGBA::new(0.0, 0.0, 0.0, 1.0)));
    
    // Terminal viewport with separate background - takes 70% of space
    let terminal_container = Box::new(Orientation::Vertical, 0);
    terminal_container.set_vexpand(true);
    terminal_container.set_hexpand(true);
    
    // Background box that will handle transparency and color
    let terminal_background = Box::new(Orientation::Vertical, 0);
    terminal_background.add_css_class("terminal-background");
    
    let terminal_scroll = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();
    terminal_scroll.add_css_class("terminal-viewport");
    
    let terminal_content = Box::new(Orientation::Vertical, 6);
    terminal_content.set_margin_start(12);
    terminal_content.set_margin_end(12);
    terminal_content.set_margin_top(12);
    terminal_content.set_margin_bottom(12);
    
    terminal_content.append(&create_terminal_line("Last login: Sun Oct 19 14:23:45 on ttys001", "gray"));
    terminal_content.append(&create_terminal_prompt());
    
    for i in 0..20 {
        terminal_content.append(&create_terminal_prompt());
        terminal_content.append(&create_terminal_line(
            &format!("echo 'Testing blur radius: {}'", i * 5),
            "white"
        ));
        terminal_content.append(&create_terminal_line(
            &format!("Testing blur radius: {}", i * 5),
            "cyan"
        ));
    }
    
    terminal_content.append(&create_terminal_prompt());
    terminal_scroll.set_child(Some(&terminal_content));
    
    // Add the scroll window to the background box using append()
    terminal_background.append(&terminal_scroll);
    terminal_container.append(&terminal_background);
    
    // Controls panel - very compact spacing, takes minimum space
    let controls = Box::new(Orientation::Vertical, 6);
    controls.add_css_class("controls-panel");
    controls.set_vexpand(false);
    controls.set_vexpand_set(true);
    
    let controls_title_box = Box::new(Orientation::Horizontal, 6);
    let sparkles = Label::new(Some("*"));
    let controls_title = Label::new(Some("Window Effects"));
    controls_title.add_css_class("controls-title");
    controls_title_box.append(&sparkles);
    controls_title_box.append(&controls_title);
    
    // Opacity slider (opposite of transparency) - very compact
    let opacity_box = Box::new(Orientation::Vertical, 2);
    let opacity_label_box = Box::new(Orientation::Horizontal, 0);
    let opacity_label = Label::new(Some("Opacity:"));
    opacity_label.add_css_class("control-label");
    opacity_label.set_hexpand(true);
    opacity_label.set_halign(gtk4::Align::Start);
    let opacity_value = Label::new(Some("70%"));
    opacity_value.add_css_class("control-value");
    opacity_label_box.append(&opacity_label);
    opacity_label_box.append(&opacity_value);
    
    let opacity_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    opacity_slider.set_value(70.0);
    
    let opacity_value_clone = opacity_value.clone();
    let current_opacity_clone = current_opacity.clone();
    
    // Create CSS providers for the background and border
    let background_provider = gtk4::CssProvider::new();
    let border_provider = gtk4::CssProvider::new();
    let terminal_background_weak = terminal_background.downgrade();
    let terminal_scroll_weak = terminal_scroll.downgrade();
    
    // Function to update background and border with current color and opacity
    let update_background = {
        let background_provider = background_provider.clone();
        let border_provider = border_provider.clone();
        let terminal_background_weak = terminal_background_weak.clone();
        let terminal_scroll_weak = terminal_scroll_weak.clone();
        move |color: &gtk4::gdk::RGBA, opacity: f64| {
            if let Some(background) = terminal_background_weak.upgrade() {
                let css = format!(
                    ".terminal-background {{ background: rgba({}, {}, {}, {}); }}",
                    (color.red() * 255.0) as u32,
                    (color.green() * 255.0) as u32,
                    (color.blue() * 255.0) as u32,
                    opacity
                );
                background_provider.load_from_data(&css);
                background.style_context().add_provider(&background_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
            }
            
            if let Some(scroll) = terminal_scroll_weak.upgrade() {
                let border_css = format!(
                    ".terminal-viewport {{ border: 1px solid rgba({}, {}, {}, 0.5); }}",
                    (color.red() * 255.0) as u32,
                    (color.green() * 255.0) as u32,
                    (color.blue() * 255.0) as u32
                );
                border_provider.load_from_data(&border_css);
                scroll.style_context().add_provider(&border_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
            }
        }
    };
    
    opacity_slider.connect_value_changed({
        let update_background = update_background.clone();
        let current_color_clone = current_color.clone();
        move |slider| {
            let slider_value = slider.value();
            // Clamp opacity to max 99% to avoid rendering issues
            let opacity_value = (slider_value / 100.0).min(0.99);
            opacity_value_clone.set_text(&format!("{:.0}%", slider_value));
            *current_opacity_clone.borrow_mut() = opacity_value;
            
            let color = *current_color_clone.borrow();
            update_background(&color, opacity_value);
        }
    });
    
    opacity_box.append(&opacity_label_box);
    opacity_box.append(&opacity_slider);
    
    // Blur radius slider - very compact
    let blur_box = Box::new(Orientation::Vertical, 2);
    let blur_label_box = Box::new(Orientation::Horizontal, 0);
    let blur_label = Label::new(Some("Blur Radius:"));
    blur_label.add_css_class("control-label");
    blur_label.set_hexpand(true);
    blur_label.set_halign(gtk4::Align::Start);
    let blur_value = Label::new(Some("50 px"));
    blur_value.add_css_class("control-value");
    blur_label_box.append(&blur_label);
    blur_label_box.append(&blur_value);
    
    let blur_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    blur_slider.set_value(50.0);
    
    let window_weak = window.downgrade();
    let blur_manager_clone = blur_manager.clone();
    let blur_value_clone = blur_value.clone();
    blur_slider.connect_value_changed(move |slider| {
        let value = slider.value();
        blur_value_clone.set_text(&format!("{:.0} px", value));
        
        if let Some(win) = window_weak.upgrade() {
            blur_manager_clone.set_blur(&win, value as u32);
        }
    });
    
    blur_box.append(&blur_label_box);
    blur_box.append(&blur_slider);
    
    // Color picker - compact
    let color_box = Box::new(Orientation::Vertical, 4);
    let color_label_box = Box::new(Orientation::Horizontal, 0);
    let color_label = Label::new(Some("Glass Tint:"));
    color_label.add_css_class("control-label");
    color_label.set_hexpand(true);
    color_label.set_halign(gtk4::Align::Start);
    
    let color_button = ColorButton::new();
    color_button.set_rgba(&gtk4::gdk::RGBA::new(0.0, 0.0, 0.0, 1.0));
    
    color_label_box.append(&color_label);
    color_label_box.append(&color_button);
    
    // Color picker callback
    {
        let update_background = update_background.clone();
        let current_color_clone = current_color.clone();
        let current_opacity_clone = current_opacity.clone();
        color_button.connect_rgba_notify(move |color_button| {
            let color = color_button.rgba();
            *current_color_clone.borrow_mut() = color;
            let opacity = *current_opacity_clone.borrow();
            update_background(&color, opacity);
        });
    }
    
    // Preset colors
    let presets_box = Box::new(Orientation::Horizontal, 8);
    let presets_label = Label::new(Some("Presets:"));
    presets_label.add_css_class("preset-label");
    presets_box.append(&presets_label);
    
    let presets = [
        ("Black", (0.0, 0.0, 0.0)),
        ("White", (1.0, 1.0, 1.0)),
        ("Red", (0.8, 0.2, 0.2)),
        ("Green", (0.2, 0.8, 0.2)),
        ("Blue", (0.2, 0.4, 0.9)),
        ("Purple", (0.6, 0.2, 0.8)),
    ];
    
    for (name, (r, g, b)) in presets {
        let preset_btn = Button::new();
        preset_btn.set_size_request(20, 20);
        preset_btn.set_tooltip_text(Some(name));
        
        // Style each button with its color
        let css = format!(
            ".preset-{} {{ background: rgb({}, {}, {}); min-width: 20px; min-height: 20px; border-radius: 10px; }}",
            name.to_lowercase(),
            (r * 255.0) as u32,
            (g * 255.0) as u32,
            (b * 255.0) as u32
        );
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(&css);
        preset_btn.style_context().add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        preset_btn.add_css_class(&format!("preset-{}", name.to_lowercase()));
        
        let rgba = gtk4::gdk::RGBA::new(r, g, b, 1.0);
        let color_button_clone = color_button.clone();
        let update_background = update_background.clone();
        let current_color_clone = current_color.clone();
        let current_opacity_clone = current_opacity.clone();
        preset_btn.connect_clicked(move |_| {
            color_button_clone.set_rgba(&rgba);
            *current_color_clone.borrow_mut() = rgba;
            let opacity = *current_opacity_clone.borrow();
            update_background(&rgba, opacity);
        });
        
        presets_box.append(&preset_btn);
    }
    
    color_box.append(&color_label_box);
    color_box.append(&presets_box);
    
    // Info section - compact
    let info_box = Box::new(Orientation::Vertical, 3);
    let info_title_box = Box::new(Orientation::Horizontal, 4);
    let info_icon = Label::new(Some("i"));
    let info_title = Label::new(Some("Architecture Note:"));
    info_title.add_css_class("info-title");
    info_title_box.append(&info_icon);
    info_title_box.append(&info_title);
    
    let info_lines = [
        "• Opacity: 0% = fully transparent, 100% = fully opaque",
        "• Blur radius: adjusts desktop blur intensity", 
        "• Glass tint: adds color overlay effect",
        "• White tint = frosted glass, Colors = stained glass",
    ];
    
    info_box.append(&info_title_box);
    for line in info_lines {
        let info_label = Label::new(Some(line));
        info_label.add_css_class("info-text");
        info_label.set_halign(gtk4::Align::Start);
        info_box.append(&info_label);
    }
    
    let warning = Label::new(Some("WARNING: Private API - May break in future macOS versions"));
    warning.add_css_class("warning-text");
    info_box.append(&warning);
    
    controls.append(&controls_title_box);
    controls.append(&opacity_box);
    controls.append(&blur_box);
    controls.append(&color_box);
    controls.append(&gtk4::Separator::new(Orientation::Horizontal));
    controls.append(&info_box);
    
    // Remove header and just use terminal and controls
    main_box.append(&terminal_container);
    main_box.append(&controls);
    
    window.set_child(Some(&main_box));
    
    // Handle window close to avoid GTK warnings
    window.connect_close_request(|_| {
        glib::Propagation::Proceed
    });
    
    // Apply initial blur and set titlebar opaque after window is realized
    let window_weak = window.downgrade();
    let blur_manager_clone = blur_manager.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
        if let Some(win) = window_weak.upgrade() {
            blur_manager_clone.set_blur(&win, 50);
            
            // Make titlebar opaque
            let window_ptr = win.as_ptr() as *mut gtk4::ffi::GtkWindow;
            let result = unsafe { macos_set_titlebar_opaque(window_ptr) };
            if result == 0 {
                println!("✅ Titlebar set to opaque");
            } else {
                eprintln!("❌ Failed to set titlebar opaque: result={}", result);
            }
        }
        glib::ControlFlow::Break
    });
    
    load_css();
    window.present();
}

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("com.example.macos-term")
        .build();

    app.connect_activate(build_ui);
    app.run()
}