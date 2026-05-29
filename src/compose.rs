/// Implement [`FromStream`](crate::FromStream) for a struct or enum.
///
/// Note that each field must implement `FromStream<Output = Self>`. If you want
/// to use a type implementing `FromStream<Output = Other>`, use the syntax
/// `name: Other as Type`, where `Other` is the field type and `Type` is the
/// type implementing `FromStream`.
///
/// You can map fields using the syntax `name: Type as OtherType => { ... }`. In
/// this example, `Type` is the final type of the field (that needn't implement
/// `FromStream`), `OtherType` is a type implementing `FromStream`, and inside
/// the block is code converting `<OtherType as FromStream>::Output` (bound to
/// `name`) into `Type`.
///
/// Among the fields, you can also use `= expr` to match a literal expression
/// implementing [`MatchStream`](crate::MatchStream). You can also use what I
/// like to call the "Archibald operator", `;^,`, to insert a "cut" operator.
///
/// See [`decompose!`](crate::decompose!) for more information.
#[macro_export]
macro_rules! compose {
    ($($t:tt)*) => { $crate::__compose_inner! {$($t)*} };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __compose_inner {
    (
        $(#[$attrs:meta])*
        $v:vis struct $name:ident
        $(< $(
            $($lifetimes:lifetime)?
            $($generics:ident)?
            $(!const $const_generics:ident : $const_generic_types:ty)?
        ),* $(,)? >)?
        $(where [$($where:tt)*])?
        { $(
            $(
                $(#[$field_attrs:meta])*
                $field_v:vis $field_name:ident : $field_type:ty
                    $(as $field_map_type:ty $( => {$( $field_map:tt )*} )?)?
            )?
            $( = $match_expr:expr )?
            $( ; ^ $(@@ $cut:tt)? )?
        ,)+ }
    ) => {
        $crate::__compose_inner! {
            $(#[$attrs])*
            $v struct $name
            $(< $(
                $($lifetimes)?
                $($generics)?
                $(!const $const_generics : $const_generic_types)?
            ),* >)?
            $(where [$($where)*])?
            { $(
                $(
                    $(#[$field_attrs])*
                    $field_v $field_name : $field_type
                        $(as $field_map_type $( => {$( $field_map )*} )?)?
                )?
                $( = $match_expr )?
                $( ; ^ $($cut)? )?
            ),+ }
        }
    };

    (
        $(#[$attrs:meta])*
        $v:vis struct $name:ident
        $(< $(
            $($lifetimes:lifetime)?
            $($generics:ident)?
            $(!const $const_generics:ident : $const_generic_types:ty)?
        ),* >)?
        $(where [$($where:tt)*])?
        { $(
            $(
                $(#[$field_attrs:meta])*
                $field_v:vis $field_name:ident : $field_type:ty
                    $(as $field_map_type:ty $( => {$( $field_map:tt )*})?)?
            )?
            $( = $match_expr:expr )?
            $( ; ^ $(@@ $cut:tt)? )?
        ),+ }
    ) => {
        $(#[$attrs])*
        $v struct $name
            $(< $(
                $($lifetimes)?
                $($generics)?
                $(const $const_generics: $const_generic_types)?
            ),* >)?
            $(where $($where)*)?
        { $(
            $(
                $(#[$field_attrs])*
                $field_v $field_name : $field_type,
            )?
        )+ }

        impl
            $(< $(
                $($lifetimes)?
                $($generics)?
                $(const $const_generics: $const_generic_types)?
            ),* >)?
            $crate::FromStream for $name
                $(< $(
                    $($lifetimes)?
                    $($generics)?
                    $($const_generics)?
                ),* >)?
            $(where $($where)*)?
        {
            type Output = Self;

            fn from_stream<S>(mut stream: &mut S) -> core::result::Result<Self, $crate::Error>
            where
                S: $crate::StreamLike,
            {
                $crate::decompose! {
                    in stream;

                $(
                    $( ^; $($cut:tt)? )?
                    $( $field_name: $crate::__if_else!([$($field_map_type)?] else [$field_type]); )?
                    $( = $match_expr; )?
                )+
                }

                $($($($( let $field_name = { $($field_map)* }; )?)?)?)+

                Ok(Self {$($($field_name,)?)+})
            }
        }
    };

    (
        $(#[$attrs:meta])*
        $v:vis enum $name:ident
        $(< $(
            $($lifetimes:lifetime)?
            $($generics:ident)?
            $(!const $const_generics:ident : $const_generic_types:ty)?
        ),* $(,)? >)?
        $(where [$($where:tt)*])?
        { $(
            $(#[$variant_attrs:meta])*
            $variant_name:ident
                $(($variant_inner:ty))?
                $( = $variant_match:expr )?
        ),+ $(,)? }
    ) => {
        $(#[$attrs])*
        $v enum $name
            $(< $(
                $($lifetimes)?
                $($generics)?
                $(const $const_generics: $const_generic_types)?
            ),* >)?
            $(where $($where)*)?
        { $(
            $(#[$variant_attrs])*
            $variant_name $((<$variant_inner as $crate::FromStream>::Output))?,
        )+ }

        impl
            $(< $(
                $($lifetimes)?
                $($generics)?
                $(const $const_generics: $const_generic_types)?
            ),* >)?
            $crate::FromStream for $name
                $(< $(
                    $($lifetimes)?
                    $($generics)?
                    $($const_generics)?
                ),* >)?
            $(where $($where)*)?
        {
            type Output = Self;

            fn from_stream<S>(stream: &mut S) -> core::result::Result<Self, $crate::Error>
            where
                S: $crate::StreamLike,
            {
                // TODO: is there a better way to aggregate errors?
                let mut err;
            $(
                $(
                    let mut view = stream.view();
                    match <$variant_inner as $crate::FromStream>::from_stream(&mut view) {
                        Ok(x) => {
                            let skipped = view.skipped();
                            stream.skip(skipped);
                            return Ok(Self::$variant_name(x));
                        }
                        Err(e) if e.fatal => return Err(e),
                        Err(e) => {
                            err = e;
                            view.reset_skip();
                        }
                    }
                )?
                $(
                    if let Ok(n) = $crate::MatchStream::match_stream(&$variant_match, stream.view()) {
                        stream.skip(n);
                        return Ok(Self::$variant_name);
                    }
                )?
            )+
                Err(err)
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __if_else {
    ([] else [$($t:tt)*]) => ($($t)*);
    ([$($t:tt)*] else [$($_:tt)*]) => ($($t)*);
}
