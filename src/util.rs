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

macro_rules! binary_op {
    // [(.x) ()] [(.y) ()] [(.z) ()] [(V3) (Add) (+) (f64)] (V3) + = (f64)
    // [(.x) () (.x)] [(.y) () (.y)] [(.z) () (.z)] [(V3) (Add) (+) (f64)] (V3) + (f64) -> (V3)
    ([$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [$($types:tt)*] + $($rest:tt)*) => {
        binary_op!([$($a)*] [$($b)*] [$($c)*] [$($types)* (Add) (+)] $($rest)*);
    };
    ([$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [$($types:tt)*] - $($rest:tt)*) => {
        binary_op!([$($a)*] [$($b)*] [$($c)*] [$($types)* (Sub) (-)] $($rest)*);
    };
    ([$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [$($types:tt)*] * $($rest:tt)*) => {
        binary_op!([$($a)*] [$($b)*] [$($c)*] [$($types)* (Mul) (*)] $($rest)*);
    };
    ([$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [$($types:tt)*] / $($rest:tt)*) => {
        binary_op!([$($a)*] [$($b)*] [$($c)*] [$($types)* (Div) (/)] $($rest)*);
    };
    ([$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [$($types:tt)*] = $($rest:tt)*) => {
        binary_op!([$($a)*] [$($b)*] [$($c)*] [$($types)* (=)] $($rest)*);
    };
    ([$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [$($types:tt)*] -> $($rest:tt)*) => {
        binary_op!([$($a)*] [$($b)*] [$($c)*] [$($types)*] $($rest)*);
    };
    ([$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [$($types:tt)*] (f64) $($rest:tt)*) => {
        binary_op!([$($a)* ()] [$($b)* ()] [$($c)* ()] [$($types)* (f64)] $($rest)*);
    };
    ([$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [$($types:tt)*] (&f64) $($rest:tt)*) => {
        binary_op!([$($a)* ()] [$($b)* ()] [$($c)* ()] [$($types)* (&f64)] $($rest)*);
    };
    ([$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [$($types:tt)*] ($type:ty) $($rest:tt)*) => {
        binary_op!([$($a)* (.x)] [$($b)* (.y)] [$($c)* (.z)] [$($types)* ($type)] $($rest)*);
    };
    ([$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [$($types:tt)*] (& $type:ty) $($rest:tt)*) => {
        binary_op!([$($a)* (.x)] [$($b)* (.y)] [$($c)* (.z)] [$($types)* (&$type)] $($rest)*);
    };
    (
        [($($a1:tt)*) ($($a2:tt)*) ($($a3:tt)*)] [($($b1:tt)*) ($($b2:tt)*) ($($b3:tt)*)] [($($c1:tt)*) ($($c2:tt)*) ($($c3:tt)*)]
        [($lhs:ty) ($trait:path) ($op:tt) ($rhs:ty) ($res:ty)]
    ) => {
        paste! {
            impl $trait<$rhs> for $lhs {
                type Output = $res;
                fn [<$trait:lower>](self, rhs: $rhs) -> Self::Output {
                    Self::Output::new()
                        $($a3)*(self $($a1)* $op rhs $($a2)*)
                        $($b3)*(self $($b1)* $op rhs $($b2)*)
                        $($c3)*(self $($c1)* $op rhs $($c2)*)
                }
            }
        }
    };
    (
        [($($a1:tt)*) ($($a2:tt)*)] [($($b1:tt)*) ($($b2:tt)*)] [($($c1:tt)*) ($($c2:tt)*)]
        [($lhs:ty) ($trait:path) ($op:tt) (=) ($rhs:ty)]
    ) => {
        paste! {
            impl [<$trait Assign>]<$rhs> for $lhs {
                fn [<$trait:lower _assign>](&mut self, rhs: $rhs) {
                    self $($a1)* = self $($a1)* $op rhs $($a2)*;
                    self $($b1)* = self $($b1)* $op rhs $($b2)*;
                    self $($c1)* = self $($c1)* $op rhs $($c2)*;
                }
            }
        }
    };
}

macro_rules! define_ops {
    (($($lhs:tt)*) () $($rest:tt)*) => {};
    (($($lhs:tt)*) ($op:tt $($rest:tt)*) = ($($rhs:tt)*) ) => {
        binary_op!([] [] [] [] ($($lhs)*) $op = ($($rhs)*));
        binary_op!([] [] [] [] ($($lhs)*) $op = (&$($rhs)*));
        define_ops!(($($lhs)*) ($($rest)*) = ($($rhs)*));
    };
    (($($lhs:tt)*) ($op:tt $($rest:tt)*) ($($rhs:tt)*) -> ($($res:tt)*)) => {
        binary_op!([] [] [] [] ($($lhs)*) $op ($($rhs)*) -> ($($res)*));
        binary_op!([] [] [] [] ($($lhs)*) $op (&$($rhs)*) -> ($($res)*));
        binary_op!([] [] [] [] (&$($lhs)*) $op ($($rhs)*) -> ($($res)*));
        binary_op!([] [] [] [] (&$($lhs)*) $op (&$($rhs)*) -> ($($res)*));
        define_ops!(($($lhs)*) ($($rest)*) ($($rhs)*) -> ($($res)*));
    };
}

pub(crate) use binary_op;
pub(crate) use default_struct;
pub(crate) use define_ops;
