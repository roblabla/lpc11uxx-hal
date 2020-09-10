//! LPC11UXX Clock Configuration
//!
//! The LPC11UXX, like all good ARM boards, has a metric shit ton of clocks that
//! can be configured to drive the various devices of the chip. We'll provide a
//! quick overview of what those clocks are, how they interact with each-other,
//! and how we configure them.
//!
//! # LPC11UXX clocks primer
//!
//! I highly suggest people to refer to [Chapter 3.4] of the LPC11UXX User
//! Manual, which contains a nice schema of how the clocks interact with
//! each-other.
//!
//! [Chapter 3.4]: https://ia801503.us.archive.org/7/items/um10462/UM10462.pdf#I4.5.2159674
//!
//! The chip has the following clocks:
//!
//! - Internal RC Oscillator (IRC), runs at 12MHz, with very bad precision.
//! - System Oscillator (SYSOSC), external to the chip and provided by the user.
//! - Watchdog Oscillator (WDCLK), external oscillator that can be used as a
//!   dedicated oscillator for the watchdog task.
//! - System PLL (SYSPLL), a phase-locked loop using either IRC or SYSOSC as an
//!   input.
//! - USB PLL (USBPLL), a phase-locked loop using either IRC or SYSOSC as an
//!   input, optionally used as the USB input clock.
//! - Main Clock, an alias to either IRC, SYSOSC, WDCLK or SYSPLL. Used as an
//!   input clock for many other clocks.
//! - System Clock, the clock that drives the CPU, system control, and PMU. It's
//!   a simple clock divider taking the Main Clock as an input. All the
//!   peripherals enabled with SYSAHBCLKCTRL take their input clocks from the
//!   System Clock.
//! - SSP0 Clock, UART Clock, SSP1 Clock are all clocks that drive their
//!   respective components. They are each configured as a simple divider from
//!   the main clock.
//! - USB Clock drives the USB controller clock. It's a divider in front of
//!   either the USBPLL or the Main Clock.
//!
//! # HAL
//!
//! The lpc11uxx-hal expects the user to configure all clocks at the start of
//! the program, and freeze their values. This is to ensure that the clocks
//! aren't changed while a device is in operation, which would cause it to run
//! at the wrong frequency.
//!
//! Clocks can be configured using an easy-to-use API where the user provides
//! the necessary frequencies for the various clocks.
//!

use lpc11uxx::SYSCON as SYSCON_IMPL;
use lpc11uxx::FLASHCTRL;
use lpc11uxx::syscon::{mainclksel, syspllclksel, wdtoscctrl, usbpllclksel};
use lpc11uxx::flashctrl::flashcfg::FLASHTIM_A;

#[derive(Debug, Clone, Copy)]
pub struct Hertz(pub u32);

impl core::ops::Mul<u32> for Hertz {
    type Output = Hertz;
    fn mul(self, rhs: u32) -> Hertz {
        Hertz(self.0 + rhs)
    }
}

const IRC_OSCILLATOR_FREQUENCY: Hertz = Hertz(12_000_000);

const WDT_OSC_RATE: &[u32] = &[
	0,					/* WDT_OSC_ILLEGAL */
	600000,				/* WDT_OSC_0_60 */
	1050000,			/* WDT_OSC_1_05 */
	1400000,			/* WDT_OSC_1_40 */
	1750000,			/* WDT_OSC_1_75 */
	2100000,			/* WDT_OSC_2_10 */
	2400000,			/* WDT_OSC_2_40 */
	2700000,			/* WDT_OSC_2_70 */
	3000000,			/* WDT_OSC_3_00 */
	3250000,			/* WDT_OSC_3_25 */
	3500000,			/* WDT_OSC_3_50 */
	3750000,			/* WDT_OSC_3_75 */
	4000000,			/* WDT_OSC_4_00 */
	4200000,			/* WDT_OSC_4_20 */
	4400000,			/* WDT_OSC_4_40 */
	4600000				/* WDT_OSC_4_60 */
];

pub fn main_clock_rate(syscon: &SYSCON_IMPL) -> Hertz {
    let pdruncfg = syscon.pdruncfg.read();
    let syscon_pll = if pdruncfg.syspll_pd().is_powered() {
        let clksel = syscon.syspllclksel.read().sel().variant();
        let pllctrl = syscon.syspllctrl.read();
        let m = pllctrl.msel().bits();
        let p = pllctrl.psel().bits();
        Some((clksel, m, p))
    } else {
        None
    };
    let watchdog_osc = if pdruncfg.wdtosc_pd().is_powered() {
        let wdtosc = syscon.wdtoscctrl.read();
        if let lpc11uxx::generic::Variant::Val(freqsel) = wdtosc.freqsel().variant() {
            Some((freqsel, wdtosc.divsel().bits()))
        } else {
            None
        }
    } else {
        None
    };
    let mainclksel = syscon.mainclksel.read().sel().variant();
    // TODO: System Oscillator.
    get_main_clock_freq(mainclksel,  syscon_pll, watchdog_osc, Some(Hertz(12_000_000)))
}

const fn unwrap<T: Copy>(opt: Option<T>) -> T {
    match opt {
        Some(v) => v,
        None => minipanic!()
    }
}

const fn wdtosc_freq_to_hertz(freq: wdtoscctrl::FREQSEL_A) -> Hertz {
    Hertz(WDT_OSC_RATE[freq as usize])
}

const fn get_main_clock_freq(main_clock_source: mainclksel::SEL_A, system_pll: Option<(syspllclksel::SEL_A, u8, u8)>, watchdog_osc: Option<(wdtoscctrl::FREQSEL_A, u8)>, sys_osc_freq: Option<Hertz>) -> Hertz {
    match (main_clock_source, system_pll, watchdog_osc) {
        (mainclksel::SEL_A::IRC_OSCILLATOR, _, _) => IRC_OSCILLATOR_FREQUENCY,
        (mainclksel::SEL_A::PLL_INPUT, Some((syspllclksel::SEL_A::IRC, ..)), _) => IRC_OSCILLATOR_FREQUENCY,
        (mainclksel::SEL_A::PLL_INPUT, Some((syspllclksel::SEL_A::CRYSTAL_OSCILLATOR, ..)), _) => unwrap(sys_osc_freq),
        (mainclksel::SEL_A::PLL_OUTPUT, Some((syspllclksel::SEL_A::IRC, m, _)), _) => Hertz(IRC_OSCILLATOR_FREQUENCY.0 * (m as u32 + 1)),
        (mainclksel::SEL_A::PLL_OUTPUT, Some((syspllclksel::SEL_A::CRYSTAL_OSCILLATOR,  m, _)), _) => Hertz(unwrap(sys_osc_freq).0 * (m as u32 + 1)),
        (mainclksel::SEL_A::WATCHDOG_OSCILLATOR, _, Some((freq, div))) => Hertz(wdtosc_freq_to_hertz(freq).0 / (2 * (1 + div as u32))),
        (mainclksel::SEL_A::PLL_INPUT, None, _) => minipanic!("Main clock configured with PLL Input, but PLL unconfigured."),
        (mainclksel::SEL_A::PLL_OUTPUT, None, _) => minipanic!("Main clock configured with PLL Output, but PLL unconfigured."),
        (mainclksel::SEL_A::WATCHDOG_OSCILLATOR, _, None) => minipanic!("Main clock configured with Watchdog Oscillator, but Watchdog Oscillator unconfigured."),
    }
}

const fn calculate_m_p(freq: Hertz, sys_osc: Option<Hertz>) -> (u8, u8) {
    let input_freq = if let Some(sys_osc) = sys_osc {
        sys_osc
    } else {
        IRC_OSCILLATOR_FREQUENCY
    };

    miniassert!(freq.0 % input_freq.0 == 0, "PLL target frequency is not a multiple of input frequency");
    let m = (freq.0 / input_freq.0) - 1;
    miniassert!(m < 32, "Target PLL frequency too high!");
    let m = m as u8;
    // TODO: re-roll loop when
    let p_val1 = 2 * 1 * freq.0;
    let p_val2 = 2 * 2 * freq.0;
    let p_val4 = 2 * 4 * freq.0;
    let p_val8 = 2 * 8 * freq.0;
    let p = if 156_000_000 <= p_val1 && p_val1 < 320_000_000 {
        0
    } else if 156_000_000 <= p_val2 && p_val2 < 320_000_000 {
        1
    } else if 156_000_000 <= p_val4 && p_val4 < 320_000_000 {
        2
    } else if 156_000_000 <= p_val8 && p_val8 < 320_000_000 {
        3
    } else {
        minipanic!("Expecting frequencty to allow selecting a good value for p")
    };
    (m, p)
}

/// A simple builder for the various Clocks of the LPC11uxx. In order to catch
/// clock misconfigurations at compile time, it provides a `validate` method
/// that will turn the `ClocksBuilder` into a `ClocksDescriptor`, that will
/// panic if a clock is misconfigured. This will cause a compile-time error.
///
/// # Usage
///
/// ```
/// const CLOCKS: ClockDescriptor = ClocksBuilder::new(None)
///     .main_clock(mainclksel::SEL_A::IRC)
///     .system_clock(Hertz(12_000_000))
///     .validate();
/// #/* Don't run the CLOCKS.build() in the doctests.
/// CLOCKS.build();
/// #*/
/// ```
pub struct ClocksBuilder {
    main_clock_source: mainclksel::SEL_A,
    system_pll: Option<(syspllclksel::SEL_A, Hertz)>,
    // Defaults to main_clock frequency
    system_clock_freq: Option<Hertz>,
    system_osc_freq: Option<Hertz>,
    watchdog_osc_freq: Option<Hertz>,
}

impl ClocksBuilder {
    pub const fn new(system_osc_freq: Option<Hertz>) -> ClocksBuilder {
        ClocksBuilder {
            main_clock_source: mainclksel::SEL_A::IRC_OSCILLATOR,
            system_pll: None,
            system_clock_freq: None,
            system_osc_freq,
            watchdog_osc_freq: None,
        }
    }

    pub const fn system_pll(mut self, pll_input: lpc11uxx::syscon::syspllclksel::SEL_A, freq: Hertz) -> Self {
        self.system_pll = Some((pll_input, freq));
        self
    }

    pub const fn system_clock(mut self, freq: Hertz) -> Self {
        self.system_clock_freq = Some(freq);
        self
    }

    pub const fn watchdog_oscillator(mut self, freq: Hertz) -> Self {
        self.watchdog_osc_freq = Some(freq);
        self
    }

    pub const fn main_clock(mut self, main_clock_source: lpc11uxx::syscon::mainclksel::SEL_A) -> Self {
        self.main_clock_source = main_clock_source;
        self
    }

    pub const fn validate(self) -> ClocksDescriptor {
        let system_pll_m_p = if let Some((input_clock, freq)) = self.system_pll {
            let sys_osc = if let syspllclksel::SEL_A::CRYSTAL_OSCILLATOR = input_clock {
                match self.system_osc_freq {
                    Some(v) => Some(v),
                    None => minipanic!("system_pll configured with external oscillator, but its frequency was not provided!")
                }
            } else {
                None
            };
            let (m, p) = calculate_m_p(freq, sys_osc);
            Some((input_clock, m, p))
        } else {
            None
        };

        let watchdog_osc = if let Some(watchdog_freq) = self.watchdog_osc_freq {
            // TODO: Use divisor to get the closest possible to the target freq
            if watchdog_freq.0 < 600_000 {
                Some((wdtoscctrl::FREQSEL_A::_0_6_MHZ, 1))
            } else if watchdog_freq.0 < 1_050_000 {
                Some((wdtoscctrl::FREQSEL_A::_1_05_MHZ, 1))
            } else if watchdog_freq.0 < 1_400_000 {
                Some((wdtoscctrl::FREQSEL_A::_1_4_MHZ, 1))
            } else if watchdog_freq.0 < 1_750_000 {
                Some((wdtoscctrl::FREQSEL_A::_1_75_MHZ, 1))
            } else if watchdog_freq.0 < 2_100_000 {
                Some((wdtoscctrl::FREQSEL_A::_2_1_MHZ, 1))
            } else if watchdog_freq.0 < 2_400_000 {
                Some((wdtoscctrl::FREQSEL_A::_2_4_MHZ, 1))
            } else if watchdog_freq.0 < 2_700_000 {
                Some((wdtoscctrl::FREQSEL_A::_2_7_MHZ, 1))
            } else if watchdog_freq.0 < 3_000_000 {
                Some((wdtoscctrl::FREQSEL_A::_3_0_MHZ, 1))
            } else if watchdog_freq.0 < 3_250_000 {
                Some((wdtoscctrl::FREQSEL_A::_3_25_MHZ, 1))
            } else if watchdog_freq.0 < 3_500_000 {
                Some((wdtoscctrl::FREQSEL_A::_3_5_MHZ, 1))
            } else if watchdog_freq.0 < 3_750_000 {
                Some((wdtoscctrl::FREQSEL_A::_3_75_MHZ, 1))
            } else if watchdog_freq.0 < 4_000_000 {
                Some((wdtoscctrl::FREQSEL_A::_4_0_MHZ, 1))
            } else if watchdog_freq.0 < 4_200_000 {
                Some((wdtoscctrl::FREQSEL_A::_4_2_MHZ, 1))
            } else if watchdog_freq.0 < 4_400_000 {
                Some((wdtoscctrl::FREQSEL_A::_4_4_MHZ, 1))
            } else if watchdog_freq.0 < 4_600_000 {
                Some((wdtoscctrl::FREQSEL_A::_4_6_MHZ, 1))
            } else {
                minipanic!("Watchdog frequency too high")
            }
        } else {
            None
        };


        let main_clock_freq = get_main_clock_freq(self.main_clock_source, system_pll_m_p, watchdog_osc, self.system_osc_freq);
        let system_clock_div = if let Some(freq) = self.system_clock_freq {
            miniassert!(main_clock_freq.0 % freq.0 == 0, "Main clock frequency is not a multiple of system clock frequency");
            (main_clock_freq.0 / freq.0) as u8
        } else {
            1
        };

        let system_clock_freq = main_clock_freq.0 / system_clock_div as u32;
        let flashtim = if system_clock_freq < 20_000_000 {
            FLASHTIM_A::_1_SYSTEM_CLOCK_FLASH
        } else if system_clock_freq < 40_000_000 {
            FLASHTIM_A::_2_SYSTEM_CLOCKS_FLAS
        } else if system_clock_freq < 50_000_000 {
            FLASHTIM_A::_3_SYSTEM_CLOCKS_FLAS
        } else {
            minipanic!("System frequency too high to configure flash controller timing")
        };

        ClocksDescriptor {
            main_clock_source: self.main_clock_source,
            system_osc_freq: self.system_osc_freq,
            system_pll_m_p,
            system_clock_div,
            watchdog_osc,
            flashtim
        }
    }
}

pub struct ClocksDescriptor {
    main_clock_source: mainclksel::SEL_A,
    system_osc_freq: Option<Hertz>,
    system_pll_m_p: Option<(syspllclksel::SEL_A, u8, u8)>,
    system_clock_div: u8,
    watchdog_osc: Option<(wdtoscctrl::FREQSEL_A, u8)>,
    flashtim: FLASHTIM_A,
}

impl ClocksDescriptor {
    // We rely on dead-code elimination to remove as many branches as possible.
    // If all goes well, the generated code should be really flat and just a
    // bunch of writes to the volatile registers.
    #[inline(always)]
    pub fn build(self, syscon: SYSCON_IMPL, fmc: &mut FLASHCTRL) -> (Clocks, PeriphClocks) {
        if let Some((pll_input, m, p)) = self.system_pll_m_p {
            match pll_input {
                // Ensure PLL input clock is powered up.
                syspllclksel::SEL_A::IRC => {
                    syscon.pdruncfg.modify(|_, v| v
                        .ircout_pd().powered()
                        .irc_pd().powered());
                },
                syspllclksel::SEL_A::CRYSTAL_OSCILLATOR => {
                    syscon.pdruncfg.modify(|_, v| v
                        .sysosc_pd().powered());

                    // 200us delay for OSC to be stabilized.
                    let i = &mut 0u32 as *mut u32;
                    loop {
                        unsafe {
                            // Safety: i is guaranteed to be valid.
                            let cur_i = i.read_volatile();
                            i.write_volatile(cur_i + 1);
                            if cur_i >= 0x100 {
                                break;
                            }
                        }
                    }
                }
            }

            // Set system PLL input clock
            syscon.syspllclksel.write(|v| v.sel().variant(pll_input));
            syscon.syspllclkuen.write(|v| v.ena().no_change());
            syscon.syspllclkuen.write(|v| v.ena().update_clock_source());

            // Power down PLL to change the PLL divider ratio
            syscon.pdruncfg.modify(|_, v| v.syspll_pd().powered_down());

            // Change divider ratio
            syscon.syspllctrl.write(|v| unsafe {
                v
                    .msel().bits(m)
                    .psel().bits(p)
            });

            // Power PLL back up
            syscon.pdruncfg.modify(|_, v| v.syspll_pd().powered());

            // Wait for PLL to lock
            while syscon.syspllstat.read().lock().is_pll_not_locked() {}
        }

        match self.main_clock_source {
            mainclksel::SEL_A::IRC_OSCILLATOR => {
                // Enable IRC Oscillator.
                syscon.pdruncfg.modify(|_, v| v
                    .ircout_pd().powered()
                    .irc_pd().powered());
            },
            mainclksel::SEL_A::WATCHDOG_OSCILLATOR => {
                // TODO: Enable Watchdog Oscillator
            }
            mainclksel::SEL_A::PLL_INPUT | mainclksel::SEL_A::PLL_OUTPUT => (),
        };

        syscon.sysahbclkdiv.write(|v| unsafe {
            v.div().bits(self.system_clock_div)
        });

        fmc.flashcfg.write(|v| v.flashtim().variant(self.flashtim));

        // Set main clock source.
        syscon.mainclksel.write(|v| v.sel().variant(self.main_clock_source));
        syscon.mainclkuen.write(|v| v.ena().no_change());
        syscon.mainclkuen.write(|v| v.ena().update_clock_source());

        // Enable IOCON (since it's necessary for GPIO configuration, and
        // basically *everything* needs GPIO...
        syscon.sysahbclkctrl.modify(|_, w| w.iocon().enabled());

        let main_clock_freq = get_main_clock_freq(self.main_clock_source, self.system_pll_m_p, self.watchdog_osc, self.system_osc_freq);
        let clocks = Clocks {
            system_osc_freq: self.system_osc_freq,
            main_clock_freq
        };

        let periph_clocks = PeriphClocks {
            usart: USARTClock { _private: () },
            usb: USBClock { _private: () },
            syscon: Syscon { _private: () },
        };
        (clocks, periph_clocks)
    }
}

/// Struct proving that the System Clocks are configured and frozen. Once this
/// struct exists, it is no longer possible to safely reconfigure or disable the
/// top-level clocks:
///
/// - IRC
/// - System Oscillator
/// - Watchdog Oscillator
/// - System PLL
/// - System Clock
/// - Main Clock
/// - IOCON Clock
#[derive(Debug, Clone, Copy)]
pub struct Clocks {
    system_osc_freq: Option<Hertz>,
    main_clock_freq: Hertz
}

impl Clocks {
    pub fn main_clock_freq(&self) -> Hertz {
        self.main_clock_freq
    }
}

pub struct USARTClock {
    _private: ()
}

pub struct PeriphClocks {
    //pub ssp0: SSP0Clock,
    //pub ssp1: SSP1Clock,
    pub usart: USARTClock,
    //pub i2c: I2CClock,
    pub usb: USBClock,
    pub syscon: Syscon
}

// API Design idea: Turn clocks into a `struct Clocks<const ClockDescription>`
// This would allow the `configure` function to take the main clock freq from
// the ClockDescription at compile-time, get the frequency at compile-time,
// validate that it's valid before actually doing the config.
//
// The big downside is that it causes a lot of API complexity since we now have
// const generics showing up in all the peripheral constructors.

impl USARTClock {
    #[inline]
    pub fn configure(&mut self, clocks: Clocks, freq: Hertz) {
        // TODO: Avoid runtime division for Peripheral Clock config.
        // BODY: We should be able to know, at compile time, the frequency of
        // BODY: the main clock, and the target frequency. So there's no reason
        // BODY: to do the divison at runtime...
        let syscon = SYSCON_IMPL::ptr();
        let div = (clocks.main_clock_freq().0 / freq.0) as u8;
        unsafe {
            cortex_m::interrupt::free(|_| {
                (*syscon).sysahbclkctrl.modify(|_, w| w.usart().enabled());
            });
            (*syscon).uartclkdiv.write(|v| v.div().bits(div));
        }
    }

    #[inline]
    pub fn disable(&mut self) {
        let syscon = SYSCON_IMPL::ptr();
        unsafe {
            cortex_m::interrupt::free(|_| {
                (*syscon).uartclkdiv.write(|w| w.div().bits(0));
                (*syscon).sysahbclkctrl.modify(|_, w| w.usart().disabled());
            });
        }
    }
}

pub struct USBClock {
    _private: ()
}

impl USBClock {
    pub fn configure(&mut self, clocks: Clocks, freq: Hertz) {
        let syscon = SYSCON_IMPL::ptr();
        let (m, p) = calculate_m_p(freq, clocks.system_osc_freq);

        let pll_input = if clocks.system_osc_freq.is_none() {
            usbpllclksel::SEL_A::IRC_THE_USB_PLL_CLO
        } else {
            usbpllclksel::SEL_A::SYSTEM_OSCILLATOR
        };

        // Set system PLL input clock
        unsafe {
            // Safety: Atomic writes to hardware registers owned by this virtual
            // structure.

            (*syscon).usbpllclksel.write(|v| v.sel().variant(pll_input));
            (*syscon).usbpllclkuen.write(|v| v.ena().no_change());
            (*syscon).usbpllclkuen.write(|v| v.ena().update_clock_source());

            // Change divider ratio
            (*syscon).usbpllctrl.write(|v| {
                v
                    .msel().bits(m)
                    .psel().bits(p)
            });

            // Power PLL up
            cortex_m::interrupt::free(|_| {
                (*syscon).pdruncfg.modify(|_, v| v.usbpll_pd().powered());
            });

            // Wait for PLL to lock
            while (*syscon).usbpllstat.read().lock().is_pll_not_locked() {}
        }
    }
}

pub struct Syscon {
    _private: ()
}

impl Syscon {
    pub fn enable_gpio(&self) {
        let syscon = SYSCON_IMPL::ptr();
        unsafe {
            cortex_m::interrupt::free(|_| {
                (*syscon).sysahbclkctrl.modify(|_, w| w.usart().disabled());
            });
        }
    }
    /*pub fn reset_status() -> u32 {

    }*/
}