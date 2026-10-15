// In release builds, hide the console window on Windows (run as a GUI app).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use convert_core::{ALL_CATEGORIES, Category, convert, fmt_num, math};
use fltk::enums::CallbackTrigger;
use fltk::prelude::*;
use fltk::{app, button::Button, input::Input, menu::Choice, output::Output, window::Window};
use std::cell::RefCell;
use std::rc::Rc;

/// Shared state, kept in an `Rc<RefCell<..>>` so every widget callback can
/// read and update it.
struct State {
    category: Category,
    from: usize,
    to: usize,
}

/// Recompute the result from the current state + input text and show it.
fn refresh(state: &Rc<RefCell<State>>, input: &Input, result: &mut Output) {
    let s = state.borrow();
    let units = s.category.units();
    let last = units.len().saturating_sub(1);
    let f = units[s.from.min(last)];
    let t = units[s.to.min(last)];
    let text = input.value();

    let out = match math::eval(&text) {
        Ok(v) => format!("{} {}", fmt_num(convert(s.category, f, t, v)), t),
        Err(_) => {
            if text.trim().is_empty() {
                String::new()
            } else {
                "(invalid expression)".to_string()
            }
        }
    };
    result.set_value(&out);
}

fn main() {
    let app = app::App::default();
    let mut wind = Window::new(120, 120, 440, 220, "Convert");

    let mut cat = Choice::new(100, 20, 250, 28, "Type:");
    for c in ALL_CATEGORIES {
        cat.add_choice(c.label());
    }
    cat.set_value(0);

    let mut from = Choice::new(60, 62, 130, 28, "From:");
    let mut swap = Button::new(195, 62, 50, 28, "Swap");
    let mut to = Choice::new(275, 62, 130, 28, "To:");

    let mut value = Input::new(100, 105, 200, 28, "Value:");
    value.set_value("1");

    let mut result = Output::new(100, 150, 250, 28, "=");
    let mut copy = Button::new(360, 150, 70, 28, "Copy");

    // Populate the initial (Length) unit lists.
    for u in Category::Length.units() {
        from.add_choice(u);
    }
    for u in Category::Length.units() {
        to.add_choice(u);
    }
    from.set_value(0);
    to.set_value(1);

    wind.end();
    wind.show();

    let state = Rc::new(RefCell::new(State {
        category: Category::Length,
        from: 0,
        to: 1,
    }));

    // Changing the category reloads both unit lists.
    {
        let state = state.clone();
        let mut from = from.clone();
        let mut to = to.clone();
        let input = value.clone();
        let mut result = result.clone();
        cat.set_callback(move |c| {
            let idx = (c.value().max(0) as usize).min(ALL_CATEGORIES.len().saturating_sub(1));
            let newcat = ALL_CATEGORIES[idx];
            let t = 1.min(newcat.units().len().saturating_sub(1));
            {
                let mut s = state.borrow_mut();
                s.category = newcat;
                s.from = 0;
                s.to = t;
            }
            from.clear();
            for u in newcat.units() {
                from.add_choice(u);
            }
            from.set_value(0);
            to.clear();
            for u in newcat.units() {
                to.add_choice(u);
            }
            to.set_value(t as i32);
            refresh(&state, &input, &mut result);
        });
    }

    // Selecting a "from" unit.
    {
        let state = state.clone();
        let input = value.clone();
        let mut result = result.clone();
        let mut handle = from.clone();
        handle.set_callback(move |c| {
            let len = state.borrow().category.units().len();
            state.borrow_mut().from = (c.value().max(0) as usize).min(len.saturating_sub(1));
            refresh(&state, &input, &mut result);
        });
    }

    // Selecting a "to" unit.
    {
        let state = state.clone();
        let input = value.clone();
        let mut result = result.clone();
        let mut handle = to.clone();
        handle.set_callback(move |c| {
            let len = state.borrow().category.units().len();
            state.borrow_mut().to = (c.value().max(0) as usize).min(len.saturating_sub(1));
            refresh(&state, &input, &mut result);
        });
    }

    // Live conversion as the user types (fire on every change, not just Enter).
    {
        let state = state.clone();
        let input = value.clone();
        let mut result = result.clone();
        let mut handle = value.clone();
        handle.set_trigger(CallbackTrigger::Changed);
        handle.set_callback(move |_| {
            refresh(&state, &input, &mut result);
        });
    }

    // Swap from/to.
    {
        let state = state.clone();
        let mut from = from.clone();
        let mut to = to.clone();
        let input = value.clone();
        let mut result = result.clone();
        swap.set_callback(move |_| {
            let (f, t) = {
                let mut s = state.borrow_mut();
                // Manual swap: can't take two `&mut` field borrows through the
                // `RefMut` deref, so swap via a temporary.
                let tmp = s.from;
                s.from = s.to;
                s.to = tmp;
                (s.from, s.to)
            };
            from.set_value(f as i32);
            to.set_value(t as i32);
            refresh(&state, &input, &mut result);
        });
    }

    // Copy the result to the clipboard.
    {
        let result = result.clone();
        copy.set_callback(move |_| {
            fltk::app::copy(&result.value());
        });
    }

    refresh(&state, &value, &mut result);

    app.run().unwrap();
}
