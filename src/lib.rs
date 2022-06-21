use std::rc::Rc;
use std::cell::RefCell;
use std::boxed;
use gio;
use stateful::{Callbacks, ValuedCallbacks};

pub const APP_ID : &'static str = "com.github.limads.papers";

pub const RESOURCE_PREFIX : &'static str = "com/github/limads/Papers";

pub const SETTINGS_FILE : &'static str = "user.json";

pub mod ui;

pub mod dirwatcher;

pub mod manager;

pub mod typesetter;

pub mod tex;

pub mod analyzer;

pub mod state;

// pub type Callbacks<T> = Rc<RefCell<Vec<boxed::Box<dyn Fn(T) + 'static>>>>;

// pub type ValuedCallbacks<A, R> = Rc<RefCell<Vec<boxed::Box<dyn Fn(A)->R + 'static>>>>;

/*pub fn call<T>(callbacks : &Callbacks<T>, arg : T)
where
    T : Clone
{
    match callbacks.try_borrow() {
        Ok(cbs) => {
            cbs.iter().for_each(|cb| cb(arg.clone()) );
        },
        Err(e) => {
            println!("{}",e );
        }
    }
}*/

/*pub trait React<S> {

    fn react(&self, source : &S);

}*/

/*
\documentclass{...}
<<Preamble>>
\begin{document}
<<Document>>
\end{document}


\title{How to Structure a LaTeX Document}
\author{Andrew Roberts}
\date{December 2004}
\maketitle

\begin{abstract}
Your abstract goes here...
...
\end{abstract}
\renewcommand{\abstractname}{Executive Summary}

\chapter
\section
\subsection
\clearpage
\tableofcontents
\listoffigures
\listoftables

(For books)
\frontmatter

\mainmatter
\chapter{First chapter}
% ...

\appendix
\chapter{First Appendix}

\backmatter
\chapter{Last note}

\bibliography


-- Formatting
\linespread{factor}

\vfill

\clearpage

\parindent

\begin{list_type}
\item The first item
\item The second item
\item The third etc \ldots
\end{list_type}

\topmargin
\includegraphics[width=.3\linewidth]{example-image}
\footnotemark
\usepackage{hyperref}
\hyperref[label_name]{''link text''}

\footnotetext{This is my footnote!}
*/

use std::collections::HashMap;
use gtk4::*;
use gtk4::prelude::*;
use gdk_pixbuf::Pixbuf;

pub fn load_icons_as_pixbufs_from_resource(icons : &[&'static str]) -> Result<HashMap<&'static str, Pixbuf>, String> {
    if let Some(display) = gdk::Display::default() {
        if let Some(theme) = IconTheme::for_display(&display) {
            theme.add_resource_path("/com/github/limads/papers");
            theme.add_resource_path("/com/github/limads/papers/icons");
            let mut icon_pixbufs = HashMap::new();
            for icon_name in icons {
                let pxb = Pixbuf::from_resource(&format!("/com/github/limads/papers/icons/scalable/actions/{}.svg", icon_name)).unwrap();
                icon_pixbufs.insert(*icon_name,pxb);
            }
            Ok(icon_pixbufs)
        } else {
            Err(format!("No icon theme for default GDK display"))
        }
    } else {
        Err(format!("No default GDK display"))
    }
}

pub fn load_icons_as_pixbufs_from_paths(icons : &[&'static str]) -> Result<HashMap<&'static str, Pixbuf>, String> {
    if let Some(display) = gdk::Display::default() {
        if let Some(theme) = IconTheme::for_display(&display) {
            let mut icon_pixbufs = HashMap::new();
            for icon_name in icons {
                if let Some(icon) = theme.lookup_icon(icon_name, &[], 16, 1, TextDirection::Ltr, IconLookupFlags::empty()) {
                    let path = icon.file()
                        .ok_or(format!("Icon {} has no corresponing file", icon_name))?
                        .path()
                        .ok_or(format!("File for icon {} has no valid path", icon_name))?;
                        let pxb = Pixbuf::from_file_at_scale(path, 16, 16, true).unwrap();
                        icon_pixbufs.insert(*icon_name,pxb);
                } else {
                    return Err(format!("No icon named {}", icon_name));
                }
            }
            Ok(icon_pixbufs)
        } else {
            Err(format!("No icon theme for default GDK display"))
        }
    } else {
        Err(format!("No default GDK display"))
    }
}

fn read_resource() -> gio::Resource {
    gio::Resource::load("data/resources.gresource").unwrap()
}



