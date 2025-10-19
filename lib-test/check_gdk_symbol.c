#include <stdio.h>
#include <dlfcn.h>

int main() {
    const char *lib_path = "/opt/homebrew/Cellar/gtk4/4.20.2/lib/libgtk-4.1.dylib";
    printf("Checking for gdk_quartz_window_get_ns_window in %s...\n", lib_path);

    void *handle = dlopen(lib_path, RTLD_LAZY);
    if (!handle) {
        fprintf(stderr, "ERROR: Could not open GTK library: %s\n", dlerror());
        return 1;
    }

    void *sym = dlsym(handle, "gdk_quartz_window_get_ns_window");
    if (sym) {
        printf("✅ Found symbol gdk_quartz_window_get_ns_window at %p\n", sym);
    } else {
        printf("❌ Symbol gdk_quartz_window_get_ns_window NOT found in GTK4\n");
    }

    dlclose(handle);
    return 0;
}
