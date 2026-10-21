#[macro_export]
macro_rules! print {
        () => {{
        $crate::sys::get_tty_dev().tty_flush();
    }};
    ($($arg:tt)*) => {{
        let formatted = $crate::alloc::format!($($arg)*);
        $crate::sys::get_tty_dev().tty_write(formatted.as_bytes());
        $crate::sys::get_tty_dev().tty_flush();
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        $crate::sys::get_tty_dev().tty_write(b"\r\n");
        $crate::sys::get_tty_dev().tty_flush();
    }};
    ($($arg:tt)*) => {{
        let formatted = $crate::alloc::format!($($arg)*);
        $crate::sys::get_tty_dev().tty_write(formatted.as_bytes());
        $crate::sys::get_tty_dev().tty_write(b"\r\n");
        $crate::sys::get_tty_dev().tty_flush();
    }};
}
