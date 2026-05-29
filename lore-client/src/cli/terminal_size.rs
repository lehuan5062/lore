// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
#[cfg(target_family = "unix")]
pub fn terminal_size() -> Option<(u16, u16)> {
    use libc::STDOUT_FILENO;
    use libc::TIOCGWINSZ;
    use libc::ioctl;
    use libc::winsize;

    let mut win_size = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    // Safety: Safe as long as we're using valid parameters and a valid winsize struct.
    let result = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut win_size) };
    (result == 0).then_some((win_size.ws_col, win_size.ws_row))
}

#[cfg(target_family = "windows")]
pub fn terminal_size() -> Option<(u16, u16)> {
    use std::os::windows::io::AsHandle;
    use std::os::windows::io::AsRawHandle;
    use std::os::windows::io::BorrowedHandle;
    use std::os::windows::io::RawHandle;

    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::System::Console::CONSOLE_SCREEN_BUFFER_INFO;
    use windows_sys::Win32::System::Console::COORD;
    use windows_sys::Win32::System::Console::GetConsoleScreenBufferInfo;
    use windows_sys::Win32::System::Console::GetStdHandle;
    use windows_sys::Win32::System::Console::SMALL_RECT;
    use windows_sys::Win32::System::Console::STD_OUTPUT_HANDLE;

    // Safety:
    // GetStdHandle is safe when used with a valid handle (e.g. STD_OUTPUT_HANDLE)
    // borrow_raw is safe as long as the handle is a valid handle, which GetStdHandle(STD_OUTPUT_HANDLE) is.
    let handle =
        unsafe { BorrowedHandle::borrow_raw(GetStdHandle(STD_OUTPUT_HANDLE) as RawHandle) };

    let raw_handle = handle.as_handle().as_raw_handle() as HANDLE;

    if raw_handle == INVALID_HANDLE_VALUE {
        return None;
    }

    let coord = COORD { X: 0, Y: 0 };
    let mut screen_buffer_info = CONSOLE_SCREEN_BUFFER_INFO {
        dwSize: coord,
        dwCursorPosition: coord,
        wAttributes: 0,
        srWindow: SMALL_RECT {
            Left: 0,
            Top: 0,
            Right: 0,
            Bottom: 0,
        },
        dwMaximumWindowSize: coord,
    };
    // SAFETY:
    // GetConsoleScreenBufferInfo is safe as long as screen_buffer_info is a valid size and raw_handle is a valid handle.
    // raw_handle is validated above.
    if unsafe { GetConsoleScreenBufferInfo(raw_handle, &mut screen_buffer_info) } == 0 {
        return None;
    }

    let w = (screen_buffer_info.srWindow.Right - screen_buffer_info.srWindow.Left + 1) as u16;
    let h = (screen_buffer_info.srWindow.Bottom - screen_buffer_info.srWindow.Top + 1) as u16;
    Some((w, h))
}
