macro_rules! RouteMap {
    (
        $(#[$enum_meta:meta])*
        $vis:vis
        enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident
            ),*
            $(,)*
        }
    ) => {
        $(
            paste::paste! {
                mod [<$variant:lower>];
            }
        )*

        #[derive(Debug)]
        pub struct RouteMap {
            inner: Vec<Box<dyn View>>,
        }

        impl RouteMap {
            pub fn get_mut(&mut self, route: $name) -> &mut dyn View {
                let index = route as usize;
                self.inner[index].as_mut()
            }

            fn from_unchecked(arr: [($name, Box<dyn View>); <$name>::COUNT]) -> Self {
                Self {
                    inner: arr.into_iter().map(|(_, view)| view).collect(),
                }
            }

            pub fn new() -> Self {
                $crate::assert_call_once!();

                $(
                    paste::paste! {
                        use [<$variant:lower>]::[<Wrapped $variant>];
                    }
                )*

                RouteMap::from_unchecked(
                    [
                        $(
                            ($name::$variant, <paste::paste!([<Wrapped $variant>])>::init())
                        ),*
                    ]
                )
            }
        }
    };
}

pub(super) use RouteMap;
