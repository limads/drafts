# To build the application into a specified directory

This exports the executable to the repo folder, and leave build artifacts at the build folder.

```
flatpak-builder --repo=/home/diego/Downloads/papers-build/repo /home/diego/Downloads/papers-build/build com.github.limads.Papers.json --state-dir=/home/diego/Downloads/papers-build/state --force-clean
```

(This will leave a lot of artifacts at state dir (replacement for .flatpak-builder at current dir), which will be created at the directory the command is called).

# To install locally

This installs the executable to the local flatpak applications directory

flatpak-builder --install /home/diego/Downloads/papers-build/build com.github.limads.Papers.json --force-clean

The flatpak build output will result in three directories: bin (with the papers executable), lib (with libpoppler.so and libpoppler-glib.so) and share (with appdata/com.github.limads.Papers.appdata.xml, app-info/icons and app-info/xmls, applications/com.github.limads/Papers.desktop, glib-2.0/schemas/(gschema files) and icons/hicolor/scalable and icons/hicolor/symbolic)
Local install will be at `~/.local/share/flatpak/` (user) or `/var/lib/flatpak/repo` (system)

Clean with `flatpak uninstall com.github.limads.Papers && flatpak uninstall --unused` (The second command will uninstall Gnome 42 SDK when not by other apps).




