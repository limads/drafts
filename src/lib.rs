/*Copyright (c) 2022 Diego da Silva Lima. All rights reserved.

This work is licensed under the terms of the GPL v3.0 License.
For a copy, see http://www.gnu.org/licenses.*/

use std::rc::Rc;
use std::cell::RefCell;
use std::boxed;
use gio;
use stateful::{Callbacks, ValuedCallbacks};

pub const APP_ID : &'static str = "io.github.limads.Drafts";

pub const RESOURCE_PREFIX : &'static str = "io/github/limads/drafts";

pub const SETTINGS_FILE : &'static str = "user.json";

pub mod ui;

pub mod manager;

pub mod typesetter;

pub mod tex;

pub mod analyzer;

pub mod state;

pub mod typst_tools;

use std::collections::HashMap;
use gtk4::*;
use gtk4::prelude::*;
use gdk_pixbuf::Pixbuf;

pub fn adjust_dimension_for_page(da : &DrawingArea, zoom_action : gio::SimpleAction, page : &poppler::Page) {
    let z = zoom_action.state().unwrap().get::<f64>().unwrap();
    let (w, h) = page.size();
    let page_w = (w * z) as i32;
    let page_h = (h * z) as i32;
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

    da.set_draw_func({
        let zoom_action = zoom_action.clone();
        move |da, ctx, _, _| {
            adjust_dimension_for_page(da, zoom_action.clone(), &page);
            draw_page_content(da, ctx, &zoom_action.clone(), &page, true);
        }
    });

    da.queue_draw();
}
