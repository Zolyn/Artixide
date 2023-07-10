pub mod customs;
pub mod menu;

#[macro_export]
macro_rules! build_styled_widget {
    ($widget:ident, $builder:expr) => {
        $builder.unwrap_or_default().build($widget::default())
    };
}
