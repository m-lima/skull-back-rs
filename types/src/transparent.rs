#[macro_export]
macro_rules! impl_fmt {
    ($self: ty, $fmt: ident, $($rest: ident),*) => {
        impl_fmt!($self, $fmt);
        impl_fmt!($self, $($rest),*);
    };

    ($self: ty, $fmt: ident) => {
        impl std::fmt::$fmt for $self {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

#[macro_export]
macro_rules! transparent {
    (nofmt readonly $self: ty, $target: ty) => {
        impl From<$self> for $target {
            fn from(value: $self) -> Self {
                value.0
            }
        }

        impl AsRef<$target> for $self {
            fn as_ref(&self) -> &$target {
                &self.0
            }
        }

        impl std::ops::Deref for $self {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }
    };
    (nofmt $self: ty, $target: ty) => {
        transparent!(nofmt readonly $self, $target);

        impl From<$target> for $self {
            fn from(value: $target) -> Self {
                Self(value)
            }
        }

        impl AsMut<$target> for $self {
            fn as_mut(&mut self) -> &mut $target {
                &mut self.0
            }
        }

        impl std::ops::DerefMut for $self {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.as_mut()
            }
        }
    };
    (readonly $self: ty, $target: ty) => {
        transparent!(nofmt readonly $self, $target);
        $crate::impl_fmt!($self, Display, Octal, Binary, UpperHex, LowerHex, UpperExp, LowerExp);
    };
    ($self: ty, $target: ty) => {
        transparent!(nofmt $self, $target);
        $crate::impl_fmt!($self, Display, Octal, Binary, UpperHex, LowerHex, UpperExp, LowerExp);
    };
}
