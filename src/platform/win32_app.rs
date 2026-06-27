use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{
    GetLastError, GlobalFree, HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM,
};
use windows_sys::Win32::Graphics::Gdi::{
    AddFontMemResourceEx, BeginPaint, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, COLOR_WINDOW,
    CreateFontW, CreatePen, CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_GUI_FONT, DEFAULT_PITCH,
    DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteObject, DrawTextW, EndPaint, FW_BOLD, FW_NORMAL,
    FillRect, GetStockObject, GetSysColor, GetSysColorBrush, HBRUSH, HDC, HFONT, HGDIOBJ,
    OUT_DEFAULT_PRECIS, PAINTSTRUCT, PS_SOLID, RemoveFontMemResourceEx, RoundRect, SelectObject,
    SetBkColor, SetBkMode, SetTextColor, TRANSPARENT,
};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
    SetClipboardData,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Memory::{
    GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
};
use windows_sys::Win32::System::Ole::CF_UNICODETEXT;
use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
use windows_sys::Win32::UI::Controls::{
    DRAWITEMSTRUCT, EM_SETMARGINS, ICC_WIN95_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx,
    ODS_FOCUS, ODS_SELECTED, ODT_BUTTON, TOOLTIPS_CLASSW, TTF_IDISHWND, TTF_SUBCLASS, TTM_ADDTOOLW,
    TTM_SETMAXTIPWIDTH, TTS_ALWAYSTIP, TTS_NOPREFIX, TTTOOLINFOW,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetFocus};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BS_OWNERDRAW, CREATESTRUCTW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow,
    DispatchMessageW, EC_LEFTMARGIN, EC_RIGHTMARGIN, EN_CHANGE, ES_NOHIDESEL, ES_READONLY,
    GetMessageW, GetWindowTextLengthW, GetWindowTextW, HICON, HMENU, HTCAPTION, HWND_NOTOPMOST,
    HWND_TOPMOST, ICON_BIG, ICON_SMALL, IDC_ARROW, IDI_APPLICATION, LoadCursorW, LoadIconW,
    MB_ICONERROR, MB_OK, MSG, MessageBoxW, PostMessageW, PostQuitMessage, RegisterClassW,
    SW_MINIMIZE, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, SendMessageW, SetWindowPos, SetWindowTextW,
    ShowWindow, TranslateMessage, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLOREDIT,
    WM_CTLCOLORSTATIC, WM_DESTROY, WM_DRAWITEM, WM_LBUTTONDOWN, WM_NCLBUTTONDOWN, WM_PAINT,
    WM_SETFONT, WM_SETICON, WNDCLASSW, WS_CHILD, WS_EX_CLIENTEDGE, WS_EX_TOPMOST, WS_POPUP,
    WS_TABSTOP, WS_VISIBLE,
};

use crate::hash;

const INPUT_ID: i32 = 101;
const OUTPUT_ID: i32 = 102;
const PASTE_ID: i32 = 103;
const COPY_ID: i32 = 104;
const MINIMIZE_ID: i32 = 105;
const TOPMOST_ID: i32 = 106;
const CLOSE_ID: i32 = 107;

const WINDOW_WIDTH: i32 = 288;
const WINDOW_HEIGHT: i32 = 250;
const FIELD_WIDTH: i32 = 170;
const FIELD_HEIGHT: i32 = 34;
const TITLE_BUTTON_SIZE: i32 = 28;
const ICON_BUTTON_WIDTH: i32 = 42;
const APP_ICON_ID: usize = 1;
const COPY_ICON: &str = "\u{E8C8}";
const COPIED_ICON: &str = "\u{E73E}";
const CLOSE_ICON: &str = "\u{E8BB}";
const MINIMIZE_ICON: &str = "\u{E921}";
const PASTE_ICON: &str = "\u{E77F}";
const PIN_ICON: &str = "\u{E718}";
const PINNED_ICON: &str = "\u{E840}";
const FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");

thread_local! {
    static UI: RefCell<UiHandles> = RefCell::new(UiHandles::default());
    static STARTUP_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[derive(Default)]
struct UiHandles {
    input: HWND,
    output: HWND,
    topmost: HWND,
    copy: HWND,
    tooltip: HWND,
    bg_brush: HBRUSH,
    field_brush: HBRUSH,
    font_resource: HANDLE,
    label_font: HFONT,
    icon_font: HFONT,
    input_font: HFONT,
    output_font: HFONT,
    title_font: HFONT,
    theme: Theme,
    hex: String,
    tooltip_text_storage: Vec<Vec<u16>>,
    is_topmost: bool,
}

struct UiResources {
    bg_brush: HBRUSH,
    field_brush: HBRUSH,
    font_resource: HANDLE,
    label_font: HFONT,
    icon_font: HFONT,
    input_font: HFONT,
    output_font: HFONT,
    title_font: HFONT,
}

#[derive(Clone, Copy)]
struct Theme {
    background: u32,
    field: u32,
    text: u32,
    muted_text: u32,
    accent: u32,
    accent_alt: u32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_system()
    }
}

impl Theme {
    fn from_system() -> Self {
        let is_dark = windows_apps_use_light_theme()
            .map(|uses_light| !uses_light)
            .unwrap_or_else(|| is_dark_color(unsafe { GetSysColor(COLOR_WINDOW) }));

        if is_dark {
            Self {
                background: rgb(7, 18, 20),
                field: rgb(9, 43, 44),
                text: rgb(222, 248, 246),
                muted_text: rgb(118, 190, 184),
                accent: rgb(0, 214, 222),
                accent_alt: rgb(123, 236, 33),
            }
        } else {
            Self {
                background: rgb(232, 242, 240),
                field: rgb(222, 248, 244),
                text: rgb(8, 29, 32),
                muted_text: rgb(42, 101, 102),
                accent: rgb(0, 173, 195),
                accent_alt: rgb(95, 190, 30),
            }
        }
    }
}

pub fn run() -> Result<(), String> {
    unsafe {
        let hinstance = GetModuleHandleW(null());
        if hinstance.is_null() {
            return Err(last_error("GetModuleHandleW failed"));
        }

        let common_controls = INITCOMMONCONTROLSEX {
            dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_WIN95_CLASSES,
        };
        InitCommonControlsEx(&common_controls);

        let class_name = wide("AdlerITWindow");
        let icon = load_app_icon(hinstance);
        let wnd_class = WNDCLASSW {
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hIcon: icon,
            hInstance: hinstance,
            hbrBackground: (COLOR_WINDOW + 1) as _,
            lpszClassName: class_name.as_ptr(),
            lpfnWndProc: Some(window_proc),
            ..Default::default()
        };

        if RegisterClassW(&wnd_class) == 0 {
            return Err(last_error("RegisterClassW failed"));
        }

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            wide("AdlerIT").as_ptr(),
            WS_POPUP | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            null_mut(),
            null_mut(),
            hinstance,
            null(),
        );
        if hwnd.is_null() {
            if let Some(error) = STARTUP_ERROR.with(|error| error.borrow_mut().take()) {
                return Err(error);
            }
            return Err(last_error("CreateWindowExW failed"));
        }

        set_window_icons(hwnd, icon);
        ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        loop {
            let result = GetMessageW(&mut msg, null_mut(), 0, 0);
            if result == -1 {
                return Err(last_error("GetMessageW failed"));
            }
            if result == 0 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

unsafe fn load_app_icon(hinstance: HINSTANCE) -> HICON {
    let icon = unsafe { LoadIconW(hinstance, resource_id(APP_ICON_ID)) };
    if icon.is_null() {
        unsafe { LoadIconW(null_mut(), IDI_APPLICATION) }
    } else {
        icon
    }
}

unsafe fn set_window_icons(hwnd: HWND, icon: HICON) {
    if icon.is_null() {
        return;
    }
    unsafe {
        SendMessageW(hwnd, WM_SETICON, ICON_BIG as WPARAM, icon as LPARAM);
        SendMessageW(hwnd, WM_SETICON, ICON_SMALL as WPARAM, icon as LPARAM);
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CREATE => {
            let create = lparam as *const CREATESTRUCTW;
            let hinstance = if create.is_null() {
                null_mut()
            } else {
                unsafe { (*create).hInstance }
            };
            match unsafe { create_controls(hwnd, hinstance) } {
                Ok(()) => unsafe {
                    update_checksum();
                },
                Err(error) => {
                    STARTUP_ERROR.with(|startup_error| {
                        *startup_error.borrow_mut() = Some(error);
                    });
                    return -1;
                }
            }
            0
        }
        WM_COMMAND => {
            let id = loword(wparam);
            let notification = hiword(wparam);
            if id == INPUT_ID && notification == EN_CHANGE {
                unsafe {
                    update_checksum();
                }
                return 0;
            }
            if id == COPY_ID {
                let hex = UI.with(|ui| ui.borrow().hex.clone());
                if hex.is_empty() {
                    return 0;
                }
                let copy = UI.with(|ui| ui.borrow().copy);
                let copied = copy_to_clipboard(hwnd, &hex);
                if copied {
                    let label = wide(COPIED_ICON);
                    unsafe {
                        SetWindowTextW(copy, label.as_ptr());
                    }
                } else {
                    unsafe {
                        error_box(hwnd, "Could not copy checksum to the clipboard.");
                    }
                }
                return 0;
            }
            if id == MINIMIZE_ID {
                unsafe {
                    ShowWindow(hwnd, SW_MINIMIZE);
                }
                return 0;
            }
            if id == TOPMOST_ID {
                toggle_topmost(hwnd);
                return 0;
            }
            if id == CLOSE_ID {
                unsafe {
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                }
                return 0;
            }
            if id == PASTE_ID {
                match read_clipboard_text(hwnd) {
                    Some(text) => {
                        let input = UI.with(|ui| ui.borrow().input);
                        let text = wide(&text);
                        unsafe {
                            SetWindowTextW(input, text.as_ptr());
                            update_checksum();
                        }
                    }
                    None => unsafe {
                        error_box(hwnd, "The clipboard does not contain text.");
                    },
                }
                return 0;
            }
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        WM_LBUTTONDOWN => {
            let x = loword_signed(lparam);
            let y = hiword_signed(lparam);
            if is_draggable_title_area(x, y) {
                unsafe {
                    ReleaseCapture();
                    SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as WPARAM, 0);
                }
                return 0;
            }
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as HDC;
            let brush = UI.with(|ui| {
                let ui = ui.borrow();
                if lparam as HWND == ui.output {
                    unsafe {
                        SetBkColor(hdc, ui.theme.field);
                        SetTextColor(hdc, ui.theme.accent);
                    }
                    return ui.field_brush;
                }

                unsafe {
                    SetBkMode(hdc, TRANSPARENT as i32);
                    SetTextColor(hdc, ui.theme.muted_text);
                }
                ui.bg_brush
            });
            brush_or_system(brush, COLOR_WINDOW) as LRESULT
        }
        WM_DRAWITEM => {
            let draw = lparam as *const DRAWITEMSTRUCT;
            if !draw.is_null() && unsafe { (*draw).CtlType } == ODT_BUTTON {
                unsafe {
                    draw_icon_button(&*draw);
                }
                return 1;
            }
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        WM_CTLCOLOREDIT => {
            let hdc = wparam as HDC;
            let brush = UI.with(|ui| {
                let ui = ui.borrow();
                unsafe {
                    SetBkColor(hdc, ui.theme.field);
                    SetTextColor(hdc, ui.theme.text);
                }
                if lparam as HWND == ui.input {
                    unsafe {
                        SetTextColor(hdc, ui.theme.muted_text);
                    }
                }
                if lparam as HWND == ui.output {
                    unsafe {
                        SetTextColor(hdc, ui.theme.accent);
                    }
                }
                ui.field_brush
            });
            brush_or_system(brush, COLOR_WINDOW) as LRESULT
        }
        WM_PAINT => {
            unsafe {
                paint_accent(hwnd);
            }
            0
        }
        WM_DESTROY => {
            UI.with(|ui| unsafe {
                let ui = ui.borrow();
                if !ui.bg_brush.is_null() {
                    DeleteObject(ui.bg_brush as HGDIOBJ);
                }
                if !ui.field_brush.is_null() {
                    DeleteObject(ui.field_brush as HGDIOBJ);
                }
                if !ui.tooltip.is_null() {
                    DestroyWindow(ui.tooltip);
                }
                if !ui.input_font.is_null() {
                    DeleteObject(ui.input_font as HGDIOBJ);
                }
                if !ui.output_font.is_null() {
                    DeleteObject(ui.output_font as HGDIOBJ);
                }
                if !ui.label_font.is_null() {
                    DeleteObject(ui.label_font as HGDIOBJ);
                }
                if !ui.icon_font.is_null() {
                    DeleteObject(ui.icon_font as HGDIOBJ);
                }
                if !ui.title_font.is_null() {
                    DeleteObject(ui.title_font as HGDIOBJ);
                }
                if !ui.font_resource.is_null() {
                    RemoveFontMemResourceEx(ui.font_resource);
                }
            });
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe fn create_controls(hwnd: HWND, hinstance: HINSTANCE) -> Result<(), String> {
    let label_font = create_system_font(-15, FW_NORMAL);
    let icon_font = create_icon_font();
    let title_font = create_system_font(-24, FW_BOLD);
    let font_resource = load_jetbrains_mono();
    let input_font = create_mono_font(-16);
    let output_font = create_mono_font(-19);
    let theme = Theme::from_system();
    let bg_brush = unsafe { CreateSolidBrush(theme.background) };
    let field_brush = unsafe { CreateSolidBrush(theme.field) };
    let resources = UiResources {
        bg_brush,
        field_brush,
        font_resource,
        label_font,
        icon_font,
        input_font,
        output_font,
        title_font,
    };

    if let Err(error) = validate_gdi_resources(&resources) {
        unsafe {
            cleanup_resources(&resources);
        }
        return Err(error);
    }

    let title = create_static(hwnd, hinstance, "AdlerIT", 24, 16, 120, 30);
    if !title.is_null() {
        unsafe {
            SendMessageW(title, WM_SETFONT, title_font as WPARAM, 1);
        }
    }
    let subtitle = create_static(hwnd, hinstance, "Adler-32 calculator", 24, 46, 160, 22);

    let minimize = create_icon_button(hwnd, hinstance, MINIMIZE_ICON, 184, 10, MINIMIZE_ID);
    let topmost = create_icon_button(hwnd, hinstance, PIN_ICON, 216, 10, TOPMOST_ID);
    let close = create_icon_button(hwnd, hinstance, CLOSE_ICON, 248, 10, CLOSE_ID);

    let input_label = create_static(hwnd, hinstance, "Input", 24, 84, 120, 20);

    let input = unsafe {
        CreateWindowExW(
            WS_EX_CLIENTEDGE,
            wide("EDIT").as_ptr(),
            wide("").as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_AUTOHSCROLL_COMPAT | ES_NOHIDESEL as u32,
            24,
            108,
            FIELD_WIDTH,
            FIELD_HEIGHT,
            hwnd,
            INPUT_ID as HMENU,
            hinstance,
            null(),
        )
    };

    let paste = unsafe {
        CreateWindowExW(
            0,
            wide("BUTTON").as_ptr(),
            wide(PASTE_ICON).as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW as u32,
            210,
            108,
            ICON_BUTTON_WIDTH,
            FIELD_HEIGHT,
            hwnd,
            PASTE_ID as HMENU,
            hinstance,
            null(),
        )
    };

    let output_label = create_static(hwnd, hinstance, "Adler-32", 24, 156, 120, 20);

    let output = unsafe {
        CreateWindowExW(
            WS_EX_CLIENTEDGE,
            wide("EDIT").as_ptr(),
            wide("").as_ptr(),
            WS_CHILD
                | WS_VISIBLE
                | ES_READONLY as u32
                | ES_AUTOHSCROLL_COMPAT
                | ES_NOHIDESEL as u32,
            24,
            180,
            FIELD_WIDTH,
            FIELD_HEIGHT,
            hwnd,
            OUTPUT_ID as HMENU,
            hinstance,
            null(),
        )
    };

    let copy = unsafe {
        CreateWindowExW(
            0,
            wide("BUTTON").as_ptr(),
            wide(COPY_ICON).as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW as u32,
            210,
            180,
            ICON_BUTTON_WIDTH,
            FIELD_HEIGHT,
            hwnd,
            COPY_ID as HMENU,
            hinstance,
            null(),
        )
    };
    let controls = [
        (title, "title label"),
        (subtitle, "subtitle label"),
        (minimize, "minimize button"),
        (topmost, "topmost button"),
        (close, "close button"),
        (input_label, "input label"),
        (input, "input field"),
        (paste, "paste button"),
        (output_label, "output label"),
        (output, "output field"),
        (copy, "copy button"),
    ];
    if let Err(error) = validate_controls(&controls) {
        unsafe {
            cleanup_resources(&resources);
        }
        return Err(error);
    }

    let (tooltip, tooltip_text_storage) = unsafe {
        create_tooltips(
            hwnd,
            hinstance,
            &[
                (minimize, "Minimize"),
                (topmost, "Keep on top"),
                (close, "Close"),
                (paste, "Paste"),
                (copy, "Copy checksum"),
            ],
        )
    };

    for control in [subtitle, input_label, output_label] {
        unsafe {
            SendMessageW(control, WM_SETFONT, label_font as WPARAM, 1);
        }
    }
    for control in [minimize, topmost, close, paste, copy] {
        unsafe {
            SendMessageW(control, WM_SETFONT, icon_font as WPARAM, 1);
        }
    }
    unsafe {
        SendMessageW(input, WM_SETFONT, input_font as WPARAM, 1);
        SendMessageW(output, WM_SETFONT, output_font as WPARAM, 1);
        SendMessageW(
            input,
            EM_SETMARGINS,
            (EC_LEFTMARGIN | EC_RIGHTMARGIN) as WPARAM,
            edit_margins(8, 8),
        );
        SendMessageW(
            output,
            EM_SETMARGINS,
            (EC_LEFTMARGIN | EC_RIGHTMARGIN) as WPARAM,
            edit_margins(8, 8),
        );
    }

    UI.with(|ui| {
        *ui.borrow_mut() = UiHandles {
            input,
            output,
            topmost,
            copy,
            tooltip,
            bg_brush,
            field_brush,
            font_resource,
            label_font,
            icon_font,
            input_font,
            output_font,
            title_font,
            theme,
            hex: String::new(),
            tooltip_text_storage,
            is_topmost: false,
        };
    });

    unsafe {
        SetFocus(input);
    }

    Ok(())
}

fn validate_gdi_resources(resources: &UiResources) -> Result<(), String> {
    let required = [
        (resources.bg_brush as *mut c_void, "background brush"),
        (resources.field_brush as *mut c_void, "field brush"),
        (resources.label_font as *mut c_void, "label font"),
        (resources.icon_font as *mut c_void, "icon font"),
        (resources.input_font as *mut c_void, "input font"),
        (resources.output_font as *mut c_void, "output font"),
        (resources.title_font as *mut c_void, "title font"),
    ];

    for (handle, name) in required {
        if handle.is_null() {
            return Err(format!("Could not create {name}."));
        }
    }

    Ok(())
}

fn validate_controls(controls: &[(HWND, &str)]) -> Result<(), String> {
    for &(control, name) in controls {
        if control.is_null() {
            return Err(format!("Could not create {name}."));
        }
    }

    Ok(())
}

unsafe fn cleanup_resources(resources: &UiResources) {
    if !resources.bg_brush.is_null() {
        unsafe {
            DeleteObject(resources.bg_brush as HGDIOBJ);
        }
    }
    if !resources.field_brush.is_null() {
        unsafe {
            DeleteObject(resources.field_brush as HGDIOBJ);
        }
    }
    if !resources.input_font.is_null() {
        unsafe {
            DeleteObject(resources.input_font as HGDIOBJ);
        }
    }
    if !resources.output_font.is_null() {
        unsafe {
            DeleteObject(resources.output_font as HGDIOBJ);
        }
    }
    if !resources.label_font.is_null() {
        unsafe {
            DeleteObject(resources.label_font as HGDIOBJ);
        }
    }
    if !resources.icon_font.is_null() {
        unsafe {
            DeleteObject(resources.icon_font as HGDIOBJ);
        }
    }
    if !resources.title_font.is_null() {
        unsafe {
            DeleteObject(resources.title_font as HGDIOBJ);
        }
    }
    if !resources.font_resource.is_null() {
        unsafe {
            RemoveFontMemResourceEx(resources.font_resource);
        }
    }
}

unsafe fn paint_accent(hwnd: HWND) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
    UI.with(|ui| {
        let ui = ui.borrow();
        let bg_brush = brush_or_system(ui.bg_brush, COLOR_WINDOW);
        let background = RECT {
            left: 0,
            top: 0,
            right: WINDOW_WIDTH,
            bottom: WINDOW_HEIGHT,
        };
        let left = RECT {
            left: 0,
            top: 0,
            right: 4,
            bottom: WINDOW_HEIGHT,
        };
        let underline = RECT {
            left: 24,
            top: 72,
            right: 252,
            bottom: 75,
        };
        let underline_glow = RECT {
            left: 204,
            top: 72,
            right: 252,
            bottom: 75,
        };

        unsafe {
            FillRect(hdc, &background, bg_brush);
            fill_with_color(hdc, &left, ui.theme.accent);
            fill_with_color(hdc, &underline, ui.theme.accent);
            fill_with_color(hdc, &underline_glow, ui.theme.accent_alt);
        }
    });
    unsafe {
        EndPaint(hwnd, &ps);
    }
}

unsafe fn fill_with_color(hdc: HDC, rect: &RECT, color: u32) {
    let brush = unsafe { CreateSolidBrush(color) };
    if !brush.is_null() {
        unsafe {
            FillRect(hdc, rect, brush);
            DeleteObject(brush as HGDIOBJ);
        }
    }
}

unsafe fn draw_icon_button(draw: &DRAWITEMSTRUCT) {
    let (theme, icon_font) = UI.with(|ui| {
        let ui = ui.borrow();
        (ui.theme, ui.icon_font)
    });
    let selected = draw.itemState & ODS_SELECTED != 0;
    let focused = draw.itemState & ODS_FOCUS != 0;
    let fill = if selected {
        theme.accent_alt
    } else {
        theme.field
    };
    let stroke = if selected || focused {
        theme.accent_alt
    } else {
        theme.accent
    };
    let text = if selected {
        theme.background
    } else {
        theme.accent
    };

    let brush = unsafe { CreateSolidBrush(fill) };
    let pen = unsafe { CreatePen(PS_SOLID, 1, stroke) };
    if brush.is_null() || pen.is_null() {
        if !brush.is_null() {
            unsafe {
                DeleteObject(brush as HGDIOBJ);
            }
        }
        if !pen.is_null() {
            unsafe {
                DeleteObject(pen as HGDIOBJ);
            }
        }
        return;
    }

    let old_brush = unsafe { SelectObject(draw.hDC, brush as HGDIOBJ) };
    let old_pen = unsafe { SelectObject(draw.hDC, pen as HGDIOBJ) };
    unsafe {
        RoundRect(
            draw.hDC,
            draw.rcItem.left,
            draw.rcItem.top,
            draw.rcItem.right,
            draw.rcItem.bottom,
            6,
            6,
        );
        SelectObject(draw.hDC, old_brush);
        SelectObject(draw.hDC, old_pen);
        DeleteObject(brush as HGDIOBJ);
        DeleteObject(pen as HGDIOBJ);
    }

    let label = wide(&unsafe { get_window_text(draw.hwndItem) });
    let mut text_rect = draw.rcItem;
    if selected {
        text_rect.left += 1;
        text_rect.top += 1;
    }
    let old_font = unsafe { SelectObject(draw.hDC, icon_font as HGDIOBJ) };
    unsafe {
        SetBkMode(draw.hDC, TRANSPARENT as i32);
        SetTextColor(draw.hDC, text);
        DrawTextW(
            draw.hDC,
            label.as_ptr(),
            -1,
            &mut text_rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        SelectObject(draw.hDC, old_font);
    }
}

fn load_jetbrains_mono() -> HANDLE {
    let mut font_count = 0u32;
    unsafe {
        AddFontMemResourceEx(
            FONT_BYTES.as_ptr() as *const c_void,
            FONT_BYTES.len() as u32,
            null(),
            &mut font_count as *mut u32,
        )
    }
}

fn create_mono_font(height: i32) -> HFONT {
    unsafe {
        CreateFontW(
            height,
            0,
            0,
            0,
            FW_NORMAL as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            CLEARTYPE_QUALITY as u32,
            DEFAULT_PITCH as u32,
            wide("JetBrains Mono").as_ptr(),
        )
    }
}

fn create_system_font(height: i32, weight: u32) -> HFONT {
    create_named_font(height, weight, "Segoe UI")
}

fn create_icon_font() -> HFONT {
    create_named_font(-18, FW_NORMAL, "Segoe MDL2 Assets")
}

fn create_named_font(height: i32, weight: u32, family: &str) -> HFONT {
    unsafe {
        CreateFontW(
            height,
            0,
            0,
            0,
            weight as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            CLEARTYPE_QUALITY as u32,
            DEFAULT_PITCH as u32,
            wide(family).as_ptr(),
        )
    }
}

fn create_static(
    hwnd: HWND,
    hinstance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> HWND {
    let control = unsafe {
        CreateWindowExW(
            0,
            wide("STATIC").as_ptr(),
            wide(text).as_ptr(),
            WS_CHILD | WS_VISIBLE | SS_LEFT_COMPAT,
            x,
            y,
            width,
            height,
            hwnd,
            null_mut(),
            hinstance,
            null(),
        )
    };
    let font = unsafe { GetStockObject(DEFAULT_GUI_FONT) };
    if !control.is_null() {
        unsafe {
            SendMessageW(control, WM_SETFONT, font as WPARAM, 1);
        }
    }
    control
}

fn create_icon_button(
    hwnd: HWND,
    hinstance: HINSTANCE,
    icon: &str,
    x: i32,
    y: i32,
    id: i32,
) -> HWND {
    unsafe {
        CreateWindowExW(
            0,
            wide("BUTTON").as_ptr(),
            wide(icon).as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW as u32,
            x,
            y,
            TITLE_BUTTON_SIZE,
            TITLE_BUTTON_SIZE,
            hwnd,
            id as HMENU,
            hinstance,
            null(),
        )
    }
}

unsafe fn create_tooltips(
    hwnd: HWND,
    hinstance: HINSTANCE,
    tools: &[(HWND, &str)],
) -> (HWND, Vec<Vec<u16>>) {
    let tooltip = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST,
            TOOLTIPS_CLASSW,
            null(),
            WS_POPUP | TTS_ALWAYSTIP | TTS_NOPREFIX,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            hwnd,
            null_mut(),
            hinstance,
            null(),
        )
    };
    if tooltip.is_null() {
        return (null_mut(), Vec::new());
    }

    unsafe {
        SendMessageW(tooltip, TTM_SETMAXTIPWIDTH, 0, 240);
    }

    let mut tooltip_text = tools
        .iter()
        .map(|(_, label)| wide(label))
        .collect::<Vec<_>>();

    for ((control, _), text) in tools.iter().zip(tooltip_text.iter_mut()) {
        if control.is_null() {
            continue;
        }

        let mut tool = TTTOOLINFOW {
            cbSize: size_of::<TTTOOLINFOW>() as u32,
            uFlags: TTF_IDISHWND | TTF_SUBCLASS,
            hwnd,
            uId: *control as usize,
            hinst: hinstance,
            lpszText: text.as_mut_ptr(),
            ..Default::default()
        };
        unsafe {
            SendMessageW(
                tooltip,
                TTM_ADDTOOLW,
                0,
                &mut tool as *mut TTTOOLINFOW as LPARAM,
            );
        }
    }

    (tooltip, tooltip_text)
}

unsafe fn update_checksum() {
    let (input_hwnd, output_hwnd, copy_hwnd) = UI.with(|ui| {
        let ui = ui.borrow();
        (ui.input, ui.output, ui.copy)
    });

    let input = unsafe { get_window_text(input_hwnd) };
    let hex = if input.is_empty() {
        String::new()
    } else {
        hash::hex(hash::adler32(input.as_bytes()))
    };

    UI.with(|ui| {
        ui.borrow_mut().hex = hex.clone();
    });

    let output = wide(&hex);
    let copy_label = wide(COPY_ICON);

    unsafe {
        SetWindowTextW(output_hwnd, output.as_ptr());
        SetWindowTextW(copy_hwnd, copy_label.as_ptr());
    }
}

unsafe fn get_window_text(hwnd: HWND) -> String {
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len <= 0 {
        return String::new();
    }

    let mut buffer = vec![0u16; len as usize + 1];
    let read = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..read as usize])
}

fn copy_to_clipboard(hwnd: HWND, text: &str) -> bool {
    unsafe {
        if OpenClipboard(hwnd) == 0 {
            return false;
        }

        EmptyClipboard();

        let wide_text = wide(text);
        let bytes = wide_text.len() * size_of::<u16>();
        let memory = GlobalAlloc(GMEM_MOVEABLE, bytes);
        if memory.is_null() {
            CloseClipboard();
            return false;
        }

        let lock = GlobalLock(memory) as *mut u16;
        if lock.is_null() {
            GlobalFree(memory);
            CloseClipboard();
            return false;
        }

        std::ptr::copy_nonoverlapping(wide_text.as_ptr(), lock, wide_text.len());
        GlobalUnlock(memory);

        if SetClipboardData(CF_UNICODETEXT as u32, memory).is_null() {
            GlobalFree(memory);
            CloseClipboard();
            return false;
        }

        CloseClipboard();
        true
    }
}

fn read_clipboard_text(hwnd: HWND) -> Option<String> {
    unsafe {
        if IsClipboardFormatAvailable(CF_UNICODETEXT as u32) == 0 {
            return None;
        }
        if OpenClipboard(hwnd) == 0 {
            return None;
        }

        let handle = GetClipboardData(CF_UNICODETEXT as u32);
        if handle.is_null() {
            CloseClipboard();
            return None;
        }

        let byte_len = GlobalSize(handle);
        if byte_len < size_of::<u16>() {
            CloseClipboard();
            return None;
        }

        let lock = GlobalLock(handle) as *const u16;
        if lock.is_null() {
            CloseClipboard();
            return None;
        }

        let max_units = byte_len / size_of::<u16>();
        let mut len = 0usize;
        while len < max_units && *lock.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(lock, len);
        let text = String::from_utf16_lossy(slice);

        GlobalUnlock(handle);
        CloseClipboard();
        Some(text)
    }
}

unsafe fn error_box(hwnd: HWND, message: &str) {
    unsafe {
        MessageBoxW(
            hwnd,
            wide(message).as_ptr(),
            wide("AdlerIT").as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn loword(value: WPARAM) -> i32 {
    (value & 0xffff) as i32
}

fn hiword(value: WPARAM) -> u32 {
    ((value >> 16) & 0xffff) as u32
}

fn loword_signed(value: LPARAM) -> i32 {
    (value as u16) as i16 as i32
}

fn hiword_signed(value: LPARAM) -> i32 {
    ((value >> 16) as u16) as i16 as i32
}

fn is_draggable_title_area(x: i32, y: i32) -> bool {
    (0..72).contains(&y) && x < 176
}

fn toggle_topmost(hwnd: HWND) {
    let (topmost_button, next_topmost) = UI.with(|ui| {
        let ui = ui.borrow();
        (ui.topmost, !ui.is_topmost)
    });

    let insert_after = if next_topmost {
        HWND_TOPMOST
    } else {
        HWND_NOTOPMOST
    };

    let updated = unsafe { SetWindowPos(hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE) };
    if updated == 0 {
        return;
    }

    UI.with(|ui| {
        ui.borrow_mut().is_topmost = next_topmost;
    });

    let icon = wide(if next_topmost { PINNED_ICON } else { PIN_ICON });
    unsafe {
        SetWindowTextW(topmost_button, icon.as_ptr());
    }
}

fn resource_id(id: usize) -> *const u16 {
    id as *const u16
}

fn edit_margins(left: u16, right: u16) -> LPARAM {
    i32::from(left) as LPARAM | ((i32::from(right) as LPARAM) << 16)
}

fn brush_or_system(brush: HBRUSH, system_color: i32) -> HBRUSH {
    if brush.is_null() {
        unsafe { GetSysColorBrush(system_color) }
    } else {
        brush
    }
}

fn last_error(context: &str) -> String {
    let code = unsafe { GetLastError() };
    format!("{context}: Windows error {code}")
}

const fn rgb(red: u8, green: u8, blue: u8) -> u32 {
    red as u32 | ((green as u32) << 8) | ((blue as u32) << 16)
}

fn is_dark_color(color: u32) -> bool {
    let red = color & 0xff;
    let green = (color >> 8) & 0xff;
    let blue = (color >> 16) & 0xff;
    (red * 30 + green * 59 + blue * 11) < 12_800
}

fn windows_apps_use_light_theme() -> Option<bool> {
    let mut value = 1u32;
    let mut value_size = size_of::<u32>() as u32;
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize").as_ptr(),
            wide("AppsUseLightTheme").as_ptr(),
            RRF_RT_REG_DWORD,
            null_mut(),
            &mut value as *mut u32 as *mut c_void,
            &mut value_size,
        )
    };

    if result == 0 { Some(value != 0) } else { None }
}

const ES_AUTOHSCROLL_COMPAT: u32 = 0x80;
const SS_LEFT_COMPAT: u32 = 0;
