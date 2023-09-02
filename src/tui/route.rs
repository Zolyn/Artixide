use strum::{EnumCount, EnumString};

use crate::assert_call_once;

use super::views::View;

macro_rules! make_route {
    (
        $(#[$enum_meta:meta])*
        $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident
            ),*
            $(,)*
        }
    ) => {
        macro_rules! make_route_map {
            () => {{
                $(
                    use $crate::tui::views::$variant;
                )*

                RouteMap::from_unchecked(
                    [
                        $(
                            ($name::$variant, <$variant>::init())
                        ),*
                    ]
                )
            }}
        }

        $(#[$enum_meta])*
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant
            ),*
        }
    };
}

make_route! {
    #[derive(Debug, Clone, Copy, EnumCount, EnumString)]
    Route {
        Main,
        #[strum(serialize = "Keyboard layout")]
        Keyboard,
        Mirror,
        Locale,
        Timezone,
        Partition,
    }
}

#[derive(Debug)]
pub struct RouteMap {
    inner: Vec<Box<dyn View>>,
}

impl RouteMap {
    pub fn get_mut(&mut self, route: Route) -> &mut dyn View {
        let index = route as usize;
        self.inner[index].as_mut()
    }

    fn from_unchecked(arr: [(Route, Box<dyn View>); Route::COUNT]) -> Self {
        Self {
            inner: arr.into_iter().map(|(_, view)| view).collect(),
        }
    }

    pub fn new() -> Self {
        assert_call_once!();
        make_route_map!()
    }
}
