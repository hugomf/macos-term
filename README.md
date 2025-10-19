# macOS Terminal with Native Blur Effects

A cross-platform terminal application built in Rust that demonstrates native library integration for advanced window effects. Currently implemented for macOS with plans for Linux and Windows support.

## 🎯 Project Purpose

This project serves as a learning platform for understanding **Rust-native library communication**. The primary goals are:

- **macOS**: Master Objective-C/Swift bridge development and private API integration
- **Linux**: Explore X11/Wayland native window management and compositing
- **Windows**: Implement DWM (Desktop Window Manager) API integration

## 🚀 Features

### Current (macOS)
- **Window Blur Effects**: Real-time adjustable blur radius using private macOS APIs
- **Glass Tint Controls**: Color overlay effects with preset colors
- **Opacity Management**: Adjustable window transparency
- **GTK4 Interface**: Modern, responsive UI with terminal-style aesthetics
- **Native Bridge**: Direct communication between Rust and Objective-C

### Technical Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Rust GTK4 UI  │◄──►│  Native Bridge   │◄──►│  macOS APIs     │
│   (main.rs)     │    │  (macos_bridge.m)│    │  (WindowServer) │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## 🏗️ Project Structure

```
macos-term/
├── src/
│   └── main.rs              # GTK4 UI and application logic
├── macos_bridge.m           # Objective-C bridge for native APIs
├── build.rs                 # Build script for compiling bridge
├── Cargo.toml              # Rust dependencies and configuration
└── README.md               # This file
```

## 🛠️ Dependencies

### Rust Crates
- `gtk4` - GUI framework
- `cocoa` - macOS bindings
- `objc2` - Objective-C runtime
- `glib` - GNOME base library

### Build Dependencies
- `cc` - C/C++ compiler integration

## 🔧 Building and Running

### Prerequisites
- **macOS** with Xcode command line tools
- **Rust** (latest stable)
- **GTK4** development libraries

### Build Steps
```bash
# Clone the repository
git clone <repository-url>
cd macos-term

# Build the project
cargo build --release

# Run the application
cargo run
```

## 🎮 Usage

1. **Opacity Control**: Adjust window transparency (0-100%)
2. **Blur Radius**: Control background blur intensity (0-100px)
3. **Glass Tint**: Apply color overlays for different visual effects
4. **Presets**: Quick color selection (Black, White, Red, Green, Blue, Purple)

## 🏗️ Development Roadmap

### Phase 1: macOS (Current)
- ✅ Basic window blur implementation
- ✅ Color tint effects
- ✅ GTK4 integration
- 🔄 Documentation and cleanup

### Phase 2: Linux (Planned)
- 🔄 X11 window management research
- ⏳ Wayland compositing integration
- ⏳ KDE KWin effects API
- ⏳ GNOME Shell extensions

### Phase 3: Windows (Future)
- ⏳ DWM API integration
- ⏳ Acrylic effects implementation
- ⏳ Win32 native bridge development

## 🔍 Technical Learning Objectives

### Native Library Communication
- **FFI (Foreign Function Interface)** patterns
- **Memory management** across language boundaries
- **Error handling** in cross-language contexts
- **Build system integration** for mixed-language projects

### Platform-Specific Challenges
- **macOS**: Private API stability and App Store compatibility
- **Linux**: Desktop environment fragmentation (GNOME/KDE/XFCE)
- **Windows**: DWM API versioning and Windows 10/11 differences

## 🤝 Contributing

This is a learning project! Contributions, suggestions, and questions are welcome:

1. **Issues**: Report bugs or request features
2. **Discussions**: Share ideas for cross-platform implementation
3. **Pull Requests**: Submit improvements or platform ports

## ⚠️ Important Notes

### macOS Private APIs
- Uses private macOS APIs that may break in future versions
- Not suitable for App Store distribution
- Intended for educational purposes only

### Cross-Platform Considerations
- Each platform requires different approaches to window effects
- API availability and stability vary significantly
- Performance characteristics differ across platforms

## 📚 Resources

- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)
- [macOS Window Management](https://developer.apple.com/documentation/appkit/nswindow)
- [GTK4 Documentation](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/)
- [Wayland Protocols](https://wayland.freedesktop.org/docs/)

## 📄 License

This project is for educational purposes. See license file for details.

---

**Built with ❤️ using Rust and native platform APIs**
