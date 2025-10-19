#include <stdio.h>
#include <dlfcn.h>

int main() {
    const char *lib_path = "/opt/homebrew/Cellar/gtk4/4.20.2/lib/libgtk-4.1.dylib";
    printf("🔍 Checking GTK4 macOS APIs in %s...\n\n", lib_path);

    void *handle = dlopen(lib_path, RTLD_LAZY);
    if (!handle) {
        fprintf(stderr, "❌ ERROR: Could not open GTK library: %s\n", dlerror());
        return 1;
    }

    // Test the deprecated function (should fail)
    void *old_sym = dlsym(handle, "gdk_quartz_window_get_ns_window");
    if (old_sym) {
        printf("⚠️  Found DEPRECATED symbol gdk_quartz_window_get_ns_window at %p\n", old_sym);
        printf("   (This function is deprecated and should not be used)\n");
    } else {
        printf("✅ Deprecated symbol gdk_quartz_window_get_ns_window correctly NOT found\n");
    }

    printf("\n");

    // Test the new official API
    void *new_sym = dlsym(handle, "gdk_macos_surface_get_native_window");
    if (new_sym) {
        printf("✅ SUCCESS: Found NEW official API gdk_macos_surface_get_native_window at %p\n", new_sym);
        printf("   This is the modern, supported way to access native window!\n");
    } else {
        printf("❌ NEW API gdk_macos_surface_get_native_window NOT found\n");
        printf("   This might require a newer version of GTK4\n");
    }

    printf("\n");

    // Check GTK version
    void *version_sym = dlsym(handle, "gtk_get_major_version");
    if (version_sym) {
        printf("ℹ️  GTK4 version info available\n");
    } else {
        printf("ℹ️  Cannot determine GTK4 version\n");
    }

    dlclose(handle);
    return 0;
}
