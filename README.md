![Drafts](https://raw.githubusercontent.com/limads/drafts/master/data/icons/hicolor/scalable/apps/io.github.limads.Drafts.svg?token=GHSAT0AAAAAABZXE27EZEVTFURGP7QUVNKCZBAU5VA)

# Drafts (Work in progress)

Drafts is an editor for technical writing that leverages the [Typst](https://typst.app/docs/reference/) typesetting system.

## Features

- A lightweight editing experience with syntax highlighting and document navigation
        
- Lightning-fast PDF preview and export (thanks to the recently open sourced [Typst compiler](https://github.com/typst/typst)).

- Menu-based interactions for common tasks (math symbol input, document formatting)

- Provides a few generic templates to help you get started.

# Installation

A Flathub release is begin worked on, but for now you can use:

## Flatpak

The preferred method of installation is Flatpak, which will guarantee you have
all the system dependencies setup and will provide a nicer system integration:

```
wget https://raw.githubusercontent.com/limads/drafts/master/io.github.limads.Drafts.Devel.json
mkdir drafts-build
flatpak-builder --repo=drafts-build/repo drafts-build/build io.github.limads.Drafts.Devel.json --state-dir=drafts-build/state --force-clean --install --user
flatpak run io.github.limads.Drafts
```

## Direct cargo install

You will need to have a few system dependencies installed (they probably
are if you are using a distribution with a recent Gnome environment (>=43):

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


