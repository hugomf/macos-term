#!/bin/bash

# --- Diagnostic Script for GDK Quartz Symbol ---

echo "1. Checking if pkg-config for GTK4 is available..."
if ! pkg-config --exists gtk4; then
    echo "ERROR: pkg-config could not find gtk4. This is usually installed via Homebrew."
    echo "You may need to run: export PKG_CONFIG_PATH=\"/opt/homebrew/lib/pkgconfig:\$PKG_CONFIG_PATH\""
    exit 1
fi

echo "2. Compiling check_gdk_symbol.c using pkg-config flags..."

# Get the necessary CFLAGS and LDFLAGS for GTK4 (which includes GDK flags)
# On your system (Homebrew), these flags should point to the correct dylibs.
GTK4_CFLAGS=$(pkg-config --cflags gtk4)
GTK4_LIBS=$(pkg-config --libs gtk4)

# Combine the necessary linking flags: GTK4 libraries + ApplicationServices framework
LINK_FLAGS="$GTK4_LIBS -Wl,-framework,ApplicationServices"

echo "   -> CFLAGS: $GTK4_CFLAGS"
echo "   -> LIBS:   $LINK_FLAGS"

# Compilation command: cc is the standard C compiler (clang on macOS)
cc check_gdk_symbol.c $GTK4_CFLAGS $LINK_FLAGS -o check_gdk_symbol_test

# Check the exit status of the compilation
if [ $? -eq 0 ]; then
    echo "3. Compilation and Linking successful."
    echo "4. Running the test executable..."
    ./check_gdk_symbol_test
    echo "5. Checking for the symbol export inside the created executable (nm check)..."
    # The 'nm' command lists symbols. The 'U' means it's an Undefined reference (we want this to be resolved).
    # If the file compiled, this command should show the symbol is NOT undefined.
    nm check_gdk_symbol_test | grep _gdk_quartz_window_get_ns_window
    echo "--- DIAGNOSTIC TEST COMPLETE ---"
else
    echo "3. ERROR: Compilation or Linking FAILED."
    echo "   If this happens, it means one of two things:"
    echo "   a) The symbol is truly not exported by your GDK library version."
    echo "   b) The output of pkg-config is missing a critical path or library link for GDK."
fi
