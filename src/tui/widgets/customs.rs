pub mod block_style;

#[macro_export]
macro_rules! set_if {
    ($widget:ident, $self:ident, $($field:ident),*) => {
        $(
            if let Some(field) = $self.$field {
                $widget = $widget.$field(field)
            };
        )*
    };
}
