macro_rules! default_struct {
    ($name:ident {$($field:ident : $type:ty = $default:expr),* $(,)?}) => {
        #[derive(Clone, Copy, PartialEq, Debug)]
        pub struct $name {
            $(pub $field: $type,)*
        }
        impl $name {
            pub fn new() -> Self {
                Self { $($field: $default,)* }
            }
            $(
            pub fn $field(self, $field: $type) -> Self {
                Self { $field, ..self }
            }
            )*
        }
    };
}
pub(crate) use default_struct;
