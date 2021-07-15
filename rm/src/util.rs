#![macro_use]

use std::process::exit;

#[macro_export]
macro_rules! print_err(
    ($msg:tt) => { {
        eprintln!("{}", format!("Error: {}", $msg));
    } };
    ($msg:tt, $($arg:tt)*) => { {
        eprintln!(concat!("Error: ", $msg), $($arg)*);
    } };
);

#[macro_export]
macro_rules! exit_with_err(
    ($($arg:tt)*) => { {
        print_err!($($arg)*);
        exit(2);
    } };
);

pub trait UnwrapOrExit<T> {
    fn unwrap_or_exit(self) -> T;
}
pub trait UnwrapOrExitWith<T> {
    fn unwrap_or_exit_with(self, err_msg: &str) -> T;
}

impl<T, E: std::fmt::Display> UnwrapOrExit<T> for Result<T, E> {
    fn unwrap_or_exit(self) -> T {
        self.unwrap_or_else(|err| exit_with_err!(err))
    }
}

impl<T, E> UnwrapOrExitWith<T> for Result<T, E> {
    fn unwrap_or_exit_with(self, err_msg: &str) -> T {
        self.unwrap_or_else(|_| {
            exit_with_err!(err_msg);
        })
    }
}

impl<T> UnwrapOrExitWith<T> for Option<T> {
    fn unwrap_or_exit_with(self, err_msg: &str) -> T {
        self.unwrap_or_else(|| {
            exit_with_err!(err_msg);
        })
    }
}
