#[macro_export]
macro_rules! impl_math_wrapper_any {
    // literal N: LeBytes<10>
    ($wrapper:ident, LeBytes<$n:literal>) => {
        $crate::impl_math_wrapper_any!(@impl $wrapper, $n);
    };
    // const ident N: LeBytes<BALANCE_BYTES>
    ($wrapper:ident, LeBytes<$n:ident>) => {
        $crate::impl_math_wrapper_any!(@impl $wrapper, $n);
    };

    (@impl $wrapper:ident, $n:tt) => {
        // Static constraints (N must fit into U256 to keep From<$wrapper> for U256 infallible).
        const _: () = assert!($n > 0, "LeBytes<N>: N must be > 0");
        const _: () = assert!($n <= 32, "LeBytes<N>: N must be <= 32");

        // Force type correctness in const context.
        const _: $wrapper = $wrapper(
            <::awesome_sails_utils::math::LeBytes<$n> as ::awesome_sails_utils::math::Zero>::ZERO
        );

        // NonZero conversions (local, to avoid depending on internal macros)
        impl ::core::convert::TryFrom<$wrapper> for ::awesome_sails_utils::math::NonZero<$wrapper> {
            type Error = ::awesome_sails_utils::math::ZeroError;
            #[inline]
            fn try_from(value: $wrapper) -> ::core::result::Result<Self, Self::Error> {
                ::awesome_sails_utils::math::NonZero::try_new(value)
            }
        }

        impl ::core::convert::From<::awesome_sails_utils::math::NonZero<$wrapper>> for $wrapper {
            #[inline]
            fn from(value: ::awesome_sails_utils::math::NonZero<$wrapper>) -> Self {
                value.into_inner()
            }
        }

        // Math traits
        impl ::awesome_sails_utils::math::Max for $wrapper {
            const MAX: Self = Self(<::awesome_sails_utils::math::LeBytes<$n>>::MAX);
        }
        impl ::awesome_sails_utils::math::Min for $wrapper {
            const MIN: Self = Self(<::awesome_sails_utils::math::LeBytes<$n>>::MIN);
        }
        impl ::awesome_sails_utils::math::Zero for $wrapper {
            const ZERO: Self = Self(<::awesome_sails_utils::math::LeBytes<$n>>::ZERO);
        }
        impl ::awesome_sails_utils::math::One for $wrapper {
            const ONE: Self = Self(<::awesome_sails_utils::math::LeBytes<$n>>::ONE);
        }

        impl ::awesome_sails_utils::math::CheckedMath for $wrapper {
            #[inline]
            fn checked_add(self, rhs: Self) -> Option<Self> {
                self.0.checked_add(rhs.0).map(Self)
            }
            #[inline]
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                self.0.checked_sub(rhs.0).map(Self)
            }
        }

        // Comparisons helpers
        impl PartialEq<::awesome_sails_utils::math::NonZero<$wrapper>> for $wrapper {
            #[inline]
            fn eq(&self, other: &::awesome_sails_utils::math::NonZero<$wrapper>) -> bool {
                self.eq(&other.into_inner())
            }
        }
        impl PartialOrd<::awesome_sails_utils::math::NonZero<$wrapper>> for $wrapper {
            #[inline]
            fn partial_cmp(
                &self,
                other: &::awesome_sails_utils::math::NonZero<$wrapper>,
            ) -> Option<::core::cmp::Ordering> {
                self.partial_cmp(&other.into_inner())
            }
        }

        impl PartialEq<::awesome_sails_utils::math::LeBytes<$n>> for $wrapper {
            #[inline]
            fn eq(&self, other: &::awesome_sails_utils::math::LeBytes<$n>) -> bool {
                self.0 == *other
            }
        }
        impl PartialOrd<::awesome_sails_utils::math::LeBytes<$n>> for $wrapper {
            #[inline]
            fn partial_cmp(
                &self,
                other: &::awesome_sails_utils::math::LeBytes<$n>,
            ) -> Option<::core::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }

        // Checked conversion FROM U256 into wrapper
        impl ::core::convert::TryFrom<::awesome_sails_utils::math::U256> for $wrapper {
            type Error = ::awesome_sails_utils::math::OverflowError;
            #[inline]
            fn try_from(
                value: ::awesome_sails_utils::math::U256,
            ) -> ::core::result::Result<Self, Self::Error> {
                let inner = ::awesome_sails_utils::math::LeBytes::<$n>::try_from(value)
                    .map_err(|_| ::awesome_sails_utils::math::OverflowError)?;
                Ok(Self(inner))
            }
        }

        // Infallible conversion INTO U256 (N <= 32 enforced above)
        impl ::core::convert::From<$wrapper> for ::awesome_sails_utils::math::U256 {
            #[inline]
            fn from(value: $wrapper) -> ::awesome_sails_utils::math::U256 {
                <::awesome_sails_utils::math::U256 as ::core::convert::TryFrom<
                    ::awesome_sails_utils::math::LeBytes<$n>
                >>::try_from(value.0)
                .unwrap_or_else(|_| unreachable!("LeBytes<N> must fit into U256 for N <= 32"))
            }
        }

        // Fallible conversion into u128 (useful for tests/logs)
        impl ::core::convert::TryFrom<$wrapper> for u128 {
            type Error = ::awesome_sails_utils::math::OverflowError;
            #[inline]
            fn try_from(value: $wrapper) -> ::core::result::Result<u128, Self::Error> {
                <u128 as ::core::convert::TryFrom<::awesome_sails_utils::math::LeBytes<$n>>>::try_from(value.0)
                    .map_err(|_| ::awesome_sails_utils::math::OverflowError)
            }
        }

        // Lossless conversions wrapper <-> LeBytes<N>
        impl ::core::convert::From<::awesome_sails_utils::math::LeBytes<$n>> for $wrapper {
            #[inline]
            fn from(v: ::awesome_sails_utils::math::LeBytes<$n>) -> Self {
                Self(v)
            }
        }
        impl ::core::convert::From<$wrapper> for ::awesome_sails_utils::math::LeBytes<$n> {
            #[inline]
            fn from(v: $wrapper) -> Self {
                v.0
            }
        }
    };
}
