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
