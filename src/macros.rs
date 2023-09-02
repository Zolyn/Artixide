// Mostly copied from thread_local!
#[macro_export]
macro_rules! lazy {
    () => {};

    ($vis:vis static $name:ident: $t:ty = $init:expr; $($rest:tt)*) => (
        $vis static $name: once_cell::sync::Lazy<$t> = once_cell::sync::Lazy::new(|| $init);
        $crate::lazy!($($rest)*);
    );

    ($vis:vis static $name:ident: $t:ty = $init:expr) => (
        $vis static $name: once_cell::sync::Lazy<$t> = once_cell::sync::Lazy::new(|| $init);
    )
}

#[macro_export]
macro_rules! let_irrefutable {
    ($v:expr, $p:pat) => {
        let $p = $v else { unreachable!() };
    };
}

#[macro_export]
macro_rules! match_irrefutable {
    ($v:expr, $p:pat, $ret:expr) => {{
        match $v {
            $p => $ret,
            _ => unreachable!(),
        }
    }};
}

#[macro_export]
macro_rules! assert_call_once {
    () => {
        $crate::assert_call_once!("This function can be only called once.")
    };
    ($($args:tt)+) => {{
        use std::sync::atomic::{AtomicBool, Ordering};

        static CALLED: AtomicBool = AtomicBool::new(false);

        let called = CALLED.swap(true, Ordering::Relaxed);
        assert!(called == false, $($args)+)
    }}
}
