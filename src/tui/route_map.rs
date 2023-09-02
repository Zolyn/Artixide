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
            casey::lower!(
                mod $variant;
            );
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
                    casey::lower!(use $variant::*;);
                )*

                RouteMap::from_unchecked(
                    [
                        $(
                            ($name::$variant, <concat_idents::concat_idents!(view = Wrapped, $variant, { view })>::init())
                        ),*
                    ]
                )
            }
        }
    };
}

pub(super) use RouteMap;
