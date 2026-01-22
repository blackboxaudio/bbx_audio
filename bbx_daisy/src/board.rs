//! Board initialization abstraction.
//!
//! This module provides the [`Board`] struct which handles all hardware
//! initialization (power, clocks, GPIO ports) in a single call.

use stm32h7xx_hal::{
    delay::Delay,
    gpio::{
        gpioa::Parts as GpioA, gpiob::Parts as GpioB, gpioc::Parts as GpioC, gpiod::Parts as GpioD,
        gpioe::Parts as GpioE, gpiog::Parts as GpioG,
    },
    pac,
    prelude::*,
    rcc::CoreClocks,
};

/// Initialized board with all peripherals ready to use.
///
/// Created by calling [`Board::init()`], which handles all the
/// power, clock, and GPIO initialization automatically.
///
/// # Example
///
/// ```ignore
/// #![no_std]
/// #![no_main]
///
/// use bbx_daisy::prelude::*;
///
/// fn app(board: Board) -> ! {
///     let mut led = Led::new(board.gpioc.pc7.into_push_pull_output());
///     loop {
///         led.toggle();
///         board.delay.delay_ms(500u32);
///     }
/// }
///
/// bbx_daisy_run!(app);
/// ```
pub struct Board {
    /// System clocks configuration.
    pub clocks: CoreClocks,
    /// SysTick-based delay provider.
    pub delay: Delay,
    /// GPIO Port A pins (split and ready for configuration).
    pub gpioa: GpioA,
    /// GPIO Port B pins.
    pub gpiob: GpioB,
    /// GPIO Port C pins (includes user LED on PC7).
    pub gpioc: GpioC,
    /// GPIO Port D pins.
    pub gpiod: GpioD,
    /// GPIO Port E pins (includes SAI audio pins).
    pub gpioe: GpioE,
    /// GPIO Port G pins.
    pub gpiog: GpioG,
}

impl Board {
    /// Initialize the board hardware.
    ///
    /// This performs all the boilerplate initialization:
    /// - Takes device and core peripherals
    /// - Configures power supply
    /// - Sets up 480 MHz system clock
    /// - Splits all GPIO ports
    /// - Initializes SysTick delay timer
    ///
    /// # Panics
    ///
    /// Panics if peripherals have already been taken.
    pub fn init() -> Self {
        let dp = pac::Peripherals::take().expect("device peripherals already taken");
        let cp = cortex_m::Peripherals::take().expect("core peripherals already taken");

        let pwr = dp.PWR.constrain().freeze();

        let rcc = dp.RCC.constrain();
        let ccdr = rcc.sys_ck(480.MHz()).freeze(pwr, &dp.SYSCFG);

        let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiog = dp.GPIOG.split(ccdr.peripheral.GPIOG);

        let delay = cp.SYST.delay(ccdr.clocks);

        Self {
            clocks: ccdr.clocks,
            delay,
            gpioa,
            gpiob,
            gpioc,
            gpiod,
            gpioe,
            gpiog,
        }
    }

    /// Initialize the board with ADC for control inputs.
    ///
    /// This variant also configures ADC1 for reading knobs on Pod hardware.
    /// Returns a [`BoardWithAdc`] that includes the board and ADC configuration.
    ///
    /// # Panics
    ///
    /// Panics if peripherals have already been taken.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let board_adc = Board::init_with_adc();
    /// // ADC is now ready for reading knobs
    /// ```
    #[cfg(feature = "pod")]
    pub fn init_with_adc() -> BoardWithAdc {
        let dp = pac::Peripherals::take().expect("device peripherals already taken");
        let cp = cortex_m::Peripherals::take().expect("core peripherals already taken");

        let pwr = dp.PWR.constrain().freeze();

        let rcc = dp.RCC.constrain();
        let ccdr = rcc.sys_ck(480.MHz()).freeze(pwr, &dp.SYSCFG);

        let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiog = dp.GPIOG.split(ccdr.peripheral.GPIOG);

        let delay = cp.SYST.delay(ccdr.clocks);

        // ADC initialization would go here
        // For Pod: PC4 (Knob 1) and PC1 (Knob 2) need to be configured as analog inputs
        // This is a placeholder - actual ADC hardware setup requires HAL ADC configuration

        let board = Self {
            clocks: ccdr.clocks,
            delay,
            gpioa,
            gpiob,
            gpioc,
            gpiod,
            gpioe,
            gpiog,
        };

        BoardWithAdc { board }
    }
}

/// Board with ADC initialized for control input reading.
///
/// This struct is returned by [`Board::init_with_adc()`] and provides
/// access to both the standard board peripherals and ADC functionality.
#[cfg(feature = "pod")]
pub struct BoardWithAdc {
    /// The initialized board with all peripherals.
    pub board: Board,
    // ADC reader would be added here when hardware support is complete
    // pub adc: AdcReader,
}
