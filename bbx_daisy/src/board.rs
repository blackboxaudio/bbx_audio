//! Board initialization abstraction.
//!
//! This module provides the [`Board`] struct which handles all hardware
//! initialization (power, clocks, GPIO ports) in a single call.

use stm32h7xx_hal::{
    delay::Delay,
    gpio::{
        gpioa::Parts as GpioA, gpiob::Parts as GpioB, gpioc::Parts as GpioC, gpiod::Parts as GpioD,
        gpioe::Parts as GpioE, gpiog::Parts as GpioG, gpioh::Parts as GpioH,
    },
    pac,
    prelude::*,
    rcc::CoreClocks,
};

#[cfg(feature = "pod")]
use stm32h7xx_hal::{
    adc::{self, Adc, AdcSampleTime},
    gpio::Analog,
    pac::{ADC1, DMA1, SAI1},
    rcc::rec,
};

#[cfg(feature = "pod")]
use crate::{
    audio::Sai1Pins,
    clock::{ClockConfig, SampleRate},
    codec::{Codec, Wm8731},
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
    /// GPIO Port H pins (includes I2C4 for codec control).
    pub gpioh: GpioH,
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
        let gpioh = dp.GPIOH.split(ccdr.peripheral.GPIOH);

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
            gpioh,
        }
    }
}

/// Audio board configuration with all peripherals needed for audio.
///
/// This struct holds the peripherals that need to be passed to
/// `audio::init_and_start()` to begin audio processing.
#[cfg(feature = "pod")]
pub struct AudioPeripherals {
    /// SAI1 peripheral for audio I/O.
    pub sai1: SAI1,
    /// DMA1 peripheral for audio DMA transfers.
    pub dma1: DMA1,
    /// DMA1 clock record.
    pub dma1_rec: rec::Dma1,
    /// Configured SAI1 pins.
    pub sai1_pins: Sai1Pins,
    /// SAI1 clock record (with PLL3_P configured).
    pub sai1_rec: rec::Sai1,
    /// Reference to system clocks.
    pub clocks: CoreClocks,
}

/// Board initialized for audio processing.
///
/// This is the result of [`AudioBoard::init()`] and contains:
/// - All GPIO ports for user access
/// - Audio peripherals ready to be started
/// - Delay timer
#[cfg(feature = "pod")]
pub struct AudioBoard {
    /// SysTick-based delay provider.
    pub delay: Delay,
    /// GPIO Port A pins.
    pub gpioa: GpioA,
    /// GPIO Port B pins.
    pub gpiob: GpioB,
    /// GPIO Port C pins (PC4=Knob1, PC1=Knob2, PC7=LED).
    pub gpioc: GpioC,
    /// GPIO Port D pins.
    pub gpiod: GpioD,
    /// GPIO Port G pins.
    pub gpiog: GpioG,
    /// Audio peripherals for starting audio.
    pub audio: AudioPeripherals,
}

#[cfg(feature = "pod")]
impl AudioBoard {
    /// Initialize the board for audio processing with Pod hardware.
    ///
    /// This configures:
    /// - 480 MHz system clock with PLL3 for SAI audio
    /// - All GPIO ports
    /// - SAI1 pins configured for I2S
    ///
    /// # Panics
    ///
    /// Panics if peripherals have already been taken.
    pub fn init() -> Self {
        let dp = pac::Peripherals::take().expect("device peripherals already taken");
        let cp = cortex_m::Peripherals::take().expect("core peripherals already taken");

        // Configure clocks with PLL3 for SAI audio
        let clock_config = ClockConfig::new(SampleRate::Rate48000);
        let ccdr = clock_config.configure(dp.PWR, dp.RCC, &dp.SYSCFG);

        // Enable I-cache for better performance
        let mut cp = cp;
        cp.SCB.enable_icache();

        // Split GPIO ports
        let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiog = dp.GPIOG.split(ccdr.peripheral.GPIOG);
        let gpioh = dp.GPIOH.split(ccdr.peripheral.GPIOH);

        // Configure SAI1 pins (all on GPIOE, AF6)
        let sai1_pins: Sai1Pins = (
            gpioe.pe2.into_alternate(),       // MCLK_A
            gpioe.pe5.into_alternate(),       // SCK_A
            gpioe.pe4.into_alternate(),       // FS_A
            gpioe.pe6.into_alternate(),       // SD_A (TX)
            Some(gpioe.pe3.into_alternate()), // SD_B (RX)
        );

        // Configure I2C4 for WM8731 codec control (PH11=SCL, PH12=SDA, AF4)
        let scl = gpioh.ph11.into_alternate().set_open_drain();
        let sda = gpioh.ph12.into_alternate().set_open_drain();
        let i2c4 = dp.I2C4.i2c((scl, sda), 400.kHz(), ccdr.peripheral.I2C4, &ccdr.clocks);

        // Initialize WM8731 codec
        let mut codec = Wm8731::with_default_address(i2c4);
        codec.init(SampleRate::Rate48000).expect("WM8731 codec init failed");

        // Get SAI1 with PLL3_P clock source (already configured in ClockConfig)
        let sai1_rec = ccdr.peripheral.SAI1;
        let dma1_rec = ccdr.peripheral.DMA1;

        let delay = cp.SYST.delay(ccdr.clocks);

        Self {
            delay,
            gpioa,
            gpiob,
            gpioc,
            gpiod,
            gpiog,
            audio: AudioPeripherals {
                sai1: dp.SAI1,
                dma1: dp.DMA1,
                dma1_rec,
                sai1_pins,
                sai1_rec,
                clocks: ccdr.clocks,
            },
        }
    }

    /// Initialize with ADC configured for knob reading.
    ///
    /// This variant configures ADC1 for reading the two knobs on Pod hardware.
    pub fn init_with_adc() -> AudioBoardWithAdc {
        let dp = pac::Peripherals::take().expect("device peripherals already taken");
        let cp = cortex_m::Peripherals::take().expect("core peripherals already taken");

        // Configure clocks with PLL3 for SAI audio
        let clock_config = ClockConfig::new(SampleRate::Rate48000);
        let ccdr = clock_config.configure(dp.PWR, dp.RCC, &dp.SYSCFG);

        // Enable I-cache for better performance
        let mut cp = cp;
        cp.SCB.enable_icache();

        // Split GPIO ports
        let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiog = dp.GPIOG.split(ccdr.peripheral.GPIOG);
        let gpioh = dp.GPIOH.split(ccdr.peripheral.GPIOH);

        // Configure SAI1 pins (all on GPIOE, AF6)
        let sai1_pins: Sai1Pins = (
            gpioe.pe2.into_alternate(),       // MCLK_A
            gpioe.pe5.into_alternate(),       // SCK_A
            gpioe.pe4.into_alternate(),       // FS_A
            gpioe.pe6.into_alternate(),       // SD_A (TX)
            Some(gpioe.pe3.into_alternate()), // SD_B (RX)
        );

        // Configure I2C4 for WM8731 codec control (PH11=SCL, PH12=SDA, AF4)
        let scl = gpioh.ph11.into_alternate().set_open_drain();
        let sda = gpioh.ph12.into_alternate().set_open_drain();
        let i2c4 = dp.I2C4.i2c((scl, sda), 400.kHz(), ccdr.peripheral.I2C4, &ccdr.clocks);

        // Initialize WM8731 codec
        let mut codec = Wm8731::with_default_address(i2c4);
        codec.init(SampleRate::Rate48000).expect("WM8731 codec init failed");

        // Configure ADC pins (analog mode)
        let knob1_pin = gpioc.pc4.into_analog();
        let knob2_pin = gpioc.pc1.into_analog();

        // Configure ADC1
        let mut delay_local = cp.SYST.delay(ccdr.clocks);
        let mut adc1: Adc<ADC1, adc::Disabled> =
            Adc::adc1(dp.ADC1, 4.MHz(), &mut delay_local, ccdr.peripheral.ADC12, &ccdr.clocks);
        adc1.set_sample_time(AdcSampleTime::T_64);
        adc1.set_resolution(adc::Resolution::SixteenBit);
        let adc1 = adc1.enable();

        // Get SAI1 with PLL3_P clock source
        let sai1_rec = ccdr.peripheral.SAI1;
        let dma1_rec = ccdr.peripheral.DMA1;

        AudioBoardWithAdc {
            delay: delay_local,
            gpioa,
            gpiob,
            gpiod,
            gpiog,
            audio: AudioPeripherals {
                sai1: dp.SAI1,
                dma1: dp.DMA1,
                dma1_rec,
                sai1_pins,
                sai1_rec,
                clocks: ccdr.clocks,
            },
            adc1,
            knob1_pin,
            knob2_pin,
        }
    }
}

/// Board with ADC initialized for control input reading.
///
/// This struct is returned by [`AudioBoard::init_with_adc()`] and provides
/// access to both the standard board peripherals and ADC functionality.
#[cfg(feature = "pod")]
pub struct AudioBoardWithAdc {
    /// SysTick-based delay provider.
    pub delay: Delay,
    /// GPIO Port A pins.
    pub gpioa: GpioA,
    /// GPIO Port B pins.
    pub gpiob: GpioB,
    /// GPIO Port D pins.
    pub gpiod: GpioD,
    /// GPIO Port G pins.
    pub gpiog: GpioG,
    /// Audio peripherals for starting audio.
    pub audio: AudioPeripherals,
    /// Configured ADC1 for knob reading.
    pub adc1: Adc<ADC1, adc::Enabled>,
    /// Knob 1 pin (PC4, analog).
    pub knob1_pin: stm32h7xx_hal::gpio::gpioc::PC4<Analog>,
    /// Knob 2 pin (PC1, analog).
    pub knob2_pin: stm32h7xx_hal::gpio::gpioc::PC1<Analog>,
}

#[cfg(feature = "pod")]
impl AudioBoardWithAdc {
    /// Read knob 1 value (0-65535 for 16-bit resolution).
    pub fn read_knob1(&mut self) -> u32 {
        self.adc1.read(&mut self.knob1_pin).unwrap_or(0)
    }

    /// Read knob 2 value (0-65535 for 16-bit resolution).
    pub fn read_knob2(&mut self) -> u32 {
        self.adc1.read(&mut self.knob2_pin).unwrap_or(0)
    }
}

/// Board with ADC for legacy API compatibility.
#[cfg(feature = "pod")]
pub struct BoardWithAdc {
    /// The initialized board with all peripherals.
    pub board: Board,
}
