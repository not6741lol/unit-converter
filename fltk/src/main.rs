// In release builds, hide the console window on Windows (run as a GUI app).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Unit converter built with `fltk-rs`.
//!
//! All behaviour lives in [`convert_core::app::App`]; this file only lays the
//! widgets out and wires callbacks to it.

use convert_core::app::App;
use convert_core::{Mode, NumFormat};
use fltk::app;
use fltk::browser::HoldBrowser;
use fltk::button::Button;
use fltk::enums::CallbackTrigger;
use fltk::frame::Frame;
use fltk::input::Input;
use fltk::menu::Choice;
use fltk::output::Output;
use fltk::prelude::*;
use fltk::window::Window;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Handles to every widget, bundled so helpers can take one argument.
#[derive(Clone)]
struct Widgets {
    mode_choice: Choice,
    theme_btn: Button,
    search: Input,
    cat: Choice,
    from: Choice,
    swap: Button,
    to: Choice,
    value: Input,
    format: Choice,
    result: Output,
    copy: Button,
    save: Button,
    fav: Button,
    hist_title: Frame,
    hist: HoldBrowser,
    fav_title: Frame,
    favs: HoldBrowser,
}

/// Refill the category dropdown from the current (filtered) category list.
fn reload_categories(app: &App, w: &mut Widgets) {
    w.cat.clear();
    let cats = app.filtered_categories();
    for c in &cats {
        w.cat.add_choice(c.label());
    }
    let idx = cats.iter().position(|c| *c == app.category).unwrap_or(0) as i32;
    w.cat.set_value(idx);
}

/// Refill the from/to dropdowns from the current category's units.
fn reload_units(app: &App, w: &mut Widgets) {
    w.from.clear();
    for u in app.units() {
        w.from.add_choice(u);
    }
    w.from.set_value(app.from as i32);

    w.to.clear();
    for u in app.units() {
        w.to.add_choice(u);
    }
    w.to.set_value(app.to as i32);
}

/// Update the result line and the saved lists.
fn refresh(app: &App, w: &mut Widgets) {
    w.result.set_value(&app.result_text());
    w.fav
        .set_label(if app.is_favorite() { "Unfav" } else { "Fav" });

    w.hist.clear();
    for e in app.history() {
        w.hist.add(&e.describe());
    }
    w.hist_title
        .set_label(&format!("History ({})", app.history().len()));

    w.favs.clear();
    for e in app.favorites() {
        w.favs.add(&e.describe());
    }
    w.fav_title
        .set_label(&format!("Favorites ({})", app.favorites().len()));
}

fn refresh_all(app: &App, w: &mut Widgets) {
    reload_categories(app, w);
    reload_units(app, w);
    refresh(app, w);
}

/// Show/hide the convert-only widgets depending on the mode.
fn apply_mode(app: &App, w: &mut Widgets) {
    let convert = app.mode == Mode::Convert;
    let mut convert_only: [&mut dyn WidgetExt; 5] = [
        &mut w.search,
        &mut w.cat,
        &mut w.from,
        &mut w.swap,
        &mut w.to,
    ];
    for widget in &mut convert_only {
        if convert {
            widget.show();
        } else {
            widget.hide();
        }
    }
    w.value.set_label(if convert { "Value:" } else { "Expr:" });
}

/// Switch fltk's global palette between light and dark.
///
/// fltk has no dark scheme of its own, so we redefine the symbolic colours all
/// widgets draw with (background / background2 / foreground) and force a redraw.
fn apply_theme(dark: bool) {
    if dark {
        app::background(43, 43, 46);
        app::background2(28, 28, 31);
        app::foreground(235, 235, 235);
        app::set_selection_color(90, 130, 210);
        app::set_inactive_color(95, 95, 100);
    } else {
        app::background(238, 238, 238);
        app::background2(255, 255, 255);
        app::foreground(0, 0, 0);
        app::set_selection_color(0, 0, 255);
        app::set_inactive_color(150, 150, 150);
    }
    app::redraw();
}

fn main() {
    let app_handle = app::App::default();
    let mut wind = Window::new(80, 80, 580, 600, "Convert");

    let mut mode_choice = Choice::new(80, 10, 130, 28, "Mode:");
    mode_choice.add_choice("Convert");
    mode_choice.add_choice("Calculator");
    mode_choice.set_value(0);
    let theme_btn = Button::new(480, 10, 80, 28, "Theme");

    let search = Input::new(90, 46, 250, 28, "Search:");
    let cat = Choice::new(90, 80, 320, 28, "Type:");
    let from = Choice::new(80, 114, 160, 28, "From:");
    let swap = Button::new(250, 114, 55, 28, "Swap");
    let to = Choice::new(330, 114, 160, 28, "To:");

    let mut value = Input::new(90, 148, 320, 28, "Value:");
    value.set_value("1");

    let mut format = Choice::new(90, 182, 120, 28, "Format:");
    for f in NumFormat::ALL {
        format.add_choice(f.label());
    }
    format.set_value(0);

    let result = Output::new(30, 216, 350, 28, "=");
    let copy = Button::new(390, 216, 65, 28, "Copy");
    let save = Button::new(460, 216, 60, 28, "Save");
    let fav = Button::new(525, 216, 55, 28, "Fav");

    let mut hist_title = Frame::new(20, 250, 300, 22, "History");
    hist_title.set_label_size(13);
    let hist = HoldBrowser::new(20, 274, 550, 140, "");
    let mut fav_title = Frame::new(20, 420, 300, 22, "Favorites");
    fav_title.set_label_size(13);
    let favs = HoldBrowser::new(20, 444, 550, 140, "");

    wind.end();
    wind.show();

    let mut widgets = Widgets {
        mode_choice,
        theme_btn,
        search,
        cat,
        from,
        swap,
        to,
        value,
        format,
        result,
        copy,
        save,
        fav,
        hist_title,
        hist,
        fav_title,
        favs,
    };

    let state = Rc::new(RefCell::new(App::new()));
    // Set while we update widgets ourselves, so their callbacks don't re-enter.
    let busy = Rc::new(Cell::new(false));

    // Initial paint.
    {
        let s = state.borrow();
        refresh_all(&s, &mut widgets);
        apply_mode(&s, &mut widgets);
    }
    apply_theme(state.borrow().dark);

    // --- mode switch: also swaps the input's label and hides unit widgets ---
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut w = widgets.clone();
        let mut handle = widgets.mode_choice.clone();
        handle.set_callback(move |c| {
            if busy.get() {
                return;
            }
            busy.set(true);
            let calculator = c.value() == 1;
            {
                let mut s = state.borrow_mut();
                s.set_mode(if calculator {
                    Mode::Calculator
                } else {
                    Mode::Convert
                });
                let input = s.input.clone();
                w.value.set_value(&input);
            }
            {
                let s = state.borrow();
                refresh_all(&s, &mut w);
                apply_mode(&s, &mut w);
            }
            app::redraw();
            busy.set(false);
        });
    }

    // --- theme ---
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut handle = widgets.theme_btn.clone();
        handle.set_callback(move |_| {
            if busy.get() {
                return;
            }
            busy.set(true);
            let dark = {
                let mut s = state.borrow_mut();
                s.toggle_dark();
                s.dark
            };
            apply_theme(dark);
            app::redraw();
            busy.set(false);
            app::redraw();
            busy.set(false);
        });
    }

    // --- search box filters the category list ---
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut w = widgets.clone();
        let mut handle = widgets.search.clone();
        handle.set_trigger(CallbackTrigger::Changed);
        handle.set_callback(move |c| {
            if busy.get() {
                return;
            }
            busy.set(true);
            state.borrow_mut().search = c.value();
            {
                let s = state.borrow();
                reload_categories(&s, &mut w);
            }
            app::redraw();
            busy.set(false);
        });
    }

    // --- category ---
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut w = widgets.clone();
        let mut handle = widgets.cat.clone();
        handle.set_callback(move |c| {
            if busy.get() {
                return;
            }
            busy.set(true);
            let idx = c.value().max(0) as usize;
            {
                let mut s = state.borrow_mut();
                if let Some(cat) = s.filtered_categories().get(idx).copied() {
                    s.set_category(cat);
                }
            }
            {
                let s = state.borrow();
                reload_units(&s, &mut w);
                refresh(&s, &mut w);
            }
            app::redraw();
            busy.set(false);
        });
    }

    // --- from / to units ---
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut w = widgets.clone();
        let mut handle = widgets.from.clone();
        handle.set_callback(move |c| {
            if busy.get() {
                return;
            }
            busy.set(true);
            state.borrow_mut().set_from(c.value().max(0) as usize);
            {
                let s = state.borrow();
                refresh(&s, &mut w);
            }
            app::redraw();
            busy.set(false);
        });
    }
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut w = widgets.clone();
        let mut handle = widgets.to.clone();
        handle.set_callback(move |c| {
            if busy.get() {
                return;
            }
            busy.set(true);
            state.borrow_mut().set_to(c.value().max(0) as usize);
            {
                let s = state.borrow();
                refresh(&s, &mut w);
            }
            app::redraw();
            busy.set(false);
        });
    }

    // --- swap ---
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut w = widgets.clone();
        let mut handle = widgets.swap.clone();
        handle.set_callback(move |_| {
            if busy.get() {
                return;
            }
            busy.set(true);
            state.borrow_mut().swap();
            {
                let s = state.borrow();
                reload_units(&s, &mut w);
                refresh(&s, &mut w);
            }
            app::redraw();
            busy.set(false);
        });
    }

    // --- value / expression ---
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut w = widgets.clone();
        let mut handle = widgets.value.clone();
        handle.set_trigger(CallbackTrigger::Changed);
        handle.set_callback(move |c| {
            if busy.get() {
                return;
            }
            busy.set(true);
            state.borrow_mut().input = c.value();
            {
                let s = state.borrow();
                refresh(&s, &mut w);
            }
            app::redraw();
            busy.set(false);
        });
    }

    // --- number format ---
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut w = widgets.clone();
        let mut handle = widgets.format.clone();
        handle.set_callback(move |c| {
            if busy.get() {
                return;
            }
            busy.set(true);
            let idx = (c.value().max(0) as usize).min(NumFormat::ALL.len() - 1);
            state.borrow_mut().format = NumFormat::ALL[idx];
            {
                let s = state.borrow();
                refresh(&s, &mut w);
            }
            app::redraw();
            busy.set(false);
        });
    }

    // --- copy / save / favourite ---
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut handle = widgets.copy.clone();
        handle.set_callback(move |_| {
            if busy.get() {
                return;
            }
            let text = state.borrow().result_text();
            if !text.is_empty() {
                app::copy(&text);
            }
        });
    }
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut w = widgets.clone();
        let mut handle = widgets.save.clone();
        handle.set_callback(move |_| {
            if busy.get() {
                return;
            }
            busy.set(true);
            state.borrow_mut().save();
            {
                let s = state.borrow();
                refresh(&s, &mut w);
            }
            app::redraw();
            busy.set(false);
        });
    }
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut w = widgets.clone();
        let mut handle = widgets.fav.clone();
        handle.set_callback(move |_| {
            if busy.get() {
                return;
            }
            busy.set(true);
            state.borrow_mut().toggle_favorite();
            {
                let s = state.borrow();
                refresh(&s, &mut w);
            }
            app::redraw();
            busy.set(false);
        });
    }

    // --- clicking a saved entry restores it ---
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut w = widgets.clone();
        let mut handle = widgets.hist.clone();
        handle.set_callback(move |c| {
            if busy.get() {
                return;
            }
            let idx = c.value();
            if idx <= 0 {
                return;
            }
            busy.set(true);
            let entry = state.borrow().history().get((idx - 1) as usize).cloned();
            if let Some(entry) = entry {
                state.borrow_mut().restore(&entry);
                let input = state.borrow().input.clone();
                w.value.set_value(&input);
                let s = state.borrow();
                refresh_all(&s, &mut w);
                apply_mode(&s, &mut w);
            }
            app::redraw();
            busy.set(false);
        });
    }
    {
        let state = state.clone();
        let busy = busy.clone();
        let mut w = widgets.clone();
        let mut handle = widgets.favs.clone();
        handle.set_callback(move |c| {
            if busy.get() {
                return;
            }
            let idx = c.value();
            if idx <= 0 {
                return;
            }
            busy.set(true);
            let entry = state.borrow().favorites().get((idx - 1) as usize).cloned();
            if let Some(entry) = entry {
                state.borrow_mut().restore(&entry);
                let input = state.borrow().input.clone();
                w.value.set_value(&input);
                let s = state.borrow();
                refresh_all(&s, &mut w);
                apply_mode(&s, &mut w);
            }
            app::redraw();
            busy.set(false);
        });
    }

    app_handle.run().unwrap();
}
