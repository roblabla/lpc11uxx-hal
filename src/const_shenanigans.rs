// Leo60228 you're a fucking genius!
#[allow(dead_code)] // this shouldn't be necessary but it is
pub mod never {
    pub type F = fn() -> !;

    pub trait HasOutput {
        type Output;
    }

    impl<O> HasOutput for fn() -> O {
        type Output = O;
    }

    pub type Never = <F as HasOutput>::Output;
}

macro_rules! minipanic {
    ($($ss:tt)*) => {{
        let default: [$crate::const_shenanigans::never::Never; 0] = [];
        // Force a panic and convert into !
        #[allow(unconditional_panic)]
        default[$crate::const_shenanigans::always_true() as usize]
    }}
}

macro_rules! miniassert {
    ($cond:expr, $($ss:tt)*) => {
        &[()][1 - ($cond as usize)]
    }
}

pub const fn always_true() -> bool {
    true
}