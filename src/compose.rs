/// Implement [`FromStream`](crate::FromStream) for a struct or enum.
///
/// Note that each field's actual type is `<T as FromStream>::Output`, *not* `T`
/// itself.
///
/// Among the fields, you can also use `= expr` to match a literal expression
/// implementing [`MatchStream`](crate::MatchStream). You can also use what I
/// like to call the "Archibald operator", `;^,`, to insert a "cut" operator.
///
/// See [`decompose!`](crate::decompose) for more information.
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
                $field_v $field_name : <$field_type as $crate::FromStream>::Output,
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
                    $( $field_name: $field_type; )?
                    $( = $match_expr; )?
                )+
                }

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
            $variant_name:ident($variant_inner:ty)
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
            $variant_name(<$variant_inner as $crate::FromStream>::Output),
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
                let mut view = stream.view();
            $(
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
            )+
                Err(err)
            }
        }
    };
}
