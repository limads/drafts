# To build the application into a specified directory

This exports the executable to the repo folder, and leave build artifacts at the build folder.

```
flatpak-builder --repo=/home/diego/Downloads/papers-build/repo /home/diego/Downloads/papers-build/build com.github.limads.Papers.json --force-clean
```

# To install locally

This installs the executable to the local flatpak applications directory

flatpak-builder --install /home/diego/Downloads/papers-build/build com.github.limads.Papers.json --force-clean
