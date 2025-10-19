use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, Box, Button, ColorButton, Label, Orientation, Scale, ScrolledWindow};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::msg_send;
use objc2_app_kit::{NSWindow, NSColor};
use std::rc::Rc;
use std::cell::RefCell;

// MARK: - Private CoreGraphics API
type CGSConnectionID = u32;
type CGSWindowID = u32;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGSDefaultConnectionForThread() -> CGSConnectionID;
}

fn cgs_set_window_background_blur_radius(
    connection: CGSConnectionID,
    window_id: CGSWindowID,
    radius: u32,
) -> i32 {
    let lib = unsafe {
        match libloading::Library::new(
            "/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics"
        ) {
            Ok(lib) => lib,
            Err(e) => {
                eprintln!("❌ Failed to load CoreGraphics: {}", e);
                return -1;
            }
        }
    };

    let func: libloading::Symbol<unsafe extern "C" fn(CGSConnectionID, CGSWindowID, u32) -> i32> =
        unsafe {
            match lib.get(b"CGSSetWindowBackgroundBlurRadius") {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("❌ Failed to load CGSSetWindowBackgroundBlurRadius: {}", e);
                    return -1;
                }
            }
        };

    let result = unsafe { func(connection, window_id, radius) };
    println!("🔄 Blur radius={}, result={}", radius, result);
    result
}

struct WindowBlurManager {
    connection_id: CGSConnectionID,
}

impl WindowBlurManager {
    fn new() -> Self {
        let connection_id = unsafe { CGSDefaultConnectionForThread() };
        println!("✅ Got connection ID: {}", connection_id);
        Self { connection_id }
    }

    fn set_blur(&self, ns_window: &NSWindow, radius: u32) {
        unsafe {
            ns_window.setOpaque(false);
            let clear_color = NSColor::clearColor();
            ns_window.setBackgroundColor(Some(&clear_color));
            ns_window.setHasShadow(true);
            ns_window.setTitlebarAppearsTransparent(true);
            
            let window_number = ns_window.windowNumber() as CGSWindowID;
            cgs_set_window_background_blur_radius(self.connection_id, window_number, radius);
            
            ns_window.invalidateShadow();
            if let Some(content_view) = ns_window.contentView() {
                content_view.setNeedsDisplay(true);
            }
        }
    }
}

// Safe wrapper for getting NSWindow
fn get_ns_window(gtk_window: &ApplicationWindow) -> Option<Retained<NSWindow>> {
    let surface = gtk_window.surface()?;
    let surface_ptr = surface.as_ptr() as *mut AnyObject;
    
    // Check if the surface responds to nativeWindow selector
    let responds: bool = unsafe { msg_send![surface_ptr, respondsToSelector: objc2::sel!(nativeWindow)] };
    
    if !responds {
        eprintln!("❌ Surface does not respond to nativeWindow selector");
        return None;
    }
    
    let ns_window_ptr: *mut AnyObject = unsafe { msg_send![surface_ptr, nativeWindow] };
    
    if ns_window_ptr.is_null() {
        eprintln!("❌ Failed to get NSWindow from GdkSurface - nativeWindow returned null");
        return None;
    }
    
    // Try to cast to NSWindow - Retained::retain returns Option, not Result
    unsafe { Retained::retain(ns_window_ptr as *mut NSWindow) }
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        "window {
            background: transparent;
        }
        .controls-panel {
            background: #1e1e1e;
            border-radius: 8px;
            margin: 8px;
            padding: 16px;
        }
        .controls-title {
            color: #ffffff;
            font-weight: bold;
            font-size: 14px;
        }
        .control-label {
            color: #cccccc;
            font-size: 11px;
        }
        .control-value {
            color: #00ff00;
            font-size: 11px;
            font-family: monospace;
        }
        .terminal-background {
            background: rgba(255, 255, 255, 0.8);
            border-radius: 8px;
            margin: 8px;
        }
        .terminal-text {
            color: #000000;
            font-family: 'Monaco', 'Menlo', monospace;
            font-size: 13px;
        }
        .preset-color {
            min-width: 20px;
            min-height: 20px;
            border-radius: 10px;
            padding: 0;
            margin: 2px;
        }
        .warning-text {
            color: #ff4444;
            font-weight: bold;
            font-size: 11px;
        }",
    );

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn update_terminal_background(terminal_background: &Box, opacity: f64, color: &gtk4::gdk::RGBA) {
    let r = (color.red() * 255.0) as u32;
    let g = (color.green() * 255.0) as u32;
    let b = (color.blue() * 255.0) as u32;
    
    // Update the CSS for the terminal background with the selected color and opacity
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(&format!(
        ".terminal-background {{
            background: rgba({}, {}, {}, {});
            border-radius: 8px;
            margin: 8px;
        }}
        .terminal-text {{
            color: {};
            font-family: 'Monaco', 'Menlo', monospace;
            font-size: 13px;
        }}",
        r, g, b, opacity,
        if color.red() > 0.5 && color.green() > 0.5 && color.blue() > 0.5 { "#000000" } else { "#ffffff" }
    ));

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Terminal - Custom Blur API")
        .default_width(800)
        .default_height(600)
        .build();

    let _blur_manager = Rc::new(WindowBlurManager::new());
    let main_box = Box::new(Orientation::Vertical, 0);
    
    // Store current state
    let current_opacity = Rc::new(RefCell::new(0.8)); // Start with 80% opacity
    let current_color = Rc::new(RefCell::new(gtk4::gdk::RGBA::new(1.0, 1.0, 1.0, 1.0))); // Start with white
    
    // Terminal viewport with separate background container - Now with more space since header is removed
    let terminal_background = Box::new(Orientation::Vertical, 0);
    terminal_background.add_css_class("terminal-background");
    terminal_background.set_vexpand(true);
    
    let terminal_scroll = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();
    
    let terminal_content = Box::new(Orientation::Vertical, 4);
    terminal_content.set_margin_start(12);
    terminal_content.set_margin_end(12);
    terminal_content.set_margin_top(12);
    terminal_content.set_margin_bottom(12);
    
    // Use ASCII characters only to avoid font loading issues
    let terminal_lines = [
        "Last login: Sun Oct 19 14:23:45 on ttys001",
        "~ $ ls -la",
        "total 64",
        "drwxr-xr-x  12 user  staff   384 Oct 19 14:23 .",
        "drwxr-xr-x   5 user  staff   160 Oct 19 14:20 ..",
        "-rw-r--r--   1 user  staff   284 Oct 19 14:23 main.rs",
        "~ $ ./target/debug/macos-term",
        "Starting terminal with blur effects...",
        "~ $ _",
    ];
    
    for line in terminal_lines {
        let label = Label::new(Some(line));
        label.set_halign(gtk4::Align::Start);
        label.add_css_class("terminal-text");
        terminal_content.append(&label);
    }
    
    terminal_scroll.set_child(Some(&terminal_content));
    terminal_background.append(&terminal_scroll);
    
    // Controls panel - More compact
    let controls = Box::new(Orientation::Vertical, 12);
    controls.add_css_class("controls-panel");
    
    let controls_title = Label::new(Some("Window Effects"));
    controls_title.set_halign(gtk4::Align::Start);
    controls_title.add_css_class("controls-title");
    
    // Opacity slider - More compact
    let opacity_box = Box::new(Orientation::Vertical, 6);
    let opacity_label_box = Box::new(Orientation::Horizontal, 0);
    let opacity_label = Label::new(Some("Background Opacity:"));
    opacity_label.set_hexpand(true);
    opacity_label.set_halign(gtk4::Align::Start);
    opacity_label.add_css_class("control-label");
    
    let opacity_value = Label::new(Some("80%"));
    opacity_value.add_css_class("control-value");
    
    opacity_label_box.append(&opacity_label);
    opacity_label_box.append(&opacity_value);
    
    let opacity_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    opacity_slider.set_value(80.0);
    
    let terminal_background_weak = terminal_background.downgrade();
    let opacity_value_clone = opacity_value.clone();
    let current_opacity_clone = current_opacity.clone();
    let current_color_clone = current_color.clone();
    
    opacity_slider.connect_value_changed(move |slider| {
        let value = slider.value();
        opacity_value_clone.set_text(&format!("{:.0}%", value));
        
        let opacity = value / 100.0;
        *current_opacity_clone.borrow_mut() = opacity;
        
        if let Some(background) = terminal_background_weak.upgrade() {
            let color = current_color_clone.borrow();
            update_terminal_background(&background, opacity, &color);
        }
    });
    
    opacity_box.append(&opacity_label_box);
    opacity_box.append(&opacity_slider);
    
    // Color picker with preset colors
    let color_box = Box::new(Orientation::Vertical, 6);
    let color_label_box = Box::new(Orientation::Horizontal, 0);
    let color_label = Label::new(Some("Glass Tint:"));
    color_label.set_hexpand(true);
    color_label.set_halign(gtk4::Align::Start);
    color_label.add_css_class("control-label");
    
    let color_button = ColorButton::new();
    color_button.set_rgba(&gtk4::gdk::RGBA::new(1.0, 1.0, 1.0, 1.0)); // Start with white for frosted glass
    
    color_label_box.append(&color_label);
    color_label_box.append(&color_button);
    
    // Preset colors - Same as SwiftUI version
    let presets_box = Box::new(Orientation::Horizontal, 4);
    let presets_label = Label::new(Some("Presets:"));
    presets_label.add_css_class("control-label");
    presets_box.append(&presets_label);
    
    let presets = [
        ("Black", (0.0, 0.0, 0.0)),
        ("White", (1.0, 1.0, 1.0)),
        ("Red", (0.8, 0.2, 0.2)),
        ("Green", (0.2, 0.8, 0.2)),
        ("Blue", (0.2, 0.4, 0.9)),
        ("Purple", (0.6, 0.2, 0.8)),
    ];
    
    let terminal_background_weak = terminal_background.downgrade();
    let current_opacity_clone = current_opacity.clone();
    let current_color_clone = current_color.clone();
    let color_button_clone_outer = color_button.clone();
    
    for (name, (r, g, b)) in presets {
        let preset_btn = Button::new();
        preset_btn.set_size_request(20, 20);
        preset_btn.add_css_class("preset-color");
        preset_btn.set_tooltip_text(Some(name));
        
        // Set the button background color
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(&format!(
            ".preset-color {{
                background: rgb({}, {}, {});
                min-width: 20px;
                min-height: 20px;
                border-radius: 10px;
                padding: 0;
                margin: 2px;
            }}",
            (r * 255.0) as u32,
            (g * 255.0) as u32,
            (b * 255.0) as u32
        ));
        
        preset_btn.style_context().add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        
        let rgba = gtk4::gdk::RGBA::new(r, g, b, 1.0);
        let terminal_background_weak = terminal_background_weak.clone();
        let current_opacity_clone = current_opacity_clone.clone();
        let current_color_clone = current_color_clone.clone();
        let color_button_clone = color_button_clone_outer.clone();
        
        preset_btn.connect_clicked(move |_| {
            color_button_clone.set_rgba(&rgba);
            *current_color_clone.borrow_mut() = rgba.clone();
            
            if let Some(background) = terminal_background_weak.upgrade() {
                let opacity = *current_opacity_clone.borrow();
                update_terminal_background(&background, opacity, &rgba);
            }
        });
        
        presets_box.append(&preset_btn);
    }
    
    // Connect color button changes
    let terminal_background_weak = terminal_background.downgrade();
    let current_opacity_clone = current_opacity.clone();
    let current_color_clone = current_color.clone();
    
    color_button.connect_rgba_notify(move |color_button| {
        let rgba = color_button.rgba();
        *current_color_clone.borrow_mut() = rgba.clone();
        
        if let Some(background) = terminal_background_weak.upgrade() {
            let opacity = *current_opacity_clone.borrow();
            update_terminal_background(&background, opacity, &rgba);
        }
    });
    
    color_box.append(&color_label_box);
    color_box.append(&presets_box);
    
    // Blur radius slider - Disabled for now to avoid crashes
    let blur_box = Box::new(Orientation::Vertical, 6);
    let blur_label_box = Box::new(Orientation::Horizontal, 0);
    let blur_label = Label::new(Some("Blur Radius: (Disabled - API Issue)"));
    blur_label.set_hexpand(true);
    blur_label.set_halign(gtk4::Align::Start);
    blur_label.add_css_class("control-label");
    
    let blur_value = Label::new(Some("N/A"));
    blur_value.add_css_class("control-value");
    
    blur_label_box.append(&blur_label);
    blur_label_box.append(&blur_value);
    
    let blur_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    blur_slider.set_value(50.0);
    blur_slider.set_sensitive(false); // Disable the slider
    
    blur_box.append(&blur_label_box);
    blur_box.append(&blur_slider);
    
    // Info section - More compact
    let info_box = Box::new(Orientation::Vertical, 4);
    let info_title = Label::new(Some("Architecture Note:"));
    info_title.set_halign(gtk4::Align::Start);
    info_title.add_css_class("control-label");
    
    let info_lines = [
        "• Opacity: 0% = fully transparent, 100% = fully opaque",
        "• Glass tint: adds color overlay effect",
        "• Blur radius: Currently disabled due to API issues",
    ];
    
    info_box.append(&info_title);
    for line in info_lines {
        let info_label = Label::new(Some(line));
        info_label.set_halign(gtk4::Align::Start);
        info_label.add_css_class("control-label");
        info_box.append(&info_label);
    }
    
    let warning = Label::new(Some("Private API - May break in future macOS versions"));
    warning.add_css_class("warning-text");
    info_box.append(&warning);
    
    // Reordered controls: Opacity first, then Color, then Blur
    controls.append(&controls_title);
    controls.append(&opacity_box);
    controls.append(&color_box);
    controls.append(&blur_box);
    controls.append(&gtk4::Separator::new(Orientation::Horizontal));
    controls.append(&info_box);
    
    // Add terminal and controls directly to main box (no header)
    main_box.append(&terminal_background);
    main_box.append(&controls);
    
    window.set_child(Some(&main_box));
    
    // Load CSS styling and apply initial color/opacity
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