//! GPIO Configuration

// TODO: GPIO: Proper top-level module documentation.
// BODY: This needs examples and high-level explanations, ideally with links
// BODY: to the reference manual.

use core::marker::PhantomData;

pub trait GpioExt {
    type Parts;
    fn split(self) -> Self::Parts;
}

pub struct Floating;
pub struct PullDown;
pub struct PullUp;

pub struct Input<MODE> {
    _mode: PhantomData<MODE>
}

// SSP
pub struct SCK0;
pub struct SSEL0;
pub struct MOSI0;
pub struct MISO0;
pub struct SCK1;
pub struct SSEL1;
pub struct MOSI1;
pub struct MISO1;

// I2C
pub struct SCL;
pub struct SDA;

// USART
pub struct RXD;
pub struct TXD;
pub struct CTS;
pub struct RTS;
pub struct DTR;
pub struct DSR;
pub struct DCD;
pub struct RI;

macro_rules! gpio_func {
   ($PXi:ident, $iocon_pio_name:ident, $into_func:ident -> $FUNC:ty { $iocon_func_name:ident }) => {
        pub fn $into_func(self) -> $PXi<$FUNC> {
            // TODO: Safety note.
            let iocon = unsafe { &*lpc11uxx::IOCON::ptr() };
            iocon.$iocon_pio_name.write(|v| v
                .func().$iocon_func_name());
            $PXi { _mode: PhantomData }
        }
   };
   ($PXi:ident, $iocon_pio_name:ident, $into_func:ident -> $FUNC:ty { $iocon_func_name:ident, $iocon_mode_name:ident }) => {
        pub fn $into_func(self) -> $PXi<$FUNC> {
            let iocon = unsafe { &*lpc11uxx::IOCON::ptr() };
            iocon.$iocon_pio_name.write(|v| v
                .func().$iocon_func_name()
                .mode().$iocon_mode_name());
            $PXi { _mode: PhantomData }
        }
   };
}

macro_rules! gpio {
    ($($port:ident: [
        $($PXi:ident: ($pxi:ident, $iocon_pio_name:ident, $DEFAULT_FUNC:ty, [
            $($into_func:ident -> $FUNC:ty { $($tt:tt)* }),*
        ]),)+
    ]),*) => {
        pub struct Parts {
            $($(pub $pxi: $port::$PXi<$DEFAULT_FUNC>,)*)*
        }

        impl GpioExt for lpc11uxx::IOCON {
            type Parts = Parts;
            fn split(self) -> Parts {
                Parts {
                    $($($pxi: $port::$PXi { _mode: PhantomData },)*)*
                }
            }
        }

        $(
        pub mod $port {
            use core::marker::PhantomData;
            use super::*;

            $(
            pub struct $PXi<FUNC> {
                pub(super) _mode: PhantomData<FUNC>
            }

            impl<FUNC> $PXi<FUNC> {
                $(gpio_func!($PXi, $iocon_pio_name, $into_func -> $FUNC { $($tt)* });)*
            }

            )*
        }
        )*
    }
}

gpio! {
    gpio0: [
        Pio2: (gpio0_pio2, pio0_2, Input<Floating>, [
            into_ssel0 -> SSEL0 { ssel0, pull_up }
        ]),
        Pio4: (gpio0_pio4, pio0_4, Input<Floating>, [
            into_scl -> SCL { i2c_scl }
        ]),
        Pio5: (gpio0_pio5, pio0_5, Input<Floating>, [
            into_sda -> SDA { i2c_sda }
        ]),
        Pio6: (gpio0_pio6, pio0_6, Input<Floating>, [
            into_sck0 -> SCK0 { sck0, pull_up }
        ]),
        Pio8: (gpio0_pio8, pio0_8, Input<Floating>, [
            into_miso0 -> MISO0 { miso0, pull_up }
        ]),
        Pio9: (gpio0_pio9, pio0_9, Input<Floating>, [
            into_mosi0 -> MOSI0 { mosi0, pull_up }
        ]),
        Pio10: (gpio0_pio10, swclk_pio0_10, Input<Floating>, [
            into_sck0 -> SCK0 { sck0, pull_up }
        ]),
        Pio18: (gpio0_pio18, pio0_18, Input<Floating>, [
            into_rxd -> RXD { rxd }
        ]),
        Pio19: (gpio0_pio19, pio0_19, Input<Floating>, [
            into_txd -> TXD { txd }
        ]),
        Pio21: (gpio0_pio21, pio0_21, Input<Floating>, [
            into_mosi1 -> MOSI1 { mosi1, pull_up }
        ]),
        Pio22: (gpio0_pio22, pio0_22, Input<Floating>, [
            into_miso1 -> MISO1 { miso1, pull_up }
        ]),
    ],
    gpio1: [
        Pio13: (gpio1_pio13, pio1_13, Input<Floating>, [
            into_txd -> TXD { txd }
        ]),
        Pio14: (gpio1_pio14, pio1_14, Input<Floating>, [
            into_rxd -> RXD { rxd }
        ]),
        Pio15: (gpio1_pio15, pio1_15, Input<Floating>, [
            into_sck1 -> SCK1 { sck1, pull_up }
        ]),
        Pio19: (gpio1_pio19, pio1_19, Input<Floating>, [
            into_ssel1 -> SSEL1 { ssel1, pull_up }
        ]),
        Pio20: (gpio1_pio20, pio1_20, Input<Floating>, [
            into_sck1 -> SCK1 { sck1, pull_up }
        ]),
        Pio21: (gpio1_pio21, pio1_21, Input<Floating>, [
            into_miso1 -> MISO1 { miso1, pull_up }
        ]),
        Pio22: (gpio1_pio22, pio1_22, Input<Floating>, [
            into_mosi1 -> MOSI1 { mosi1, pull_up }
        ]),
        Pio23: (gpio1_pio23, pio1_23, Input<Floating>, [
            into_ssel1 -> SSEL1 { ssel1, pull_up }
        ]),
        Pio26: (gpio1_pio26, pio1_26, Input<Floating>, [
            into_rxd -> RXD { rxd }
        ]),
        Pio27: (gpio1_pio27, pio1_27, Input<Floating>, [
            into_txd -> TXD { txd }
        ]),
        Pio29: (gpio1_pio29, pio1_29, Input<Floating>, [
            into_sck0 -> SCK0 { sck0, pull_up }
        ]),
    ]
}