# Local build

The local build might be less convenient, since you must make sure
dynamic library dependencies are available. But if they are, the
build will be considerably faster than the flatpak build. You also
are running a version of papers without flatpak container restrictions. 

Building papers locally require the following system dependencies:

```
sudo apt install libpoppler-glib-dev libpoppler-dev libicu-dev
```

Poppler is a cairo-based PDF rendering library, and is used for PDF rendering inside the application. 
ICU is required by tectonic (the Rust-based Latex engine wrapper used by drafts). 

If you can't or don't want to use the system libraries, you can build them manually
with the help from the scripts below. Just make sure to have all the generated
dynamic libraries at $LD_LIBRARY_PATH when executing papers.

# Flatpak build

Perhaps you want to use a flatpak SDK instead of the system toolchain. This is perhaps the easiest approach,
since both poppler and libicu will be bundled, and you don't have to worry about them. 

If you don't have flatpak installed yet:

```
sudo apt install flatpak
```

Just make sure to have
the flatpak SDK and rust toolchain extension installed (note that this step will be done automatically if you
just run flatpak build [drafts]:

```
flatpak install org.gnome.Sdk org.gnome.Platform org.freedesktop.Sdk.Extension.rust-stable
```

When using the flatpak build, poppler will be bundled.

Workaround: Copy system libicuuc
cp /usr/lib/x86_64-linux-gnu/libicuuc.so.67.1 /home/diego/Downloads/papers-build/build/files/bin
mv /home/diego/Downloads/papers-build/build/files/bin/libicuuc.so.67.1 /home/diego/Downloads/papers-build/build/files/bin/libicuuc.so.69

```
LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/home/diego/Downloads/papers-build/build/files/lib ./papers
```

## To build the application into a custom directory

This exports the executable to the repo folder, and leave build artifacts at the build folder.

```
# Build + install

flatpak-builder --repo=/home/diego/Downloads/papers-build/repo \
    /home/diego/Downloads/papers-build/build com.github.limads.Papers.json \
    --state-dir=/home/diego/Downloads/papers-build/state --force-clean --install --user
```

```
# Just build, no install

flatpak-builder --repo=/home/diego/Downloads/papers-build/repo \
    /home/diego/Downloads/papers-build/build com.github.limads.Papers.json \
    --state-dir=/home/diego/Downloads/papers-build/state --force-clean
```

```
# Step-by-step to install after build with no install

flatpak build-finish /home/diego/Downloads/papers-build/build
flatpak build-export /home/diego/Downloads/papers-build/repo /home/diego/Downloads/papers-build/build org.github.limads.Papers

# Export a bundle
flatpak build-bundle /home/diego/Downloads/papers-build/repo /home/diego/Downloads/papers-build/papers.flatpak com.github.limads.Papers master

# Install from bundle
flatpak install --user --bundle /home/diego/Downloads/papers-build/papers.flatpak
```

```
# Run
flatpak run com.github.limads.Papers master
```

```
# Uninstall
flatpak uninstall com.github.limads.Papers
```

(This will leave a lot of artifacts at state dir (replacement for .flatpak-builder at current dir), which will be created at the directory the command is called).

# Checking the meson build of the final module

```
meson setup builddir
meson compile -C builddir
```

## To install locally

This installs the executable to the local flatpak applications directory

flatpak-builder --install /home/diego/Downloads/papers-build/build com.github.limads.Papers.json --force-clean

The flatpak build output will result in three directories: bin (with the papers executable), 
lib (with libpoppler.so and libpoppler-glib.so) and share 
(with appdata/com.github.limads.Papers.appdata.xml, app-info/icons and app-info/xmls, 
applications/com.github.limads/Papers.desktop, glib-2.0/schemas/(gschema files) and 
icons/hicolor/scalable and icons/hicolor/symbolic)
Local install will be at `~/.local/share/flatpak/` (user) or `/var/lib/flatpak/repo` (system)

Clean with `flatpak uninstall com.github.limads.Papers && flatpak uninstall --unused`
(The second command will uninstall Gnome 42 SDK when not by other apps).

To verify the checksum of the libicu zip:

```
sha256sum icu4c-69_1-src.tgz
```

# Local dynamic libraries dependencies build scripts

## Local ICU build

```
wget https://github.com/unicode-org/icu/releases/download/release-69-1/icu4c-69_1-src.tgz
mkdir icu4c-69_1-src
tar -xvf icu4c-69_1-src.tgz -C icu4c-69_1-src
cd icu4c-69_1-src
mkdir icu4c-build
cd icu4c-build
../icu/source/runConfigureICU Linux
make check
```

## Local poppler build

```
git clone https://gitlab.freedesktop.org/poppler/poppler.git
cd poppler
mkdir build && cd build
cmake .. -DENABLE_BOOST=OFF -DENABLE_LIBOPENJPEG=none -DCMAKE_CXX_FLAGS=-I/usr/include -DBUILD_GTK_TESTS=OFF -DBUILD_QT5_TESTS=OFF -DBUILD_QT6_TESTS=OFF -DBUILD_CPP_TESTS=OFF -DBUILD_MANUAL_TESTS=OFF -DENABLE_UTILS=OFF -DENABLE_CPP=OFF -DENABLE_GOBJECT_INTROSPECTION=OFF -DENABLE_QT5=OFF -DENABLE_QT6=OFF -DENABLE_LIBCURL=OFF -DENABLE_ZLIB=OFF
make
```

libjpeg is a transitive dependency of libpoppler. In theory, libpoppler requires libjpeg if DENABLE_DCTDECODER is at the default value libjpeg. 
But if we set it to none or unmaintained, compilation fails at CMakeFiles/poppler.dir/poppler/DCTStream.cc.o. We could drop it if the flag
actually worked. If we could compile it with "-DENABLE_DCTDECODER=none", the libjpeg module could be dropped.

# Local libjpeg build

```
wget http://www.ijg.org/files/jpegsrc.v6b.tar.gz
mkdir jpegsrc
tar -xvf jpegsrc.v6b.tar.gz -C jpegsrc

mkdir jpegsrc/out
mkdir jpegsrc/out/bin
mkdir jpegsrc/out/lib
mkdir jpegsrc/out/include
mkdir jpegsrc/out/man
mkdir jpegsrc/out/man/man1

cd jpegsrc
./configure --prefix=/home/diego/Downloads/jpegsrc/out --enable-shared

# For some reason the Makefile is built with a reference to a local libtool executable. Change to system libtool.
sed -i 's/\.\/libtool/libtool/g' Makefile

make
make install
```

libjpeg must installed **after** libpoppler, or else libpoppler compilation fails.
libjpeg.so symlink must be removed before the last Rust moduel is built, 
or else the Rust linking will fail. Just keep libjpeg.so.62 

Inspect dir 

```        
ls -R | grep ":$" | sed -e 's/:$//' -e 's/[^-][^\/]*\//──/g' -e 's/─/├/' -e '$s/├/└/'

commands : [
    "ls ../.. -R | grep \":$\" | sed -e 's/:$//' -e 's/[^-][^\\/]*\\//──/g' -e 's/─/├/' -e '$s/├/└/'"
]
```

libjpeg module

```json
{
    "name" : "libjpeg",
    "buildsystem" : "simple",
    "build-commands" : [
    	"mkdir ${FLATPAK_DEST}/man",
    	"mkdir ${FLATPAK_DEST}/man/man1",
	"./configure --prefix=/app --enable-shared",
	"sed -i 's/\\.\\/libtool/libtool --tag=CC/g' Makefile",
	"make",
	"make install"
	],
    "cleanup" : [
        "/man",
    	"/bin",
    	"/include",
	    "libjpeg.so",
	    "libjpeg.so.62.0.0.debug"
    ],
"sources" : [
	    {
	"type": "archive",
	"url": "http://www.ijg.org/files/jpegsrc.v6b.tar.gz",
	"sha256": "75c3ec241e9996504fe02a9ed4d12f16b74ade713972f3db9e65ce95cd27e35d"
    }
]
},
```

To access the build directory of the current module (/run/build/$MODULE):

$FLATPAK_BUILDER_BUILDDIR

To access the executables directory (app/bin)

$PATH

-- Plain cargo build without meson

The .desktop XML file should be installed to /home/diego/.local/share/flatpak/exports/applications 
Icons should be installed to /home/diego/.local/share/flatpak/exports/share/icons/hicolor/symbolic/apps (symbolic icons) and
/home/diego/.local/share/flatpak/exports/share/icons/hicolor/scalable/apps (application icon)

/home/diego/.local/share/flatpak/exports/bin and /home/diego/.local/share/flatpak/exports/share contains only
symlinks to the actual application files. Those are located under /home/diego/.local/share/flatpak/app/<APPID>/x86_64/stable/hash/export and
/home/diego/.local/share/flatpak/app/<APPID>/x86_64/stable/hash/bin

commands : [
    "export PKG_CONFIG_PATH=$LD_LIBRARY_PATH/pkgconfig",
    "cargo build --manifest-path=Cargo.toml --target-dir=${FLATPAK_BUILDER_BUILDDIR}",
    "cp data/${FLATPAK_ID}.desktop app/share/applications"
    "cp data/icons/hicolor/scalable/apps ${FLATPAK_DEST}/share/icons/hicolor/scalable/apps
    "cp data/icons/hicolor/symbolic/apps ${FLATPAK_DEST}/share/icons/hicolor/symbolic/apps
    "cp data/${FLATPAK_ID}.appdata.xml ${FLATPAK_DEST}/share/metainfo
]

"cp $FLATPAK_BUILDER_BUILDDIR/lib/libicuuc.so.69 $FLATPAK_BUILDER_BUILDDIR/lib/libicuuc.so.67"

# File validation

desktop-file-validate data/com.github.limads.Papers.desktop

appstream-util validate-relax
/app/share/metainfo/org.gnome.Dictionary.appdata.xml
