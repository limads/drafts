![Drafts](https://raw.githubusercontent.com/limads/drafts/master/data/icons/hicolor/scalable/apps/io.github.limads.Drafts.svg?token=GHSAT0AAAAAABZXE27EZEVTFURGP7QUVNKCZBAU5VA)

# Drafts

A self-contained document preparation system.

Drafts is an editor for technical writing that leverages the [Typst](https://typst.app/docs/reference/) typesetting system.

## Features

- A lightweight editing experience with syntax highlighting and document navigation
        
- Lightning-fast PDF preview and export

- Menu-based interactions for common tasks (math symbol input, document formatting)

# Installation

A flathub release is begin worked on, but for now you can use:

## Flatpak

The preferred method of installation is Flatpak, which will guarantee you have
all the system dependencies setup and will provide a nicer system integration:

```
git clone https://github.com/limads/drafts
cd drafts
mkdir -p build
flatpak-builder --repo=build/repo build/build io.github.limads.Drafts.Devel.json --state-dir=build/state --force-clean --install --user
flatpak run io.github.limads.Drafts
```

## Direct cargo install

You will need to have a few system dependencies installed (they probably
are if you are on a distribution with a recent Gnome environment (>=43):

```
libgtk-4-1
libgtksourceview-5
libpoppler123
```

Make sure you also have a recent Rust toolchain (>=1.67), then use `cargo build` 
or `cargo install`:

```
git clone https://github.com/limads/drafts
cd drafts
cargo install --path .
./cargo/bin/drafts
```


