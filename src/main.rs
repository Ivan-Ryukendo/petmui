#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("This prototype targets Windows only.");
}

#[cfg(target_os = "windows")]
mod windows_app {
    use std::ffi::c_void;
    use std::fs;
    use std::mem::{size_of, zeroed};
    use std::path::{Component, PathBuf};
    use std::ptr::{null, null_mut};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    type Bool = i32;
    type Dword = u32;
    type Uint = u32;
    type Wparam = usize;
    type Lparam = isize;
    type Lresult = isize;
    type Hinstance = *mut c_void;
    type Hwnd = *mut c_void;
    type Hicon = *mut c_void;
    type Hcursor = *mut c_void;
    type Hbrush = *mut c_void;
    type Hmenu = *mut c_void;
    type Hhook = *mut c_void;
    type Hdc = *mut c_void;
    type Hbitmap = *mut c_void;
    type Hgdiobj = *mut c_void;
    type Handle = *mut c_void;

    const WS_POPUP: Dword = 0x80000000;
    const WS_EX_LAYERED: Dword = 0x00080000;
    const WS_EX_TOPMOST: Dword = 0x00000008;
    const WS_EX_TOOLWINDOW: Dword = 0x00000080;
    const WS_EX_NOACTIVATE: Dword = 0x08000000;
    const WS_EX_TRANSPARENT: Dword = 0x00000020;

    const WM_DESTROY: Uint = 0x0002;
    const WM_TIMER: Uint = 0x0113;
    const WM_COMMAND: Uint = 0x0111;
    const WM_LBUTTONDOWN: Uint = 0x0201;
    const WM_NCLBUTTONDOWN: Uint = 0x00A1;
    const WM_RBUTTONUP: Uint = 0x0205;
    const WM_KEYDOWN: Uint = 0x0100;
    const WM_SYSKEYDOWN: Uint = 0x0104;
    const WM_APP: Uint = 0x8000;
    const WM_TRAY: Uint = WM_APP + 1;

    const HTCAPTION: Wparam = 2;
    const SW_HIDE: i32 = 0;
    const SW_SHOWNORMAL: i32 = 1;
    const SW_SHOWNOACTIVATE: i32 = 4;
    const SWP_NOSIZE: Uint = 0x0001;
    const SWP_NOMOVE: Uint = 0x0002;
    const SWP_NOACTIVATE: Uint = 0x0010;
    const HWND_TOPMOST: Hwnd = -1isize as Hwnd;
    const GWL_EXSTYLE: i32 = -20;

    const ULW_ALPHA: Dword = 0x00000002;
    const AC_SRC_OVER: u8 = 0x00;
    const AC_SRC_ALPHA: u8 = 0x01;
    const BI_RGB: Dword = 0;
    const DIB_RGB_COLORS: Uint = 0;

    const NIM_ADD: Dword = 0x00000000;
    const NIM_DELETE: Dword = 0x00000002;
    const NIM_MODIFY: Dword = 0x00000001;
    const NIF_MESSAGE: Dword = 0x00000001;
    const NIF_ICON: Dword = 0x00000002;
    const NIF_TIP: Dword = 0x00000004;
    const TPM_RIGHTBUTTON: Uint = 0x0002;
    const TPM_BOTTOMALIGN: Uint = 0x0020;
    const MF_STRING: Uint = 0x0000;
    const MF_SEPARATOR: Uint = 0x0800;
    const MF_CHECKED: Uint = 0x0008;

    const ID_TRAY_PAUSE: usize = 1001;
    const ID_TRAY_HIDE: usize = 1002;
    const ID_TRAY_RELOAD: usize = 1003;
    const ID_TRAY_SETTINGS: usize = 1004;
    const ID_TRAY_PETS_FOLDER: usize = 1005;
    const ID_TRAY_IMPORTS_FOLDER: usize = 1006;
    const ID_TRAY_EXIT: usize = 1007;
    const TIMER_RENDER: Wparam = 42;

    const WH_KEYBOARD_LL: i32 = 13;
    const HC_ACTION: i32 = 0;

    const PROCESS_QUERY_LIMITED_INFORMATION: Dword = 0x1000;
    const TH32CS_SNAPPROCESS: Dword = 0x00000002;
    const INVALID_HANDLE_VALUE: Handle = -1isize as Handle;
    const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
    const MAX_ATLAS_BYTES: usize = 64 * 1024 * 1024;
    const MAX_CELL_DIMENSION: i32 = 512;
    const MAX_COLUMNS: usize = 32;
    const MAX_ROWS: usize = 64;

    static KEY_EVENTS: AtomicU64 = AtomicU64::new(0);
    static mut APP: *mut App = null_mut();
    static mut KEYBOARD_HOOK: Hhook = null_mut();

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Size {
        cx: i32,
        cy: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    struct WndClassW {
        style: Uint,
        lpfn_wnd_proc: Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>,
        cb_cls_extra: i32,
        cb_wnd_extra: i32,
        h_instance: Hinstance,
        h_icon: Hicon,
        h_cursor: Hcursor,
        hbr_background: Hbrush,
        lpsz_menu_name: *const u16,
        lpsz_class_name: *const u16,
    }

    #[repr(C)]
    struct BlendFunction {
        blend_op: u8,
        blend_flags: u8,
        source_constant_alpha: u8,
        alpha_format: u8,
    }

    #[repr(C)]
    struct BitmapInfoHeader {
        bi_size: Dword,
        bi_width: i32,
        bi_height: i32,
        bi_planes: u16,
        bi_bit_count: u16,
        bi_compression: Dword,
        bi_size_image: Dword,
        bi_x_pels_per_meter: i32,
        bi_y_pels_per_meter: i32,
        bi_clr_used: Dword,
        bi_clr_important: Dword,
    }

    #[repr(C)]
    struct BitmapInfo {
        bmi_header: BitmapInfoHeader,
        bmi_colors: [Dword; 1],
    }

    #[repr(C)]
    struct NotifyIconDataW {
        cb_size: Dword,
        hwnd: Hwnd,
        u_id: Uint,
        u_flags: Uint,
        u_callback_message: Uint,
        h_icon: Hicon,
        sz_tip: [u16; 128],
        dw_state: Dword,
        dw_state_mask: Dword,
        sz_info: [u16; 256],
        u_timeout_or_version: Uint,
        sz_info_title: [u16; 64],
        dw_info_flags: Dword,
        guid_item: [u8; 16],
        h_balloon_icon: Hicon,
    }

    #[repr(C)]
    struct LastInputInfo {
        cb_size: Uint,
        dw_time: Dword,
    }

    #[repr(C)]
    struct ProcessEntry32W {
        dw_size: Dword,
        cnt_usage: Dword,
        th32_process_id: Dword,
        th32_default_heap_id: usize,
        th32_module_id: Dword,
        cnt_threads: Dword,
        th32_parent_process_id: Dword,
        pc_pri_class_base: i32,
        dw_flags: Dword,
        sz_exe_file: [u16; 260],
    }

    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassW(lp_wnd_class: *const WndClassW) -> u16;
        fn CreateWindowExW(
            dw_ex_style: Dword,
            lp_class_name: *const u16,
            lp_window_name: *const u16,
            dw_style: Dword,
            x: i32,
            y: i32,
            n_width: i32,
            n_height: i32,
            h_wnd_parent: Hwnd,
            h_menu: Hmenu,
            h_instance: Hinstance,
            lp_param: *mut c_void,
        ) -> Hwnd;
        fn DefWindowProcW(hwnd: Hwnd, msg: Uint, wparam: Wparam, lparam: Lparam) -> Lresult;
        fn DispatchMessageW(lp_msg: *const Msg) -> Lresult;
        fn GetMessageW(lp_msg: *mut Msg, hwnd: Hwnd, msg_filter_min: Uint, msg_filter_max: Uint) -> Bool;
        fn TranslateMessage(lp_msg: *const Msg) -> Bool;
        fn PostQuitMessage(exit_code: i32);
        fn ShowWindow(hwnd: Hwnd, n_cmd_show: i32) -> Bool;
        fn SetTimer(hwnd: Hwnd, n_id_event: Wparam, u_elapse: Uint, lp_timer_func: *const c_void) -> Wparam;
        fn KillTimer(hwnd: Hwnd, u_id_event: Wparam) -> Bool;
        fn UpdateLayeredWindow(
            hwnd: Hwnd,
            hdc_dst: Hdc,
            ppt_dst: *const Point,
            psize: *const Size,
            hdc_src: Hdc,
            ppt_src: *const Point,
            cr_key: Dword,
            pblend: *const BlendFunction,
            dw_flags: Dword,
        ) -> Bool;
        fn GetDC(hwnd: Hwnd) -> Hdc;
        fn ReleaseDC(hwnd: Hwnd, hdc: Hdc) -> i32;
        fn SetWindowPos(hwnd: Hwnd, hwnd_insert_after: Hwnd, x: i32, y: i32, cx: i32, cy: i32, flags: Uint) -> Bool;
        fn GetWindowLongPtrW(hwnd: Hwnd, n_index: i32) -> isize;
        fn SetWindowLongPtrW(hwnd: Hwnd, n_index: i32, dw_new_long: isize) -> isize;
        fn SendMessageW(hwnd: Hwnd, msg: Uint, wparam: Wparam, lparam: Lparam) -> Lresult;
        fn LoadIconW(h_instance: Hinstance, lp_icon_name: *const u16) -> Hicon;
        fn LoadCursorW(h_instance: Hinstance, lp_cursor_name: *const u16) -> Hcursor;
        fn SetForegroundWindow(hwnd: Hwnd) -> Bool;
        fn GetCursorPos(lp_point: *mut Point) -> Bool;
        fn CreatePopupMenu() -> Hmenu;
        fn AppendMenuW(hmenu: Hmenu, u_flags: Uint, u_id_new_item: usize, lp_new_item: *const u16) -> Bool;
        fn TrackPopupMenu(hmenu: Hmenu, u_flags: Uint, x: i32, y: i32, n_reserved: i32, hwnd: Hwnd, prc_rect: *const Rect) -> Bool;
        fn DestroyMenu(hmenu: Hmenu) -> Bool;
        fn GetLastInputInfo(plii: *mut LastInputInfo) -> Bool;
        fn GetTickCount() -> Dword;
        fn GetForegroundWindow() -> Hwnd;
        fn GetWindowThreadProcessId(hwnd: Hwnd, lpdw_process_id: *mut Dword) -> Dword;
        fn GetWindowTextW(hwnd: Hwnd, lp_string: *mut u16, n_max_count: i32) -> i32;
        fn SetWindowsHookExW(id_hook: i32, lpfn: Option<unsafe extern "system" fn(i32, Wparam, Lparam) -> Lresult>, hmod: Hinstance, dw_thread_id: Dword) -> Hhook;
        fn CallNextHookEx(hhk: Hhook, n_code: i32, wparam: Wparam, lparam: Lparam) -> Lresult;
        fn UnhookWindowsHookEx(hhk: Hhook) -> Bool;
    }

    #[link(name = "shell32")]
    extern "system" {
        fn Shell_NotifyIconW(dw_message: Dword, lp_data: *mut NotifyIconDataW) -> Bool;
        fn ShellExecuteW(hwnd: Hwnd, lp_operation: *const u16, lp_file: *const u16, lp_parameters: *const u16, lp_directory: *const u16, n_show_cmd: i32) -> isize;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
        fn DeleteDC(hdc: Hdc) -> Bool;
        fn CreateDIBSection(hdc: Hdc, pbmi: *const BitmapInfo, usage: Uint, ppv_bits: *mut *mut c_void, hsection: Handle, offset: Dword) -> Hbitmap;
        fn SelectObject(hdc: Hdc, h: Hgdiobj) -> Hgdiobj;
        fn DeleteObject(ho: Hgdiobj) -> Bool;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(lp_module_name: *const u16) -> Hinstance;
        fn OpenProcess(dw_desired_access: Dword, b_inherit_handle: Bool, dw_process_id: Dword) -> Handle;
        fn QueryFullProcessImageNameW(h_process: Handle, dw_flags: Dword, lp_exe_name: *mut u16, lpdw_size: *mut Dword) -> Bool;
        fn CloseHandle(h_object: Handle) -> Bool;
        fn CreateToolhelp32Snapshot(dw_flags: Dword, th32_process_id: Dword) -> Handle;
        fn Process32FirstW(h_snapshot: Handle, lppe: *mut ProcessEntry32W) -> Bool;
        fn Process32NextW(h_snapshot: Handle, lppe: *mut ProcessEntry32W) -> Bool;
    }

    #[repr(C)]
    struct Msg {
        hwnd: Hwnd,
        message: Uint,
        wparam: Wparam,
        lparam: Lparam,
        time: Dword,
        pt: Point,
    }

    #[derive(Clone)]
    struct Config {
        pet_size: i32,
        pet_directory: Option<PathBuf>,
        enable_typing_detection: bool,
        click_through_in_games: bool,
        typing_timeout: Duration,
        sleep_timeout: Duration,
        agent_processes: Vec<String>,
        game_processes: Vec<String>,
        music_processes: Vec<String>,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                pet_size: 96,
                pet_directory: None,
                enable_typing_detection: false,
                click_through_in_games: true,
                typing_timeout: Duration::from_secs(2),
                sleep_timeout: Duration::from_secs(300),
                agent_processes: vec![
                    "codex.exe".into(),
                    "claude.exe".into(),
                    "cursor.exe".into(),
                    "code.exe".into(),
                    "powershell.exe".into(),
                    "windowsterminal.exe".into(),
                ],
                game_processes: vec![
                    "steam.exe".into(),
                    "robloxplayerbeta.exe".into(),
                    "minecraft.windows.exe".into(),
                ],
                music_processes: vec!["pear-desktop.exe".into(), "spotify.exe".into()],
            }
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum PetState {
        Idle,
        Typing,
        AgentWorking,
        AgentSuccess,
        AgentFailed,
        MusicPlaying,
        MusicPaused,
        Gaming,
        Sleeping,
    }

    impl PetState {
        fn label(self) -> &'static str {
            match self {
                PetState::Idle => "Idle",
                PetState::Typing => "Typing",
                PetState::AgentWorking => "Agent working",
                PetState::AgentSuccess => "Agent success",
                PetState::AgentFailed => "Agent failed",
                PetState::MusicPlaying => "Music playing",
                PetState::MusicPaused => "Music paused",
                PetState::Gaming => "Gaming",
                PetState::Sleeping => "Sleeping",
            }
        }
    }

    struct PetPackage {
        cell_width: i32,
        cell_height: i32,
        columns: usize,
        rows: Vec<String>,
        pixels: Vec<u32>,
    }

    impl PetPackage {
        fn frame_pixels(&self, state: PetState, frame: u32, target_size: i32) -> Option<Vec<u32>> {
            if self.cell_width <= 0 || self.cell_height <= 0 || self.columns == 0 || self.rows.is_empty() {
                return None;
            }

            let row = self.row_for_state(state)?;
            let source_width = self.cell_width as usize * self.columns;
            let source_height = self.cell_height as usize * self.rows.len();
            if self.pixels.len() != source_width.checked_mul(source_height)? {
                return None;
            }

            let source_x = ((frame / 6) as usize % self.columns) * self.cell_width as usize;
            let source_y = row * self.cell_height as usize;
            let size = target_size.max(64).min(192) as usize;
            let mut out = vec![0u32; size * size];
            for y in 0..size {
                let sy = source_y + y * self.cell_height as usize / size;
                for x in 0..size {
                    let sx = source_x + x * self.cell_width as usize / size;
                    out[y * size + x] = self.pixels[sy * source_width + sx];
                }
            }
            Some(out)
        }

        fn row_for_state(&self, state: PetState) -> Option<usize> {
            let names: &[&str] = match state {
                PetState::Idle => &["idle"],
                PetState::Typing => &["typing", "running", "idle"],
                PetState::AgentWorking => &["review", "working", "running", "typing", "idle"],
                PetState::AgentSuccess => &["success", "waving", "idle"],
                PetState::AgentFailed => &["failed", "fail", "idle"],
                PetState::MusicPlaying => &["music-playing", "music", "waving", "idle"],
                PetState::MusicPaused => &["music-paused", "idle"],
                PetState::Gaming => &["jumping", "running-right", "running", "idle"],
                PetState::Sleeping => &["sleeping", "sleep", "idle"],
            };
            names.iter().find_map(|name| self.rows.iter().position(|row| row == name))
        }
    }

    struct App {
        hwnd: Hwnd,
        config: Config,
        pet_package: Option<PetPackage>,
        paused: bool,
        hidden: bool,
        click_through: bool,
        state: PetState,
        frame: u32,
        last_render: Instant,
        last_key_count: u64,
        last_keyboard_activity: Instant,
        flash_state: Option<(PetState, Instant)>,
    }

    pub fn run() {
        unsafe {
            let hinstance = GetModuleHandleW(null());
            let class_name = wide("LightweightDesktopPetWindow");
            let wc = WndClassW {
                style: 0,
                lpfn_wnd_proc: Some(wnd_proc),
                cb_cls_extra: 0,
                cb_wnd_extra: 0,
                h_instance: hinstance,
                h_icon: LoadIconW(null_mut(), 32512usize as *const u16),
                h_cursor: LoadCursorW(null_mut(), 32512usize as *const u16),
                hbr_background: null_mut(),
                lpsz_menu_name: null(),
                lpsz_class_name: class_name.as_ptr(),
            };

            if RegisterClassW(&wc) == 0 {
                return;
            }

            let config = load_config();
            let pet_package = load_pet_package(&config);
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name.as_ptr(),
                wide("Pocket Agent").as_ptr(),
                WS_POPUP,
                120,
                120,
                config.pet_size,
                config.pet_size,
                null_mut(),
                null_mut(),
                hinstance,
                null_mut(),
            );

            if hwnd.is_null() {
                return;
            }

            let mut app = Box::new(App {
                hwnd,
                config,
                pet_package,
                paused: false,
                hidden: false,
                click_through: false,
                state: PetState::Idle,
                frame: 0,
                last_render: Instant::now(),
                last_key_count: KEY_EVENTS.load(Ordering::Relaxed),
                last_keyboard_activity: Instant::now() - Duration::from_secs(60),
                flash_state: None,
            });

            APP = app.as_mut() as *mut App;
            app.sync_keyboard_hook();

            add_tray_icon(hwnd, "Pocket Agent");
            app.render();
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            SetTimer(hwnd, TIMER_RENDER, 100, null());

            let mut msg: Msg = zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            remove_tray_icon(hwnd);
            if !KEYBOARD_HOOK.is_null() {
                uninstall_keyboard_hook();
            }
            drop(app);
        }
    }

    impl App {
        fn tick(&mut self) {
            if self.hidden {
                return;
            }

            self.update_keyboard_activity();
            if !self.paused {
                self.state = self.resolve_state();
            }
            self.sync_overlay_mode();
            self.sync_keyboard_hook();
            self.frame = self.frame.wrapping_add(1);
            self.last_render = Instant::now();
            self.render();
            unsafe {
                update_tray_tip(self.hwnd, &format!("Pocket Agent - {}", self.state.label()));
            }
        }

        fn update_keyboard_activity(&mut self) {
            let key_count = KEY_EVENTS.load(Ordering::Relaxed);
            if key_count != self.last_key_count {
                self.last_keyboard_activity = Instant::now();
                self.last_key_count = key_count;
            }
        }

        fn resolve_state(&mut self) -> PetState {
            if let Some((state, until)) = self.flash_state {
                if Instant::now() < until {
                    return state;
                }
                self.flash_state = None;
            }

            let foreground = foreground_process_name();
            let active_for = last_input_age();

            if let Some(exe) = foreground.as_ref() {
                if contains_name(&self.config.game_processes, exe) {
                    return PetState::Gaming;
                }
            }

            if let Some(exe) = foreground.as_ref() {
                if contains_name(&self.config.agent_processes, exe) {
                    let title = foreground_window_title().to_ascii_lowercase();
                    if title.contains("failed") || title.contains("error") || title.contains("panic") {
                        self.flash_state = Some((PetState::AgentFailed, Instant::now() + Duration::from_secs(4)));
                        return PetState::AgentFailed;
                    }
                    if title.contains("success") || title.contains("complete") || title.contains("finished") {
                        self.flash_state = Some((PetState::AgentSuccess, Instant::now() + Duration::from_secs(4)));
                        return PetState::AgentSuccess;
                    }
                    return PetState::AgentWorking;
                }
            }

            if self.config.enable_typing_detection && Instant::now().duration_since(self.last_keyboard_activity) <= self.config.typing_timeout {
                return PetState::Typing;
            }

            if process_exists_any(&self.config.music_processes) {
                let foreground_is_music = foreground
                    .as_ref()
                    .is_some_and(|exe| contains_name(&self.config.music_processes, exe));
                if foreground_is_music && foreground_window_title().to_ascii_lowercase().contains("pause") {
                    return PetState::MusicPaused;
                }
                return PetState::MusicPlaying;
            }

            if active_for >= self.config.sleep_timeout {
                return PetState::Sleeping;
            }

            PetState::Idle
        }

        fn render(&self) {
            let size = self.config.pet_size.max(64).min(192);
            let pixels = self
                .pet_package
                .as_ref()
                .and_then(|package| package.frame_pixels(self.state, self.frame, size))
                .unwrap_or_else(|| {
                    let mut pixels = vec![0u32; (size * size) as usize];
                    draw_pet(&mut pixels, size, self.state, self.frame);
                    pixels
                });
            unsafe {
                update_layered_pixels(self.hwnd, size, size, &pixels);
            }
        }

        fn toggle_pause(&mut self) {
            self.paused = !self.paused;
            if self.paused {
                self.state = PetState::Idle;
            }
            self.sync_keyboard_hook();
            self.render();
        }

        fn toggle_hidden(&mut self) {
            self.hidden = !self.hidden;
            self.sync_keyboard_hook();
            unsafe {
                ShowWindow(self.hwnd, if self.hidden { SW_HIDE } else { SW_SHOWNOACTIVATE });
            }
        }

        fn reload_config_and_pet(&mut self) {
            self.config = load_config();
            self.pet_package = load_pet_package(&self.config);
            self.sync_keyboard_hook();
            self.sync_overlay_mode();
            self.render();
        }

        fn sync_keyboard_hook(&self) {
            unsafe {
                if self.config.enable_typing_detection && !self.paused && !self.hidden {
                    install_keyboard_hook();
                } else {
                    uninstall_keyboard_hook();
                }
            }
        }

        fn sync_overlay_mode(&mut self) {
            let should_click_through = self.config.click_through_in_games && self.state == PetState::Gaming;
            if should_click_through == self.click_through {
                return;
            }
            self.click_through = should_click_through;
            unsafe {
                set_click_through(self.hwnd, should_click_through);
            }
        }
    }

    unsafe extern "system" fn wnd_proc(hwnd: Hwnd, msg: Uint, wparam: Wparam, lparam: Lparam) -> Lresult {
        match msg {
            WM_TIMER => {
                if wparam == TIMER_RENDER && !APP.is_null() {
                    (*APP).tick();
                }
                0
            }
            WM_LBUTTONDOWN => {
                if APP.is_null() || !(*APP).click_through {
                    SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION, 0);
                }
                0
            }
            WM_RBUTTONUP => {
                show_tray_menu(hwnd);
                0
            }
            WM_TRAY => {
                let mouse_msg = lparam as Uint;
                if mouse_msg == WM_RBUTTONUP || mouse_msg == WM_LBUTTONDOWN {
                    show_tray_menu(hwnd);
                }
                0
            }
            WM_COMMAND => {
                let id = wparam & 0xffff;
                if !APP.is_null() {
                    match id {
                        ID_TRAY_PAUSE => (*APP).toggle_pause(),
                        ID_TRAY_HIDE => (*APP).toggle_hidden(),
                        ID_TRAY_RELOAD => (*APP).reload_config_and_pet(),
                        ID_TRAY_SETTINGS => open_settings_folder(hwnd),
                        ID_TRAY_PETS_FOLDER => open_pets_folder(hwnd),
                        ID_TRAY_IMPORTS_FOLDER => open_imports_folder(hwnd),
                        ID_TRAY_EXIT => {
                            KillTimer(hwnd, TIMER_RENDER);
                            PostQuitMessage(0);
                        }
                        _ => {}
                    }
                }
                0
            }
            WM_DESTROY => {
                KillTimer(hwnd, TIMER_RENDER);
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    unsafe extern "system" fn keyboard_proc(code: i32, wparam: Wparam, lparam: Lparam) -> Lresult {
        if code == HC_ACTION && (wparam as Uint == WM_KEYDOWN || wparam as Uint == WM_SYSKEYDOWN) {
            KEY_EVENTS.fetch_add(1, Ordering::Relaxed);
        }
        CallNextHookEx(null_mut(), code, wparam, lparam)
    }

    unsafe fn install_keyboard_hook() {
        if KEYBOARD_HOOK.is_null() {
            KEYBOARD_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), null_mut(), 0);
        }
    }

    unsafe fn uninstall_keyboard_hook() {
        if !KEYBOARD_HOOK.is_null() {
            UnhookWindowsHookEx(KEYBOARD_HOOK);
            KEYBOARD_HOOK = null_mut();
        }
    }

    unsafe fn set_click_through(hwnd: Hwnd, enabled: bool) {
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as Dword;
        let next = if enabled {
            current | WS_EX_TRANSPARENT
        } else {
            current & !WS_EX_TRANSPARENT
        };
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next as isize);
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }

    unsafe fn update_layered_pixels(hwnd: Hwnd, width: i32, height: i32, pixels: &[u32]) {
        let screen_dc = GetDC(null_mut());
        if screen_dc.is_null() {
            return;
        }
        let mem_dc = CreateCompatibleDC(screen_dc);
        if mem_dc.is_null() {
            ReleaseDC(null_mut(), screen_dc);
            return;
        }

        let mut bits: *mut c_void = null_mut();
        let bmi = BitmapInfo {
            bmi_header: BitmapInfoHeader {
                bi_size: size_of::<BitmapInfoHeader>() as Dword,
                bi_width: width,
                bi_height: -height,
                bi_planes: 1,
                bi_bit_count: 32,
                bi_compression: BI_RGB,
                bi_size_image: 0,
                bi_x_pels_per_meter: 0,
                bi_y_pels_per_meter: 0,
                bi_clr_used: 0,
                bi_clr_important: 0,
            },
            bmi_colors: [0],
        };

        let bitmap = CreateDIBSection(screen_dc, &bmi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        if bitmap.is_null() || bits.is_null() {
            DeleteDC(mem_dc);
            ReleaseDC(null_mut(), screen_dc);
            return;
        }

        std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits as *mut u32, pixels.len());
        let old = SelectObject(mem_dc, bitmap as Hgdiobj);
        let size = Size { cx: width, cy: height };
        let src = Point { x: 0, y: 0 };
        let blend = BlendFunction {
            blend_op: AC_SRC_OVER,
            blend_flags: 0,
            source_constant_alpha: 255,
            alpha_format: AC_SRC_ALPHA,
        };

        UpdateLayeredWindow(hwnd, screen_dc, null(), &size, mem_dc, &src, 0, &blend, ULW_ALPHA);
        SelectObject(mem_dc, old);
        DeleteObject(bitmap as Hgdiobj);
        DeleteDC(mem_dc);
        ReleaseDC(null_mut(), screen_dc);
    }

    fn draw_pet(pixels: &mut [u32], size: i32, state: PetState, frame: u32) {
        let bob = match state {
            PetState::Sleeping => 0,
            PetState::Typing | PetState::Gaming | PetState::AgentWorking => ((frame / 2) % 3) as i32 - 1,
            _ => ((frame / 6) % 3) as i32 - 1,
        };
        let (body, accent, face) = palette(state);
        let cx = size / 2;
        let cy = size / 2 + bob;
        let scale = size as f32 / 96.0;

        ellipse(pixels, size, cx, cy + (18.0 * scale) as i32, (30.0 * scale) as i32, (22.0 * scale) as i32, body);
        ellipse(pixels, size, cx, cy - (4.0 * scale) as i32, (25.0 * scale) as i32, (24.0 * scale) as i32, body);
        ellipse(pixels, size, cx - (17.0 * scale) as i32, cy - (26.0 * scale) as i32, (10.0 * scale) as i32, (13.0 * scale) as i32, body);
        ellipse(pixels, size, cx + (17.0 * scale) as i32, cy - (26.0 * scale) as i32, (10.0 * scale) as i32, (13.0 * scale) as i32, body);
        ellipse(pixels, size, cx - (13.0 * scale) as i32, cy - (8.0 * scale) as i32, (4.0 * scale) as i32, (5.0 * scale) as i32, face);
        ellipse(pixels, size, cx + (13.0 * scale) as i32, cy - (8.0 * scale) as i32, (4.0 * scale) as i32, (5.0 * scale) as i32, face);

        match state {
            PetState::Sleeping => {
                line(pixels, size, cx - 14, cy - 7, cx - 7, cy - 5, face);
                line(pixels, size, cx + 7, cy - 5, cx + 14, cy - 7, face);
                z_marks(pixels, size, cx + 26, cy - 36, accent);
            }
            PetState::Typing => {
                rect(pixels, size, cx - 24, cy + 28, cx + 24, cy + 40, accent);
                for i in 0..3 {
                    let x = cx - 12 + i * 12;
                    rect(pixels, size, x, cy + 31, x + 5, cy + 34, face);
                }
            }
            PetState::Gaming => {
                rect(pixels, size, cx - 26, cy + 23, cx + 26, cy + 38, accent);
                ellipse(pixels, size, cx - 16, cy + 30, 4, 4, face);
                ellipse(pixels, size, cx + 16, cy + 30, 4, 4, face);
            }
            PetState::MusicPlaying => {
                note(pixels, size, cx + 24, cy - 31, accent);
                note(pixels, size, cx - 34, cy - 20, accent);
            }
            PetState::MusicPaused => {
                rect(pixels, size, cx - 6, cy - 37, cx - 2, cy - 24, accent);
                rect(pixels, size, cx + 2, cy - 37, cx + 6, cy - 24, accent);
            }
            PetState::AgentWorking => {
                rect(pixels, size, cx - 22, cy + 26, cx + 22, cy + 38, accent);
                line(pixels, size, cx - 8, cy + 31, cx - 2, cy + 35, face);
                line(pixels, size, cx + 8, cy + 31, cx + 2, cy + 35, face);
            }
            PetState::AgentSuccess => {
                line(pixels, size, cx - 14, cy + 7, cx - 4, cy + 17, accent);
                line(pixels, size, cx - 4, cy + 17, cx + 18, cy - 10, accent);
            }
            PetState::AgentFailed => {
                line(pixels, size, cx - 18, cy - 15, cx - 8, cy - 5, accent);
                line(pixels, size, cx - 8, cy - 15, cx - 18, cy - 5, accent);
                line(pixels, size, cx + 8, cy - 15, cx + 18, cy - 5, accent);
                line(pixels, size, cx + 18, cy - 15, cx + 8, cy - 5, accent);
            }
            PetState::Idle => {
                line(pixels, size, cx - 8, cy + 9, cx, cy + 12, face);
                line(pixels, size, cx, cy + 12, cx + 8, cy + 9, face);
            }
        }
    }

    fn palette(state: PetState) -> (u32, u32, u32) {
        match state {
            PetState::Gaming => (argb(255, 91, 180, 116), argb(255, 38, 89, 55), argb(255, 21, 35, 29)),
            PetState::Typing => (argb(255, 88, 166, 224), argb(255, 245, 197, 66), argb(255, 20, 32, 44)),
            PetState::AgentWorking => (argb(255, 126, 132, 232), argb(255, 255, 193, 92), argb(255, 23, 24, 40)),
            PetState::AgentSuccess => (argb(255, 78, 199, 139), argb(255, 21, 116, 78), argb(255, 14, 49, 39)),
            PetState::AgentFailed => (argb(255, 228, 96, 103), argb(255, 130, 31, 48), argb(255, 42, 20, 24)),
            PetState::MusicPlaying => (argb(255, 241, 135, 91), argb(255, 61, 129, 104), argb(255, 39, 27, 24)),
            PetState::MusicPaused => (argb(255, 160, 169, 181), argb(255, 87, 93, 105), argb(255, 28, 33, 39)),
            PetState::Sleeping => (argb(255, 117, 129, 151), argb(255, 255, 221, 118), argb(255, 25, 32, 44)),
            PetState::Idle => (argb(255, 98, 202, 190), argb(255, 255, 214, 102), argb(255, 20, 39, 40)),
        }
    }

    fn argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    }

    fn ellipse(p: &mut [u32], size: i32, cx: i32, cy: i32, rx: i32, ry: i32, color: u32) {
        for y in (cy - ry)..=(cy + ry) {
            for x in (cx - rx)..=(cx + rx) {
                let dx = (x - cx) as f32 / rx.max(1) as f32;
                let dy = (y - cy) as f32 / ry.max(1) as f32;
                if dx * dx + dy * dy <= 1.0 {
                    put(p, size, x, y, color);
                }
            }
        }
    }

    fn rect(p: &mut [u32], size: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
        for y in y1..=y2 {
            for x in x1..=x2 {
                put(p, size, x, y, color);
            }
        }
    }

    fn line(p: &mut [u32], size: i32, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            for oy in -1..=1 {
                put(p, size, x0, y0 + oy, color);
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn note(p: &mut [u32], size: i32, x: i32, y: i32, color: u32) {
        rect(p, size, x, y, x + 3, y + 18, color);
        rect(p, size, x + 3, y, x + 13, y + 3, color);
        ellipse(p, size, x - 2, y + 19, 6, 4, color);
    }

    fn z_marks(p: &mut [u32], size: i32, x: i32, y: i32, color: u32) {
        line(p, size, x, y, x + 11, y, color);
        line(p, size, x + 11, y, x, y + 10, color);
        line(p, size, x, y + 10, x + 11, y + 10, color);
    }

    fn put(p: &mut [u32], size: i32, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && x < size && y < size {
            p[(y * size + x) as usize] = color;
        }
    }

    unsafe fn add_tray_icon(hwnd: Hwnd, tip: &str) {
        let mut nid = tray_data(hwnd, tip);
        Shell_NotifyIconW(NIM_ADD, &mut nid);
    }

    unsafe fn update_tray_tip(hwnd: Hwnd, tip: &str) {
        let mut nid = tray_data(hwnd, tip);
        nid.u_flags = NIF_TIP;
        Shell_NotifyIconW(NIM_MODIFY, &mut nid);
    }

    unsafe fn remove_tray_icon(hwnd: Hwnd) {
        let mut nid = tray_data(hwnd, "");
        Shell_NotifyIconW(NIM_DELETE, &mut nid);
    }

    unsafe fn tray_data(hwnd: Hwnd, tip: &str) -> NotifyIconDataW {
        let mut nid: NotifyIconDataW = zeroed();
        nid.cb_size = size_of::<NotifyIconDataW>() as Dword;
        nid.hwnd = hwnd;
        nid.u_id = 1;
        nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.u_callback_message = WM_TRAY;
        nid.h_icon = LoadIconW(null_mut(), 32512usize as *const u16);
        copy_wide(&mut nid.sz_tip, tip);
        nid
    }

    unsafe fn show_tray_menu(hwnd: Hwnd) {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return;
        }
        let paused = !APP.is_null() && (*APP).paused;
        let hidden = !APP.is_null() && (*APP).hidden;
        AppendMenuW(menu, MF_STRING | if paused { MF_CHECKED } else { 0 }, ID_TRAY_PAUSE, wide("Pause").as_ptr());
        AppendMenuW(menu, MF_STRING, ID_TRAY_HIDE, wide(if hidden { "Show" } else { "Hide" }).as_ptr());
        AppendMenuW(menu, MF_STRING, ID_TRAY_RELOAD, wide("Reload Pet").as_ptr());
        AppendMenuW(menu, MF_STRING, ID_TRAY_SETTINGS, wide("Open Settings Folder").as_ptr());
        AppendMenuW(menu, MF_STRING, ID_TRAY_PETS_FOLDER, wide("Open Pets Folder").as_ptr());
        AppendMenuW(menu, MF_STRING, ID_TRAY_IMPORTS_FOLDER, wide("Open Import Folder").as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(menu, MF_STRING, ID_TRAY_EXIT, wide("Exit").as_ptr());

        let mut pt = Point { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        SetForegroundWindow(hwnd);
        TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_BOTTOMALIGN, pt.x, pt.y, 0, hwnd, null());
        DestroyMenu(menu);
    }

    unsafe fn open_settings_folder(hwnd: Hwnd) {
        open_folder_in_explorer(hwnd, app_base_dir(), false);
    }

    unsafe fn open_pets_folder(hwnd: Hwnd) {
        let folder = pets_dir();
        let _ = fs::create_dir_all(imports_dir());
        write_folder_readme(&folder, "Put converted pet folders here. Use tools\\convert_hatch_pet.py to convert Codex pets, spritesheets, images, or emoji into petmui packages.");
        open_folder_in_explorer(hwnd, folder, true);
    }

    unsafe fn open_imports_folder(hwnd: Hwnd) {
        let folder = imports_dir();
        write_folder_readme(&folder, "Drop custom PNG, WebP, JPEG, BMP, or GIF files here. Convert one with tools\\convert_hatch_pet.py <image> ..\\your-pet --static --write-config ..\\..\\config.toml, then Reload Pet.");
        open_folder_in_explorer(hwnd, folder, true);
    }

    unsafe fn open_folder_in_explorer(hwnd: Hwnd, folder: PathBuf, create: bool) {
        if create && fs::create_dir_all(&folder).is_err() {
            return;
        }
        let Ok(folder) = folder.canonicalize() else {
            return;
        };
        if !folder.is_dir() {
            return;
        }
        let operation = wide("open");
        let explorer = wide("explorer.exe");
        let params = wide(&folder.to_string_lossy());
        let cwd = app_base_dir();
        let cwd = wide(&cwd.to_string_lossy());
        ShellExecuteW(
            hwnd,
            operation.as_ptr(),
            explorer.as_ptr(),
            params.as_ptr(),
            cwd.as_ptr(),
            SW_SHOWNORMAL,
        );
    }

    fn load_config() -> Config {
        let mut config = Config::default();
        let mut candidates = Vec::new();
        candidates.push(app_base_dir().join("config.toml"));

        let loaded = candidates
            .into_iter()
            .find_map(|path| fs::read_to_string(&path).ok().map(|text| (path, text)));

        if let Some((path, text)) = loaded {
            let base_dir = path.parent().map(PathBuf::from);
            for line in text.lines() {
                let clean = line.split('#').next().unwrap_or("").trim();
                if clean.is_empty() || clean.starts_with('[') {
                    continue;
                }
                if let Some((key, value)) = clean.split_once('=') {
                    apply_config_value(&mut config, key.trim(), value.trim(), &text, base_dir.as_ref());
                }
            }
        }

        config.agent_processes = lower_list(config.agent_processes);
        config.game_processes = lower_list(config.game_processes);
        config.music_processes = lower_list(config.music_processes);
        config
    }

    fn app_base_dir() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn pets_dir() -> PathBuf {
        app_base_dir().join("pets")
    }

    fn imports_dir() -> PathBuf {
        pets_dir().join("imports")
    }

    fn write_folder_readme(folder: &PathBuf, text: &str) {
        if fs::create_dir_all(folder).is_err() {
            return;
        }
        let path = folder.join("README.txt");
        if !path.exists() {
            let _ = fs::write(path, text);
        }
    }

    fn apply_config_value(config: &mut Config, key: &str, value: &str, full_text: &str, base_dir: Option<&PathBuf>) {
        match key {
            "pet_size" => {
                if let Ok(n) = value.parse::<i32>() {
                    config.pet_size = n.max(64).min(192);
                }
            }
            "pet_directory" => {
                if let Some(path) = parse_string(value) {
                    let path = PathBuf::from(path);
                    config.pet_directory = Some(if path.is_absolute() {
                        path
                    } else if let Some(base) = base_dir {
                        base.join(path)
                    } else {
                        path
                    });
                }
            }
            "enable_typing_detection" => {
                if let Some(value) = parse_bool(value) {
                    config.enable_typing_detection = value;
                }
            }
            "click_through_in_games" => {
                if let Some(value) = parse_bool(value) {
                    config.click_through_in_games = value;
                }
            }
            "typing_timeout_seconds" => {
                if let Ok(n) = value.parse::<u64>() {
                    config.typing_timeout = Duration::from_secs(n);
                }
            }
            "sleep_timeout_seconds" => {
                if let Ok(n) = value.parse::<u64>() {
                    config.sleep_timeout = Duration::from_secs(n);
                }
            }
            "agent_processes" => config.agent_processes = parse_array_for_key(full_text, key),
            "game_processes" => config.game_processes = parse_array_for_key(full_text, key),
            "music_processes" => config.music_processes = parse_array_for_key(full_text, key),
            _ => {}
        }
    }

    fn parse_bool(value: &str) -> Option<bool> {
        match value.trim() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }

    fn load_pet_package(config: &Config) -> Option<PetPackage> {
        let directory = config.pet_directory.as_ref()?;
        let directory = directory.canonicalize().ok()?;
        if !directory.is_dir() {
            return None;
        }
        let manifest_path = directory.join("pet.json");
        if fs::metadata(&manifest_path).ok()?.len() > MAX_MANIFEST_BYTES {
            return None;
        }
        let manifest = fs::read_to_string(manifest_path).ok()?;
        let renderer = json_string(&manifest, "renderer")?;
        if renderer == "static-bgra-v1" {
            return load_static_pet_package(&directory, &manifest);
        }
        if renderer != "raw-bgra-atlas-v1" {
            return None;
        }

        let atlas = json_string(&manifest, "atlas").unwrap_or_else(|| "atlas.bgra".into());
        let cell_width = json_i32(&manifest, "cellWidth")?;
        let cell_height = json_i32(&manifest, "cellHeight")?;
        let columns = json_usize(&manifest, "columns")?;
        let rows = json_string_array(&manifest, "rows");
        if cell_width <= 0
            || cell_width > MAX_CELL_DIMENSION
            || cell_height <= 0
            || cell_height > MAX_CELL_DIMENSION
            || columns == 0
            || columns > MAX_COLUMNS
            || rows.is_empty()
            || rows.len() > MAX_ROWS
        {
            return None;
        }

        let width = cell_width as usize * columns;
        let height = cell_height as usize * rows.len();
        let expected = width.checked_mul(height)?.checked_mul(4)?;
        if expected > MAX_ATLAS_BYTES {
            return None;
        }
        let atlas_path = contained_package_file(&directory, &atlas)?;
        if fs::metadata(&atlas_path).ok()?.len() != expected as u64 {
            return None;
        }
        let bytes = fs::read(atlas_path).ok()?;
        if bytes.len() != expected {
            return None;
        }

        let pixels = bytes
            .chunks_exact(4)
            .map(|px| u32::from_le_bytes([px[0], px[1], px[2], px[3]]))
            .collect();

        Some(PetPackage {
            cell_width,
            cell_height,
            columns,
            rows: rows.into_iter().map(|row| row.to_ascii_lowercase()).collect(),
            pixels,
        })
    }

    fn load_static_pet_package(directory: &PathBuf, manifest: &str) -> Option<PetPackage> {
        let image = json_string(manifest, "image").unwrap_or_else(|| "image.bgra".into());
        let width = json_i32(manifest, "width")?;
        let height = json_i32(manifest, "height")?;
        if width <= 0 || width > MAX_CELL_DIMENSION || height <= 0 || height > MAX_CELL_DIMENSION {
            return None;
        }

        let expected = (width as usize).checked_mul(height as usize)?.checked_mul(4)?;
        if expected > MAX_ATLAS_BYTES {
            return None;
        }
        let image_path = contained_package_file(directory, &image)?;
        if fs::metadata(&image_path).ok()?.len() != expected as u64 {
            return None;
        }
        let bytes = fs::read(image_path).ok()?;
        if bytes.len() != expected {
            return None;
        }

        let pixels = bytes
            .chunks_exact(4)
            .map(|px| u32::from_le_bytes([px[0], px[1], px[2], px[3]]))
            .collect();

        Some(PetPackage {
            cell_width: width,
            cell_height: height,
            columns: 1,
            rows: vec!["idle".into()],
            pixels,
        })
    }

    fn contained_package_file(directory: &PathBuf, name: &str) -> Option<PathBuf> {
        let relative = PathBuf::from(name);
        if relative.is_absolute()
            || relative.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return None;
        }
        let candidate = directory.join(relative).canonicalize().ok()?;
        if candidate.starts_with(directory) && candidate.is_file() {
            Some(candidate)
        } else {
            None
        }
    }

    fn parse_string(value: &str) -> Option<String> {
        let value = value.trim();
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            Some(value[1..value.len() - 1].replace("\\\"", "\""))
        } else {
            None
        }
    }

    fn json_string(text: &str, key: &str) -> Option<String> {
        let value = json_value_after_key(text, key)?;
        parse_json_string(value)
    }

    fn json_i32(text: &str, key: &str) -> Option<i32> {
        let value = json_value_after_key(text, key)?;
        let end = value
            .find(|ch: char| !ch.is_ascii_digit() && ch != '-')
            .unwrap_or(value.len());
        value[..end].trim().parse().ok()
    }

    fn json_usize(text: &str, key: &str) -> Option<usize> {
        json_i32(text, key).and_then(|n| usize::try_from(n).ok())
    }

    fn json_string_array(text: &str, key: &str) -> Vec<String> {
        let Some(value) = json_value_after_key(text, key) else {
            return Vec::new();
        };
        let Some(open) = value.find('[') else {
            return Vec::new();
        };
        let Some(close) = value[open + 1..].find(']') else {
            return Vec::new();
        };
        value[open + 1..open + 1 + close]
            .split(',')
            .filter_map(parse_json_string)
            .collect()
    }

    fn json_value_after_key<'a>(text: &'a str, key: &str) -> Option<&'a str> {
        let needle = format!("\"{key}\"");
        let after_key = text.find(&needle).map(|index| &text[index + needle.len()..])?;
        let colon = after_key.find(':')?;
        Some(after_key[colon + 1..].trim_start())
    }

    fn parse_json_string(value: &str) -> Option<String> {
        let value = value.trim_start();
        if !value.starts_with('"') {
            return None;
        }
        let mut escaped = false;
        let mut out = String::new();
        for ch in value[1..].chars() {
            if escaped {
                out.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                return Some(out);
            } else {
                out.push(ch);
            }
        }
        None
    }

    fn parse_array_for_key(text: &str, key: &str) -> Vec<String> {
        let needle = format!("{key} = [");
        let Some(start) = text.find(&needle) else {
            return Vec::new();
        };
        let rest = &text[start + needle.len()..];
        let Some(end) = rest.find(']') else {
            return Vec::new();
        };
        rest[..end]
            .split(',')
            .filter_map(|part| {
                let item = part.trim().trim_matches('"').trim();
                if item.is_empty() {
                    None
                } else {
                    Some(item.to_ascii_lowercase())
                }
            })
            .collect()
    }

    fn lower_list(values: Vec<String>) -> Vec<String> {
        values.into_iter().map(|v| v.to_ascii_lowercase()).collect()
    }

    fn contains_name(list: &[String], name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        list.iter().any(|item| item == &lower)
    }

    fn last_input_age() -> Duration {
        unsafe {
            let mut info = LastInputInfo {
                cb_size: size_of::<LastInputInfo>() as Uint,
                dw_time: 0,
            };
            if GetLastInputInfo(&mut info) == 0 {
                return Duration::ZERO;
            }
            let now = GetTickCount();
            Duration::from_millis(now.wrapping_sub(info.dw_time) as u64)
        }
    }

    fn foreground_process_name() -> Option<String> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            process_name_from_pid(pid)
        }
    }

    fn foreground_window_title() -> String {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return String::new();
            }
            let mut buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            String::from_utf16_lossy(&buf[..len.max(0) as usize])
        }
    }

    unsafe fn process_name_from_pid(pid: Dword) -> Option<String> {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as Dword;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
        CloseHandle(handle);
        if ok == 0 || size == 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..size as usize]);
        path.rsplit(['\\', '/']).next().map(|name| name.to_ascii_lowercase())
    }

    fn process_exists_any(names: &[String]) -> bool {
        if names.is_empty() {
            return false;
        }
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return false;
            }
            let mut entry: ProcessEntry32W = zeroed();
            entry.dw_size = size_of::<ProcessEntry32W>() as Dword;
            let mut ok = Process32FirstW(snapshot, &mut entry);
            while ok != 0 {
                let name = wide_buf_to_string(&entry.sz_exe_file).to_ascii_lowercase();
                if names.iter().any(|needle| needle == &name) {
                    CloseHandle(snapshot);
                    return true;
                }
                ok = Process32NextW(snapshot, &mut entry);
            }
            CloseHandle(snapshot);
        }
        false
    }

    fn wide_buf_to_string(buf: &[u16]) -> String {
        let len = buf.iter().position(|ch| *ch == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..len])
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn copy_wide<const N: usize>(target: &mut [u16; N], s: &str) {
        let w = wide(s);
        let len = (N - 1).min(w.len().saturating_sub(1));
        target[..len].copy_from_slice(&w[..len]);
        target[len] = 0;
    }
}

#[cfg(target_os = "windows")]
fn main() {
    windows_app::run();
}
