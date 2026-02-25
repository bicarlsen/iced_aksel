use iced_core::{keyboard, mouse};

#[derive(Debug)]
pub(crate) enum Handler<Message: Clone, F> {
    Direct(Message),
    Closure(F),
}

pub trait Invoke<Args> {
    type Output;
    fn invoke(&self, args: Args) -> Self::Output;
}

macro_rules! impl_invoke {
    () => {
        impl<F, R> Invoke<()> for F
        where
            F: Fn() -> R,
        {
            type Output = R;
            fn invoke(&self, _args: ()) -> R { (self)() }
        }
    };
    ($($T:ident),+) => {
        impl<F, R, $($T),+> Invoke<($($T,)+)> for F
        where
            F: Fn($($T),+) -> R,
        {
            type Output = R;
            #[allow(non_snake_case)]
            fn invoke(&self, args: ($($T,)+)) -> R {
                // destructure the tuple into separate args
                let ($($T,)+) = args;
                (self)($($T),+)
            }
        }
    };
}

impl_invoke!();
impl_invoke!(A);
impl_invoke!(A, B);
impl_invoke!(A, B, C);
impl_invoke!(A, B, C, D);

// Normalize return type to Option<Message>
pub trait IntoOption<T> {
    fn into_option(self) -> Option<T>;
}

impl<T> IntoOption<T> for T {
    fn into_option(self) -> Option<T> {
        Some(self)
    }
}

impl<T> IntoOption<T> for Option<T> {
    fn into_option(self) -> Option<T> {
        self
    }
}

impl<Message: Clone, F> Handler<Message, F> {
    pub fn run<Args>(&self, args: Args) -> Option<Message>
    where
        F: Invoke<Args>,
        <F as Invoke<Args>>::Output: IntoOption<Message>,
    {
        match self {
            Self::Direct(m) => Some(m.clone()),
            Self::Closure(f) => f.invoke(args).into_option(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PressEvent<P> {
    pub position: P,
    pub button: mouse::Button,
    pub click_kind: mouse::click::Kind,
    pub modifiers: keyboard::Modifiers,
}

impl<P> PressEvent<P> {
    pub(crate) fn new(
        position: P,
        button: mouse::Button,
        click_kind: mouse::click::Kind,
        modifiers: keyboard::Modifiers,
    ) -> Self {
        Self {
            position,
            button,
            click_kind,
            modifiers,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReleaseEvent<P> {
    pub position: P,
    pub button: mouse::Button,
    pub click_kind: Option<mouse::click::Kind>,
    pub modifiers: keyboard::Modifiers,
}

impl<P> ReleaseEvent<P> {
    pub(crate) fn new(
        position: P,
        button: mouse::Button,
        click_kind: Option<mouse::click::Kind>,
        modifiers: keyboard::Modifiers,
    ) -> Self {
        Self {
            position,
            button,
            click_kind,
            modifiers,
        }
    }
}
