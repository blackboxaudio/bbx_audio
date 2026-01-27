//! Board initialization abstraction.
//!
//! This module provides the [`Board`] struct which handles all hardware
//! initialization (power, clocks, GPIO ports) in a single call.

#[cfg(feature = "pod")]
use stm32h7xx_hal::{
    adc::{self, Adc},
    gpio::Analog,
    pac::{ADC1, DMA1, SAI1},
    rcc::rec,
};
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
use crate::{
    audio::Sai1Pins,
    clock::{ClockConfig, SampleRate},
    codec::{Codec, CodecError, Wm8731},
};

/// Board initialization error.
#[cfg(feature = "pod")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardError {
    /// Codec initialization failed.
    CodecInit(CodecError),
    /// Peripherals have already been taken.
    PeripheralsTaken,
}

/// ADC configuration for control inputs.
#[cfg(feature = "pod")]
#[derive(Debug, Clone, Copy)]
pub struct AdcConfig {
    /// ADC resolution (12-bit or 16-bit).
    pub resolution: adc::Resolution,
    /// ADC sample time (affects conversion speed vs accuracy trade-off).
    pub sample_time: adc::AdcSampleTime,
}

#[cfg(feature = "pod")]
impl AdcConfig {
    /// Default configuration for knobs: 12-bit resolution, T_64 sample time.
    ///
    /// 12-bit is sufficient for knobs and provides faster conversion than 16-bit.
    pub fn default_knobs() -> Self {
        Self {
            resolution: adc::Resolution::TwelveBit,
            sample_time: adc::AdcSampleTime::T_64,
        }
    }

    /// High-precision configuration: 16-bit resolution, T_810 sample time.
    ///
    /// Use for CV inputs or when maximum precision is needed.
    /// Slower conversion but more accurate.
    pub fn high_precision() -> Self {
        Self {
            resolution: adc::Resolution::SixteenBit,
            sample_time: adc::AdcSampleTime::T_810,
        }
    }
}

#[cfg(feature = "pod")]
impl Default for AdcConfig {
    fn default() -> Self {
        Self::default_knobs()
    }
}

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
    /// Configured sample rate (48kHz or 96kHz).
    pub sample_rate: SampleRate,
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
/// - Codec handle for runtime control (volume, gain, mute)
#[cfg(feature = "pod")]
pub struct AudioBoard<CODEC> {
    /// Audio codec handle (allows runtime volume/gain/mute control).
    pub codec: CODEC,
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
impl AudioBoard<Wm8731<stm32h7xx_hal::i2c::I2c<pac::I2C4>>> {
    /// Initialize the board for audio processing with Pod hardware.
    ///
    /// This configures:
    /// - 480 MHz system clock with PLL3 for SAI audio
    /// - All GPIO ports
    /// - SAI1 pins configured for I2S
    /// - WM8731 codec via I2C
    ///
    /// Returns the initialized board with codec handle for runtime control.
    ///
    /// # Errors
    ///
    /// Returns `BoardError::PeripheralsTaken` if peripherals have already been taken.
    /// Returns `BoardError::CodecInit` if codec initialization fails.
    pub fn init_pod() -> Result<Self, BoardError> {
        let dp = pac::Peripherals::take().ok_or(BoardError::PeripheralsTaken)?;
        let cp = cortex_m::Peripherals::take().ok_or(BoardError::PeripheralsTaken)?;

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
        codec.init(SampleRate::Rate48000).map_err(BoardError::CodecInit)?;

        // Get SAI1 with PLL3_P clock source (already configured in ClockConfig)
        let sai1_rec = ccdr.peripheral.SAI1;
        let dma1_rec = ccdr.peripheral.DMA1;

        let delay = cp.SYST.delay(ccdr.clocks);

        Ok(AudioBoard {
            codec,
            delay,
            gpioa,
            gpiob,
            gpioc,
            gpiod,
            gpiog,
            audio: AudioPeripherals {
                sample_rate: SampleRate::Rate48000,
                sai1: dp.SAI1,
                dma1: dp.DMA1,
                dma1_rec,
                sai1_pins,
                sai1_rec,
                clocks: ccdr.clocks,
            },
        })
    }

    /// Initialize the board for audio processing with Pod hardware (legacy API).
    ///
    /// This is a convenience wrapper around `init_pod()` that maintains backwards compatibility.
    /// Prefer using `init_pod()` for clarity.
    pub fn init() -> Result<Self, BoardError> {
        Self::init_pod()
    }

    /// Initialize with ADC configured for knob reading.
    ///
    /// This variant configures ADC1 for reading the two knobs on Pod hardware.
    /// The ADC configuration (resolution, sample time) can be customized via `adc_config`.
    ///
    /// # Arguments
    ///
    /// * `adc_config` - ADC configuration (resolution, sample time). Use `AdcConfig::default_knobs()` for standard knob
    ///   reading.
    ///
    /// # Errors
    ///
    /// Returns `BoardError::PeripheralsTaken` if peripherals have already been taken.
    /// Returns `BoardError::CodecInit` if codec initialization fails.
    pub fn init_with_adc_config(
        adc_config: AdcConfig,
    ) -> Result<AudioBoardWithAdc<Wm8731<stm32h7xx_hal::i2c::I2c<pac::I2C4>>>, BoardError> {
        let dp = pac::Peripherals::take().ok_or(BoardError::PeripheralsTaken)?;
        let cp = cortex_m::Peripherals::take().ok_or(BoardError::PeripheralsTaken)?;

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
        codec.init(SampleRate::Rate48000).map_err(BoardError::CodecInit)?;

        // Configure ADC pins (analog mode)
        let knob1_pin = gpioc.pc4.into_analog();
        let knob2_pin = gpioc.pc1.into_analog();

        // Configure ADC1 with user-specified settings
        let mut delay_local = cp.SYST.delay(ccdr.clocks);
        let mut adc1: Adc<ADC1, adc::Disabled> =
            Adc::adc1(dp.ADC1, 4.MHz(), &mut delay_local, ccdr.peripheral.ADC12, &ccdr.clocks);
        adc1.set_sample_time(adc_config.sample_time);
        adc1.set_resolution(adc_config.resolution);
        let adc1 = adc1.enable();

        // Get SAI1 with PLL3_P clock source
        let sai1_rec = ccdr.peripheral.SAI1;
        let dma1_rec = ccdr.peripheral.DMA1;

        Ok(AudioBoardWithAdc {
            codec,
            delay: delay_local,
            gpioa,
            gpiob,
            gpiod,
            gpiog,
            audio: AudioPeripherals {
                sample_rate: SampleRate::Rate48000,
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
        })
    }

    /// Initialize with ADC using default configuration (legacy API).
    ///
    /// This is a convenience wrapper that uses `AdcConfig::default_knobs()`.
    /// For custom ADC configuration, use `init_with_adc_config()`.
    pub fn init_with_adc() -> Result<AudioBoardWithAdc<Wm8731<stm32h7xx_hal::i2c::I2c<pac::I2C4>>>, BoardError> {
        Self::init_with_adc_config(AdcConfig::default_knobs())
    }
}

/// Board with ADC initialized for control input reading.
///
/// This struct is returned by [`AudioBoard::init_with_adc()`] and provides
/// access to both the standard board peripherals and ADC functionality.
#[cfg(feature = "pod")]
pub struct AudioBoardWithAdc<CODEC> {
    /// Audio codec handle (allows runtime volume/gain/mute control).
    pub codec: CODEC,
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
impl<CODEC> AudioBoardWithAdc<CODEC> {
    /// Read knob 1 value.
    ///
    /// Returns raw ADC value (0-4095 for 12-bit, 0-65535 for 16-bit depending on configuration).
    pub fn read_knob1(&mut self) -> u32 {
        self.adc1.read(&mut self.knob1_pin).unwrap_or(0)
    }

    /// Read knob 2 value.
    ///
    /// Returns raw ADC value (0-4095 for 12-bit, 0-65535 for 16-bit depending on configuration).
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
