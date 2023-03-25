use std::rc::Rc;
use std::cell::RefCell;
use std::boxed;
use gio;
use stateful::{Callbacks, ValuedCallbacks};

pub const APP_ID : &'static str = "io.github.limads.Drafts";

pub const RESOURCE_PREFIX : &'static str = "io/github/limads/drafts";

pub const SETTINGS_FILE : &'static str = "user.json";

pub mod ui;

pub mod dirwatcher;

pub mod manager;

pub mod typesetter;

pub mod tex;

pub mod analyzer;

pub mod state;

pub mod typst_tools;

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

pub fn typesetting_helper() -> Result<(), String> {

    use std::io::{Read, Write};
    use std::env;
    use std::path::{Path, PathBuf};

    let mut args = env::args().skip(1);
    let mut base_path = None;
    let mut out_path = None;
    while let (Some(a), Some(b)) = (args.next(), args.next()) {
        if a == "-p" {
            let pb = PathBuf::from(&b);
            if pb.exists() && pb.is_dir() {
                base_path = Some(pb);
            } else {
                return Err(String::from("No dir at informed path"));
            }
        } else if a == "-o" {
            let pb = PathBuf::from(&b);
            if pb.exists() && pb.is_dir() {
                out_path = Some(pb);
            }
        } else {
            return Err(String::from("Unrecognized option"));
        }
    }
    let mut latex = Vec::new();
    std::io::stdin().read_to_end(&mut latex).unwrap();
    let latex = String::from_utf8(latex).unwrap();
    if let (Some(base_path), Some(out_path)) = (base_path, out_path) {
        let doc = crate::typesetter::typeset_document(&latex, &base_path, &out_path)?;
        std::io::stdout().write_all(&doc);
        Ok(())
    } else {
        Err(String::from("Missing base or output path"))
    }
}

pub fn adjust_dimension_for_page(da : &DrawingArea, zoom_action : gio::SimpleAction, page : &poppler::Page) {
    let z = zoom_action.state().unwrap().get::<f64>().unwrap();
    let (w, h) = page.size();
    let page_w = (w * z) as i32;
    let page_h = (h * z) as i32;
    println!("page dims = {:?}", (page_w, page_h));
    da.set_width_request(page_w);
    da.set_height_request(page_h);
}

pub fn draw_page_content(
    da : &DrawingArea,
    ctx : &cairo::Context,
    zoom_action : &gio::SimpleAction,
    page : &poppler::Page,
    draw_borders : bool
) {
    let z = zoom_action.state().unwrap().get::<f64>().unwrap();
    ctx.save();

    let (w, h) = (da.allocation().width() as f64, da.allocation().height() as f64);

    // Draw white background of page
    ctx.set_source_rgb(1., 1., 1.);
    ctx.rectangle(1., 1., w, h);
    ctx.fill();

    // Draw page borders

    // let color = 0.5843;
    // let grad = cairo::LinearGradient::new(0.0, 0.0, w, h);
    // grad.add_color_stop_rgba(0.0, color, color, color, 0.5);
    // grad.add_color_stop_rgba(0.5, color, color, color, 1.0);
    // Linear gradient derefs into pattern.
    // ctx.set_source(&*grad);

    if draw_borders {
        ctx.set_source_rgb(ui::PAGE_BORDER_COLOR, ui::PAGE_BORDER_COLOR, ui::PAGE_BORDER_COLOR);
        ctx.set_line_width(ui::PAGE_BORDER_WIDTH);

        // Keep 1px away from page limits
        ctx.move_to(1., 1.);
        ctx.line_to(w - 1., 1.);
        ctx.line_to(w - 1., h);
        ctx.line_to(1., h - 1.);
        ctx.line_to(1., 1.);

        ctx.stroke();
    }

    // Poppler always render with the same dpi from the physical page resolution. We must
    // apply a scale to the context if we want the content to be scaled.
    ctx.scale(z, z);

    // TODO remove the transmute when GTK/cairo version match.
    page.render(unsafe { std::mem::transmute::<_, _>(ctx) });

    ctx.restore();
}

pub fn configure_da_for_doc(da : &DrawingArea) {
    da.set_vexpand(false);
    da.set_hexpand(false);
    da.set_halign(Align::Center);
    da.set_valign(Align::Center);
    da.set_margin_top(16);
    da.set_margin_start(16);
    da.set_margin_end(16);
}

pub fn draw_page_at_area(doc : &poppler::Document, page_ix : i32, da : &DrawingArea, zoom_action : &gio::SimpleAction) {
    let page = doc.page(page_ix).unwrap();
    configure_da_for_doc(&da);
    if page_ix == doc.n_pages()-1 {
        da.set_margin_bottom(16);
    }

    //da.set_width_request((A4.0 * PX_PER_MM) as i32);
    //da.set_height_request((A4.1 * PX_PER_MM) as i32);

    da.set_draw_func({
        let zoom_action = zoom_action.clone();
        move |da, ctx, _, _| {
            adjust_dimension_for_page(da, zoom_action.clone(), &page);
            draw_page_content(da, ctx, &zoom_action.clone(), &page, true);
        }
    });

    add_page_gestures(da);
    da.queue_draw();
}

fn add_page_gestures(da : &DrawingArea) {
    /*let motion = EventControllerMotion::new();
    motion.connect_enter(|motion, x, y| {
        // let cursor_ty = gdk::CursorType::Text;
        // cursor.set_curor(Some(&gdk::Cursor::for_display(gdk::Display::default(), cursor_ty)));
    });
    motion.connect_leave(|motion| {
        // let cursor_ty = gdk::CursorType::Arrow;
        // cursor.set_curor(Some(&gdk::Cursor::for_display(gdk::Display::default(), cursor_ty)));
    });
    motion.connect_motion({
        let page = doc.page(page_ix).unwrap();
        move |motion, x, y| {
            if page.text_for_area(&mut poppler::Rectangle::new()).is_some() {
                // Text cursor
            } else {
                // Arrow cursor
            }
        }
    });
    da.add_controller(&motion);
    let drag = GestureDrag::new();
    drag.connect_drag_begin({
        let page = doc.page(page_ix).unwrap();
        move |drag, x, y| {

        }
    });
    drag.connect_drag_end({
        let page = doc.page(page_ix).unwrap();
        move |drag, x, y| {

        }
    });*/
}
