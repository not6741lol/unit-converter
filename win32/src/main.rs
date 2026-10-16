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

use convert_core::app::App;
use convert_core::{Mode, NumFormat};
use std::cell::RefCell;
use std::ffi::c_void;
use windows::Win32::Foundation::{HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{HBRUSH, UpdateWindow};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowTextLengthW,
    GetWindowTextW, HCURSOR, HICON, HMENU, IDC_ARROW, LoadCursorW, MSG, PostQuitMessage,
    RegisterClassW, SW_HIDE, SW_SHOW, SendMessageW, SetWindowTextW, ShowWindow, TranslateMessage,
    WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASS_STYLES, WNDCLASSW,
};
use windows::core::PCWSTR;

// --- Win32 constants (kept as raw numbers to avoid feature/typing churn) ---
const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_COMMAND: u32 = 0x0111;

const WS_CHILD: u32 = 0x4000_0000;
const WS_VISIBLE: u32 = 0x1000_0000;
const WS_TABSTOP: u32 = 0x0001_0000;
const WS_BORDER: u32 = 0x0080_0000;
const WS_VSCROLL: u32 = 0x0020_0000;
const ES_AUTOHSCROLL: u32 = 0x0080;
const CBS_DROPDOWNLIST: u32 = 0x0003;
const CBS_HASSTRINGS: u32 = 0x0200;
const LBS_NOTIFY: u32 = 0x0001;
const LBS_NOINTEGRALHEIGHT: u32 = 0x0100;

// Notification codes (CBN_SELCHANGE, LBN_SELCHANGE and BN_CLICKED are all 1/0).
const CBN_SELCHANGE: u32 = 1;
const LBN_SELCHANGE: u32 = 1;
const BN_CLICKED: u32 = 0;
const EN_CHANGE: u32 = 0x0300;

// Messages.
const CB_ADDSTRING: u32 = 0x0143;
const CB_GETCURSEL: u32 = 0x0147;
const CB_SETCURSEL: u32 = 0x014E;
const CB_RESETCONTENT: u32 = 0x014B;
const LB_ADDSTRING: u32 = 0x0180;
const LB_RESETCONTENT: u32 = 0x0184;
const LB_GETCURSEL: u32 = 0x0188;

// Control IDs.
const ID_MODE: usize = 100;
const ID_SEARCH: usize = 101;
const ID_CATEGORY: usize = 102;
const ID_FROM: usize = 103;
const ID_TO: usize = 104;
const ID_SWAP: usize = 105;
const ID_VALUE: usize = 106;
const ID_FORMAT: usize = 107;
const ID_RESULT: usize = 108;
const ID_COPY: usize = 109;
const ID_SAVE: usize = 110;
const ID_FAV: usize = 111;
const ID_HIST: usize = 112;
const ID_FAVS: usize = 113;

/// Handles to every control, so the command handler can reach them.
#[derive(Default, Clone, Copy)]
struct Ui {
    mode: isize,
    search: isize,
    search_lbl: isize,
    category: isize,
    cat_lbl: isize,
    from: isize,
    from_lbl: isize,
    swap: isize,
    to: isize,
    to_lbl: isize,
    value: isize,
    value_lbl: isize,
    format: isize,
    result: isize,
    fav: isize,
    hist: isize,
    favs: isize,
}

thread_local! {
    static CTRL: RefCell<Ui> = RefCell::new(Ui::default());
    static APP: RefCell<App> = RefCell::new(App::new());
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
            // Fixed-size window: WS_OVERLAPPEDWINDOW minus the thick frame and
            // maximize box, so the absolutely-positioned controls can't be
            // clipped by resizing.
            WINDOW_STYLE(0x00CA_0000),
            100,
            100,
            580,
            640,
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
            let id = wparam.0 & 0xFFFF;
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
    let instance = HINSTANCE(
        GetModuleHandleW(None)
            .map(|m| m.0)
            .unwrap_or(std::ptr::null_mut()),
    );

    let make =
        |class: &str, text: &str, style: u32, x: i32, y: i32, w: i32, h: i32, id: usize| -> isize {
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
            );
            // Never panic across the FFI boundary: fall back to a null handle.
            match child {
                Ok(h) => h.0 as isize,
                Err(_) => 0,
            }
        };

    let combo = CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_TABSTOP;
    let list = WS_BORDER | WS_VSCROLL | LBS_NOTIFY | LBS_NOINTEGRALHEIGHT | WS_TABSTOP;

    // Mode: Convert / Calculator.
    let _ = make("STATIC", "Mode:", 0, 15, 14, 45, 22, 0);
    let mode = make("COMBOBOX", "", combo, 65, 12, 130, 200, ID_MODE);
    add_combo_item(mode, "Convert");
    add_combo_item(mode, "Calculator");
    set_combo_sel(mode, 0);

    // Category search.
    let search_lbl = make("STATIC", "Search:", 0, 15, 50, 55, 22, 0);
    let search = make(
        "EDIT",
        "",
        ES_AUTOHSCROLL | WS_TABSTOP,
        75,
        48,
        250,
        24,
        ID_SEARCH,
    );

    // Category.
    let cat_lbl = make("STATIC", "Type:", 0, 15, 84, 45, 22, 0);
    let category = make("COMBOBOX", "", combo, 65, 82, 320, 200, ID_CATEGORY);

    // From / To (+ Swap).
    let from_lbl = make("STATIC", "From:", 0, 15, 118, 45, 22, 0);
    let from = make("COMBOBOX", "", combo, 65, 116, 150, 200, ID_FROM);
    let swap = make("BUTTON", "Swap", WS_TABSTOP, 222, 116, 50, 24, ID_SWAP);
    let to_lbl = make("STATIC", "To:", 0, 280, 118, 25, 22, 0);
    let to = make("COMBOBOX", "", combo, 308, 116, 150, 200, ID_TO);

    // Value.
    let value_lbl = make("STATIC", "Value:", 0, 15, 152, 55, 22, 0);
    let value = make(
        "EDIT",
        "1",
        ES_AUTOHSCROLL | WS_TABSTOP,
        75,
        150,
        300,
        24,
        ID_VALUE,
    );

    // Number format.
    let _ = make("STATIC", "Format:", 0, 15, 186, 55, 22, 0);
    let format = make("COMBOBOX", "", combo, 75, 184, 120, 200, ID_FORMAT);
    for f in NumFormat::ALL {
        add_combo_item(format, f.label());
    }
    set_combo_sel(format, 0);

    // Result + actions.
    let _ = make("STATIC", "=", 0, 15, 220, 20, 22, 0);
    let result = make("STATIC", "", 0, 40, 218, 330, 24, ID_RESULT);
    let _ = make("BUTTON", "Copy", WS_TABSTOP, 380, 217, 60, 26, ID_COPY);
    let _ = make("BUTTON", "Save", WS_TABSTOP, 445, 217, 55, 26, ID_SAVE);
    let fav = make("BUTTON", "Fav", WS_TABSTOP, 505, 217, 45, 26, ID_FAV);

    // Saved lists.
    let _ = make("STATIC", "History", 0, 15, 252, 300, 22, 0);
    let hist = make("LISTBOX", "", list, 15, 272, 530, 140, ID_HIST);
    let _ = make("STATIC", "Favorites", 0, 15, 420, 300, 22, 0);
    let favs = make("LISTBOX", "", list, 15, 440, 530, 140, ID_FAVS);

    CTRL.with(|c| {
        *c.borrow_mut() = Ui {
            mode,
            search,
            search_lbl,
            category,
            cat_lbl,
            from,
            from_lbl,
            swap,
            to,
            to_lbl,
            value,
            value_lbl,
            format,
            result,
            fav,
            hist,
            favs,
        };
    });

    reload_categories();
    reload_units();
    refresh_lists();
    apply_mode();
    recompute();
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

unsafe fn list_sel(list: isize) -> i32 {
    SendMessageW(hwnd(list), LB_GETCURSEL, None, None).0 as i32
}

unsafe fn fill_list(list: isize, items: &[String]) {
    let _ = SendMessageW(hwnd(list), LB_RESETCONTENT, None, None);
    for item in items {
        let t = wide(item);
        let _ = SendMessageW(
            hwnd(list),
            LB_ADDSTRING,
            None,
            Some(LPARAM(t.as_ptr() as isize)),
        );
    }
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

/// Put `text` on the Windows clipboard as `CF_UNICODETEXT`.
unsafe fn set_clipboard(text: &str) {
    let data = wide(text); // UTF-16, including the NUL terminator
    let bytes = data.len() * std::mem::size_of::<u16>();

    if OpenClipboard(None).is_err() {
        return;
    }
    let _ = EmptyClipboard();

    if let Ok(mem) = GlobalAlloc(GMEM_MOVEABLE, bytes) {
        let dst = GlobalLock(mem) as *mut u16;
        if !dst.is_null() {
            std::ptr::copy_nonoverlapping(data.as_ptr(), dst, data.len());
            let _ = GlobalUnlock(mem);
            // On success the system takes ownership of the memory.
            let _ = SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(mem.0)));
        }
    }

    let _ = CloseClipboard();
}

/// Refill the category dropdown from the (search-filtered) category list.
unsafe fn reload_categories() {
    let ui = CTRL.with(|c| *c.borrow());
    let (cats, sel) = APP.with(|a| {
        let a = a.borrow();
        let cats = a.filtered_categories();
        let sel = cats.iter().position(|c| *c == a.category).unwrap_or(0);
        (cats, sel)
    });

    let _ = SendMessageW(hwnd(ui.category), CB_RESETCONTENT, None, None);
    for c in &cats {
        add_combo_item(ui.category, c.label());
    }
    set_combo_sel(ui.category, sel as i32);
}

/// Refill the from/to dropdowns from the current category's units.
unsafe fn reload_units() {
    let ui = CTRL.with(|c| *c.borrow());
    let (units, from, to) = APP.with(|a| {
        let a = a.borrow();
        let units = a.units();
        (units, a.from, a.to)
    });

    let _ = SendMessageW(hwnd(ui.from), CB_RESETCONTENT, None, None);
    let _ = SendMessageW(hwnd(ui.to), CB_RESETCONTENT, None, None);
    for u in units {
        add_combo_item(ui.from, u);
        add_combo_item(ui.to, u);
    }
    set_combo_sel(ui.from, from as i32);
    set_combo_sel(ui.to, to as i32);
}

/// Repaint the history and favourites list boxes.
unsafe fn refresh_lists() {
    let ui = CTRL.with(|c| *c.borrow());
    let (history, favorites) = APP.with(|a| {
        let a = a.borrow();
        let history: Vec<String> = a.history().iter().map(|e| e.describe()).collect();
        let favorites: Vec<String> = a.favorites().iter().map(|e| e.describe()).collect();
        (history, favorites)
    });
    fill_list(ui.hist, &history);
    fill_list(ui.favs, &favorites);
}

/// Write the current result and star state to their controls.
unsafe fn recompute() {
    let ui = CTRL.with(|c| *c.borrow());
    if ui.value == 0 {
        return;
    }
    let (out, fav_label) = APP.with(|a| {
        let a = a.borrow();
        (
            a.result_text(),
            if a.is_favorite() { "Unfav" } else { "Fav" },
        )
    });
    set_text(ui.result, &out);
    set_text(ui.fav, fav_label);
}

/// Show/hide the unit widgets and relabel the input, depending on the mode.
unsafe fn apply_mode() {
    let ui = CTRL.with(|c| *c.borrow());
    let convert = APP.with(|a| a.borrow().mode == Mode::Convert);

    let show = |control: isize, visible: bool| {
        let _ = ShowWindow(hwnd(control), if visible { SW_SHOW } else { SW_HIDE });
    };
    for control in [
        ui.search_lbl,
        ui.search,
        ui.cat_lbl,
        ui.category,
        ui.from_lbl,
        ui.from,
        ui.swap,
        ui.to_lbl,
        ui.to,
    ] {
        show(control, convert);
    }

    set_text(ui.value_lbl, if convert { "Value:" } else { "Expr:" });
}

/// Load a saved entry (from history or favourites) back into the UI.
unsafe fn restore_entry(entry: convert_core::app::Entry) {
    let ui = CTRL.with(|c| *c.borrow());
    let (input, mode) = APP.with(|a| {
        let mut a = a.borrow_mut();
        a.restore(&entry);
        (a.input.clone(), a.mode)
    });

    set_text(ui.value, &input);
    set_combo_sel(ui.mode, if mode == Mode::Calculator { 1 } else { 0 });
    apply_mode();
    reload_categories();
    reload_units();
    recompute();
}

unsafe fn on_command(id: usize, code: u32) {
    let c = CTRL.with(|ui| *ui.borrow());
    // Ignore notifications that arrive during WM_CREATE, before the handles
    // are stored (e.g. the initial EN_CHANGE from the value edit).
    if c.value == 0 {
        return;
    }
    match id {
        ID_MODE if code == CBN_SELCHANGE => {
            let calculator = combo_sel(c.mode) == 1;
            let input = APP.with(|a| {
                let mut a = a.borrow_mut();
                a.set_mode(if calculator {
                    Mode::Calculator
                } else {
                    Mode::Convert
                });
                a.input.clone()
            });
            set_text(c.value, &input);
            apply_mode();
            reload_categories();
            reload_units();
            recompute();
        }
        ID_SEARCH if code == EN_CHANGE => {
            let text = get_text(c.search);
            APP.with(|a| a.borrow_mut().search = text);
            reload_categories();
        }
        ID_CATEGORY if code == CBN_SELCHANGE => {
            let idx = combo_sel(c.category).max(0) as usize;
            APP.with(|a| {
                let mut a = a.borrow_mut();
                if let Some(cat) = a.filtered_categories().get(idx).copied() {
                    a.set_category(cat);
                }
            });
            reload_units();
            recompute();
        }
        ID_FROM if code == CBN_SELCHANGE => {
            let idx = combo_sel(c.from).max(0) as usize;
            APP.with(|a| a.borrow_mut().set_from(idx));
            recompute();
        }
        ID_TO if code == CBN_SELCHANGE => {
            let idx = combo_sel(c.to).max(0) as usize;
            APP.with(|a| a.borrow_mut().set_to(idx));
            recompute();
        }
        ID_SWAP if code == BN_CLICKED => {
            APP.with(|a| a.borrow_mut().swap());
            reload_units();
            recompute();
        }
        ID_VALUE if code == EN_CHANGE => {
            let text = get_text(c.value);
            APP.with(|a| a.borrow_mut().input = text);
            recompute();
        }
        ID_FORMAT if code == CBN_SELCHANGE => {
            let idx = (combo_sel(c.format).max(0) as usize).min(NumFormat::ALL.len() - 1);
            APP.with(|a| a.borrow_mut().format = NumFormat::ALL[idx]);
            recompute();
        }
        ID_COPY if code == BN_CLICKED => {
            let text = APP.with(|a| a.borrow().result_text());
            if !text.is_empty() {
                set_clipboard(&text);
            }
        }
        ID_SAVE if code == BN_CLICKED => {
            APP.with(|a| a.borrow_mut().save());
            refresh_lists();
        }
        ID_FAV if code == BN_CLICKED => {
            APP.with(|a| a.borrow_mut().toggle_favorite());
            recompute();
            refresh_lists();
        }
        ID_HIST if code == LBN_SELCHANGE => {
            let idx = list_sel(c.hist);
            if idx >= 0 {
                let entry = APP.with(|a| a.borrow().history().get(idx as usize).cloned());
                if let Some(entry) = entry {
                    restore_entry(entry);
                }
            }
        }
        ID_FAVS if code == LBN_SELCHANGE => {
            let idx = list_sel(c.favs);
            if idx >= 0 {
                let entry = APP.with(|a| a.borrow().favorites().get(idx as usize).cloned());
                if let Some(entry) = entry {
                    restore_entry(entry);
                }
            }
        }
        _ => {}
    }
}
