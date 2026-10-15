// In release builds, hide the console window on Windows (run as a GUI app).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Raw Win32 calls are inherently unsafe; allow the terse form inside our
// `unsafe fn`s rather than wrapping every single line.
#![allow(unsafe_op_in_unsafe_fn)]

//! Unit converter built directly on the Win32 API with the `windows` crate.
//!
//! No GUI framework at all — just the raw operating-system controls, which is
//! why the resulting binary is a fraction of the size of the others.
//!
//! # Disclaimer
//!
//! **This is an ultra-experimental proof of concept.** Its only purpose is to
//! show how small a native GUI can be when you skip every framework and talk
//! to the OS directly. It is *not* a serious, maintainable application:
//!
//! * It is **Windows-only** and written with `unsafe` all over the place.
//! * It has received **no hardening, no accessibility work, and no real
//!   testing** beyond "the window opens".
//! * It should **not** be taken seriously for its capability or reliability,
//!   and must **not** be used in production.
//!
//! If you want something to actually use, reach for the `egui` or `fltk`
//! front-ends. This one is **for research and investigation by developers
//! only.**

use convert_core::{convert, fmt_num, math, Category, ALL_CATEGORIES};
use std::cell::RefCell;
use std::ffi::c_void;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{UpdateWindow, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowTextLengthW,
    GetWindowTextW, LoadCursorW, PostQuitMessage, RegisterClassW, SendMessageW, SetWindowTextW,
    ShowWindow, TranslateMessage, HCURSOR, HICON, HMENU, MSG, SW_SHOW, WNDCLASSW,
    WNDCLASS_STYLES, WINDOW_EX_STYLE, WINDOW_STYLE, IDC_ARROW,
};

// --- Win32 constants (kept as raw numbers to avoid feature/typing churn) ---
const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_COMMAND: u32 = 0x0111;

const WS_CHILD: u32 = 0x4000_0000;
const WS_VISIBLE: u32 = 0x1000_0000;
const WS_TABSTOP: u32 = 0x0001_0000;
const ES_AUTOHSCROLL: u32 = 0x0080;
const CBS_DROPDOWNLIST: u32 = 0x0003;
const CBS_HASSTRINGS: u32 = 0x0200;

const CBN_SELCHANGE: u32 = 1;
const EN_CHANGE: u32 = 0x0300;

const CB_ADDSTRING: u32 = 0x0143;
const CB_GETCURSEL: u32 = 0x0147;
const CB_SETCURSEL: u32 = 0x014E;
const CB_RESETCONTENT: u32 = 0x014B;

// Control IDs.
const ID_CATEGORY: usize = 101;
const ID_FROM: usize = 102;
const ID_TO: usize = 103;
const ID_VALUE: usize = 104;
const ID_RESULT: usize = 105;

#[derive(Default, Clone, Copy)]
struct Controls {
    category: isize,
    from: isize,
    to: isize,
    value: isize,
    result: isize,
}

thread_local! {
    static CTRL: RefCell<Controls> = RefCell::new(Controls::default());
}

fn main() {
    unsafe {
        let hmodule = GetModuleHandleW(None).unwrap();
        let instance = HINSTANCE(hmodule.0);
        let class_name = wide("ConvertWin32Class");

        let wc = WNDCLASSW {
            style: WNDCLASS_STYLES(0x0002 | 0x0001), // CS_HREDRAW | CS_VREDRAW
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: HICON(std::ptr::null_mut()),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or(HCURSOR(std::ptr::null_mut())),
            hbrBackground: HBRUSH(16 as *mut c_void), // COLOR_3DFACE + 1
            lpszMenuName: PCWSTR::null(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
        };
        RegisterClassW(&wc);

        let title = wide("Convert");
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            WINDOW_STYLE(0x00CF_0000), // WS_OVERLAPPEDWINDOW
            100,
            100,
            400,
            260,
            None,
            None,
            Some(instance),
            None,
        )
        .unwrap();

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// Encode a Rust string as a null-terminated UTF-16 buffer.
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn hwnd(v: isize) -> HWND {
    HWND(v as *mut c_void)
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            create_controls(hwnd);
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as usize;
            let code = ((wparam.0 >> 16) & 0xFFFF) as u32;
            on_command(id, code);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn create_controls(parent: HWND) {
    let instance = HINSTANCE(GetModuleHandleW(None).unwrap().0);

    let make = |class: &str, text: &str, style: u32, x: i32, y: i32, w: i32, h: i32, id: usize| -> isize {
        let class = wide(class);
        let text = wide(text);
        let child = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class.as_ptr()),
            PCWSTR(text.as_ptr()),
            WINDOW_STYLE(WS_CHILD | WS_VISIBLE | style),
            x,
            y,
            w,
            h,
            Some(parent),
            Some(HMENU(id as *mut c_void)),
            Some(instance),
            None,
        )
        .unwrap();
        child.0 as isize
    };

    let _ = make("STATIC", "Type:", 0, 15, 20, 40, 22, 0);
    let cat_box = make(
        "COMBOBOX",
        "",
        CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_TABSTOP,
        60,
        18,
        250,
        200,
        ID_CATEGORY,
    );
    for c in ALL_CATEGORIES {
        add_combo_item(cat_box, c.label());
    }
    set_combo_sel(cat_box, 0);

    let _ = make("STATIC", "From:", 0, 15, 58, 40, 22, 0);
    let from_box = make(
        "COMBOBOX",
        "",
        CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_TABSTOP,
        60,
        56,
        160,
        200,
        ID_FROM,
    );

    let _ = make("STATIC", "To:", 0, 230, 58, 25, 22, 0);
    let to_box = make(
        "COMBOBOX",
        "",
        CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_TABSTOP,
        255,
        56,
        110,
        200,
        ID_TO,
    );

    let _ = make("STATIC", "Value:", 0, 15, 98, 45, 22, 0);
    let value_box = make(
        "EDIT",
        "1",
        ES_AUTOHSCROLL | WS_TABSTOP,
        60,
        96,
        250,
        24,
        ID_VALUE,
    );

    let _ = make("STATIC", "=", 0, 15, 140, 20, 22, 0);
    let result_box = make("STATIC", "", 0, 40, 140, 320, 24, ID_RESULT);

    CTRL.with(|c| {
        *c.borrow_mut() = Controls {
            category: cat_box,
            from: from_box,
            to: to_box,
            value: value_box,
            result: result_box,
        };
    });

    fill_units(Category::Length, from_box, to_box);
    recompute();
}

/// (Re)fill the from/to combo boxes for the given category.
unsafe fn fill_units(category: Category, from_box: isize, to_box: isize) {
    let _ = SendMessageW(hwnd(from_box), CB_RESETCONTENT, None, None);
    let _ = SendMessageW(hwnd(to_box), CB_RESETCONTENT, None, None);
    for u in category.units() {
        add_combo_item(from_box, u);
        add_combo_item(to_box, u);
    }
    set_combo_sel(from_box, 0);
    set_combo_sel(to_box, 1.min(category.units().len() as i32 - 1));
}

unsafe fn add_combo_item(combo: isize, text: &str) {
    let t = wide(text);
    let _ = SendMessageW(
        hwnd(combo),
        CB_ADDSTRING,
        None,
        Some(LPARAM(t.as_ptr() as isize)),
    );
}

unsafe fn combo_sel(combo: isize) -> i32 {
    SendMessageW(hwnd(combo), CB_GETCURSEL, None, None).0 as i32
}

unsafe fn set_combo_sel(combo: isize, index: i32) {
    let _ = SendMessageW(
        hwnd(combo),
        CB_SETCURSEL,
        Some(WPARAM(index as usize)),
        None,
    );
}

unsafe fn get_text(control: isize) -> String {
    let len = GetWindowTextLengthW(hwnd(control));
    if len <= 0 {
        return String::new();
    }
    let mut buf = vec![0u16; len as usize + 1];
    let read = GetWindowTextW(hwnd(control), &mut buf);
    String::from_utf16_lossy(&buf[..read as usize])
}

unsafe fn set_text(control: isize, text: &str) {
    let t = wide(text);
    let _ = SetWindowTextW(hwnd(control), PCWSTR(t.as_ptr()));
}

/// The whole app: read current widgets, convert, write the result label.
unsafe fn recompute() {
    let c = CTRL.with(|c| *c.borrow());
    if c.value == 0 {
        return;
    }
    let cat_idx = combo_sel(c.category).max(0) as usize;
    let category = ALL_CATEGORIES[cat_idx.min(ALL_CATEGORIES.len() - 1)];
    let units = category.units();

    let from_idx = combo_sel(c.from).max(0) as usize;
    let to_idx = combo_sel(c.to).max(0) as usize;
    let from = units[from_idx.min(units.len() - 1)];
    let to = units[to_idx.min(units.len() - 1)];

    let text = get_text(c.value);
    let out = match math::eval(&text) {
        Ok(v) => format!("{} {}", fmt_num(convert(category, from, to, v)), to),
        Err(_) => {
            if text.trim().is_empty() {
                String::new()
            } else {
                "(invalid expression)".to_string()
            }
        }
    };
    set_text(c.result, &out);
}

unsafe fn on_command(id: usize, code: u32) {
    let c = CTRL.with(|c| *c.borrow());
    match id {
        ID_CATEGORY if code == CBN_SELCHANGE => {
            let idx = (combo_sel(c.category).max(0) as usize)
                .min(ALL_CATEGORIES.len() - 1);
            let category = ALL_CATEGORIES[idx];
            fill_units(category, c.from, c.to);
            recompute();
        }
        ID_FROM | ID_TO if code == CBN_SELCHANGE => recompute(),
        ID_VALUE if code == EN_CHANGE => recompute(),
        _ => {}
    }
}
