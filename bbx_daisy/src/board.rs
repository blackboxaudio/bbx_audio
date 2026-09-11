//! Board initialization abstraction.
//!
//! This module provides the [`Board`] struct which handles all hardware
//! initialization (power, clocks, GPIO ports) in a single call.
//!
//! ## Singleton Pattern
//!
//! Board uses a singleton pattern to ensure hardware is only initialized once:
//!
//! ```ignore
//! // Safe: returns None if already taken
//! let board = Board::take().expect("board already taken");
//!
//! // Unsafe: bypasses singleton check
//! let board = unsafe { Board::steal() };
//! ```

#[cfg(any(feature = "pod", feature = "patch_sm"))]
use stm32h7xx_hal::{
    adc::{self, Adc},
    gpio::Analog,
    pac::ADC1,
};
#[cfg(feature = "patch_sm")]
use stm32h7xx_hal::{
    dac::{self, DacExt},
    gpio::{Input, Output, PushPull},
    pac::DAC,
};
use stm32h7xx_hal::{
    delay::Delay,
    gpio::{
        gpioa::Parts as GpioA, gpiob::Parts as GpioB, gpioc::Parts as GpioC, gpiod::Parts as GpioD,
        gpioe::Parts as GpioE, gpiof::Parts as GpioF, gpiog::Parts as GpioG, gpioh::Parts as GpioH,
        gpioi::Parts as GpioI,
    },
    pac::{self, DMA1, SAI1},
    prelude::*,
    rcc::{CoreClocks, rec},
};

#[cfg(feature = "seed")]
use crate::codec::Ak4556;
// The codec init trait is used by boards that drive a codec over a bus; the Seed 1.2
// PCM3060 is strapped in hardware, so it needs no driver call.
#[cfg(any(feature = "seed", feature = "seed_1_1", feature = "pod", feature = "patch_sm"))]
use crate::codec::Codec;
#[cfg(feature = "patch_sm")]
use crate::codec::Pcm3060;
#[cfg(any(feature = "seed_1_1", feature = "pod"))]
use crate::codec::Wm8731;
#[cfg(feature = "patch_sm")]
use crate::peripherals::{CvOut, GateOut};
use crate::{
    audio::Sai1Pins,
    clock::{ClockConfig, SampleRate},
    codec::CodecError,
};

// Singleton marker - prevents taking Board more than once
// Using no_mangle to prevent linking different versions
#[unsafe(no_mangle)]
static BBX_DAISY_BOARD: () = ();

/// Set to `true` when `take` was called to make `Board` a singleton.
static mut BOARD_TAKEN: bool = false;

/// Board initialization error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardError {
    /// Codec initialization failed.
    CodecInit(CodecError),
    /// Peripherals have already been taken.
    PeripheralsTaken,
}

/// ADC configuration for control inputs.
#[cfg(any(feature = "pod", feature = "patch_sm"))]
#[derive(Debug, Clone, Copy)]
pub struct AdcConfig {
    /// ADC resolution (12-bit or 16-bit).
    pub resolution: adc::Resolution,
    /// ADC sample time (affects conversion speed vs accuracy trade-off).
    pub sample_time: adc::AdcSampleTime,
}

#[cfg(any(feature = "pod", feature = "patch_sm"))]
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

#[cfg(any(feature = "pod", feature = "patch_sm"))]
impl Default for AdcConfig {
    fn default() -> Self {
        Self::default_knobs()
    }
}

/// Initialized board with all peripherals ready to use.
///
/// Created by calling [`Board::take()`] or [`Board::init()`], which handles all the
/// power, clock, and GPIO initialization automatically.
///
/// ## Singleton Pattern
///
/// Board uses a singleton pattern to ensure hardware is only initialized once:
///
/// ```ignore
/// let board = Board::take().expect("board already taken");
/// ```
///
/// ## Accessing Peripherals
///
/// ```ignore
/// use bbx_daisy::prelude::*;
///
/// let board = Board::take().unwrap();
///
/// // Get the user LED
/// let mut led = UserLed::new(board.gpioc.pc7);
/// led.toggle();
/// ```
///
/// For flash and SDRAM, see the `flash` and `sdram` modules which provide
/// high-level initialization functions.
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
    /// GPIO Port D pins (includes SDRAM data pins).
    pub gpiod: GpioD,
    /// GPIO Port E pins (includes SAI audio pins, SDRAM data pins).
    pub gpioe: GpioE,
    /// GPIO Port F pins (includes QSPI flash pins, SDRAM address pins).
    pub gpiof: GpioF,
    /// GPIO Port G pins (includes QSPI CS, SDRAM control pins).
    pub gpiog: GpioG,
    /// GPIO Port H pins (includes I2C4 for codec control, SDRAM data pins).
    pub gpioh: GpioH,
    /// GPIO Port I pins (includes SDRAM data pins).
    pub gpioi: GpioI,
}

impl Board {
    /// Take the board singleton.
    ///
    /// Returns `Some(Board)` on first call, `None` on subsequent calls.
    /// This is the preferred way to initialize the board.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let board = Board::take().expect("board already taken");
    /// ```
    #[inline]
    pub fn take() -> Option<Self> {
        cortex_m::interrupt::free(|_| {
            if unsafe { BOARD_TAKEN } {
                None
            } else {
                Some(unsafe { Board::steal() })
            }
        })
    }

    /// Unsafely take the board, bypassing the singleton check.
    ///
    /// # Safety
    ///
    /// This bypasses the singleton pattern. The caller must ensure that
    /// the board is not initialized multiple times, which could cause
    /// undefined behavior.
    #[inline]
    pub unsafe fn steal() -> Self {
        unsafe { BOARD_TAKEN = true };
        Self::init_internal()
    }

    /// Initialize the board hardware (legacy API).
    ///
    /// This is equivalent to `Board::take().unwrap()` and is provided for
    /// backwards compatibility.
    ///
    /// # Panics
    ///
    /// Panics if the board has already been taken.
    #[inline]
    pub fn init() -> Self {
        Self::take().expect("board peripherals already taken")
    }

    /// Internal initialization - called by take() or steal().
    fn init_internal() -> Self {
        let dp = pac::Peripherals::take().expect("device peripherals already taken");
        let cp = cortex_m::Peripherals::take().expect("core peripherals already taken");

        // Configure power without VOS0 (use default VOS1 - high performance mode)
        let pwr = dp.PWR.constrain().freeze();

        let rcc = dp.RCC.constrain();
        // Use 400 MHz which is supported in VOS1 mode without VOS0
        let ccdr = rcc.sys_ck(400.MHz()).freeze(pwr, &dp.SYSCFG);

        let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiof = dp.GPIOF.split(ccdr.peripheral.GPIOF);
        let gpiog = dp.GPIOG.split(ccdr.peripheral.GPIOG);
        let gpioh = dp.GPIOH.split(ccdr.peripheral.GPIOH);
        let gpioi = dp.GPIOI.split(ccdr.peripheral.GPIOI);

        let delay = cp.SYST.delay(ccdr.clocks);

        Self {
            clocks: ccdr.clocks,
            delay,
            gpioa,
            gpiob,
            gpioc,
            gpiod,
            gpioe,
            gpiof,
            gpiog,
            gpioh,
            gpioi,
        }
    }
}

/// Audio board configuration with all peripherals needed for audio.
///
/// This struct holds the peripherals that need to be passed to
/// `audio::init_and_start()` to begin audio processing.
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

/// Configure the WM8731 codec over I2C2 (SCL=PH4, SDA=PB11, AF4) — used by Seed 1.1 / Pod.
#[cfg(any(feature = "seed_1_1", feature = "pod"))]
fn configure_wm8731_i2c2(
    gpiob: GpioB,
    gpioh: GpioH,
    i2c2: pac::I2C2,
    i2c2_rec: rec::I2c2,
    clocks: &CoreClocks,
    sample_rate: SampleRate,
) -> Result<(), BoardError> {
    let scl = gpioh.ph4.into_alternate().set_open_drain();
    let sda = gpiob.pb11.into_alternate().set_open_drain();
    let i2c = i2c2.i2c((scl, sda), 400.kHz(), i2c2_rec, clocks);
    let mut codec = Wm8731::with_default_address(i2c);
    codec.init(sample_rate).map_err(BoardError::CodecInit)?;
    Ok(())
}

/// Initialize the audio hardware for the selected board and return the peripherals needed
/// to start streaming via [`crate::audio::init_and_start`].
///
/// Configures the PLL3 SAI clock (12.288 MHz MCLK @ 48 kHz), the SAI1 pins
/// (PE2/PE5/PE4/PE6/PE3, AF6 — identical on all boards), and the board's codec. The codec
/// and the SAI direction are selected at compile time per board feature (stm32h7xx-hal types
/// the SAI channels), so the feature must match the Seed revision:
///
/// - `seed`: AK4556 (no I2C; reset pulse on PB11)
/// - `seed_1_1` / `pod`: WM8731 over I2C2 (SCL=PH4, SDA=PB11)
/// - `seed_1_2`: PCM3060 (strapped in hardware; PB11 held low for de-emphasis off)
/// - `patch_sm`: PCM3060 over I2C2 (SCL=PB10, SDA=PB11)
///
/// Codec/I2C/reset handles are dropped after configuration; the codec retains its state.
///
/// # Errors
///
/// Returns [`BoardError::PeripheralsTaken`] if peripherals were already taken, or
/// [`BoardError::CodecInit`] if codec configuration over I2C fails.
pub fn init_audio() -> Result<AudioPeripherals, BoardError> {
    let dp = pac::Peripherals::take().ok_or(BoardError::PeripheralsTaken)?;

    let sample_rate = SampleRate::Rate48000;
    let ccdr = ClockConfig::new(sample_rate).configure(dp.PWR, dp.RCC, &dp.SYSCFG);

    // SAI1 pins are identical on every audio board: PE2/PE5/PE4/PE6/PE3, AF6.
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let sai1_pins: Sai1Pins = (
        gpioe.pe2.into_alternate(),
        gpioe.pe5.into_alternate(),
        gpioe.pe4.into_alternate(),
        gpioe.pe6.into_alternate(),
        Some(gpioe.pe3.into_alternate()),
    );

    // AK4556 (original Seed): no I2C; release from power-down via PB11.
    #[cfg(feature = "seed")]
    {
        let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
        let mut codec_reset = gpiob.pb11.into_push_pull_output();
        codec_reset.set_low();
        cortex_m::asm::delay(480_000); // ~1 ms @ 480 MHz
        codec_reset.set_high();
        cortex_m::asm::delay(480_000);
        let mut codec = Ak4556::new();
        codec.init(sample_rate).map_err(BoardError::CodecInit)?;
    }

    // WM8731 (Seed 1.1 / Pod): configure over I2C2.
    #[cfg(any(feature = "seed_1_1", feature = "pod"))]
    configure_wm8731_i2c2(
        dp.GPIOB.split(ccdr.peripheral.GPIOB),
        dp.GPIOH.split(ccdr.peripheral.GPIOH),
        dp.I2C2,
        ccdr.peripheral.I2C2,
        &ccdr.clocks,
        sample_rate,
    )?;

    // PCM3060 (Seed 2 DFM): strapped in hardware (no I2C); PB11 low disables de-emphasis.
    #[cfg(feature = "seed_1_2")]
    {
        let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
        let mut deemphasis = gpiob.pb11.into_push_pull_output();
        deemphasis.set_low();
    }

    // Patch SM: PCM3060 over I2C2 (SCL=PB10, SDA=PB11, per libDaisy's
    // daisy_patch_sm — PH11/PH12 are SDRAM data lines on this module).
    #[cfg(feature = "patch_sm")]
    {
        let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
        let scl = gpiob.pb10.into_alternate().set_open_drain();
        let sda = gpiob.pb11.into_alternate().set_open_drain();
        let i2c2 = dp.I2C2.i2c((scl, sda), 400.kHz(), ccdr.peripheral.I2C2, &ccdr.clocks);
        let mut codec = Pcm3060::with_default_address(i2c2);
        codec.init(sample_rate).map_err(BoardError::CodecInit)?;
    }

    let sai1_rec = ccdr
        .peripheral
        .SAI1
        .kernel_clk_mux(stm32h7xx_hal::rcc::rec::Sai1ClkSel::Pll3P);
    let dma1_rec = ccdr.peripheral.DMA1;

    Ok(AudioPeripherals {
        sample_rate,
        sai1: dp.SAI1,
        dma1: dp.DMA1,
        dma1_rec,
        sai1_pins,
        sai1_rec,
        clocks: ccdr.clocks,
    })
}

/// Initialize the audio hardware **and** ADC1 for the Pod's two knobs.
///
/// The codec is auto-detected at runtime (see [`init_audio`]); ADC1 reads knob 1 on PC4
/// and knob 2 on PC0. Returns the audio peripherals plus the enabled ADC and knob pins.
///
/// # Errors
///
/// Returns [`BoardError::PeripheralsTaken`] if peripherals were already taken, or
/// [`BoardError::CodecInit`] if codec configuration fails.
#[cfg(feature = "pod")]
pub fn init_audio_with_adc() -> Result<AudioBoardWithAdc, BoardError> {
    let dp = pac::Peripherals::take().ok_or(BoardError::PeripheralsTaken)?;
    let cp = cortex_m::Peripherals::take().ok_or(BoardError::PeripheralsTaken)?;

    let sample_rate = SampleRate::Rate48000;
    let ccdr = ClockConfig::new(sample_rate).configure(dp.PWR, dp.RCC, &dp.SYSCFG);

    // SAI1 pins (PE2/PE5/PE4/PE6/PE3, AF6).
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let sai1_pins: Sai1Pins = (
        gpioe.pe2.into_alternate(),
        gpioe.pe5.into_alternate(),
        gpioe.pe4.into_alternate(),
        gpioe.pe6.into_alternate(),
        Some(gpioe.pe3.into_alternate()),
    );

    // WM8731 over I2C2 (Pod uses a Seed 1.1).
    configure_wm8731_i2c2(
        dp.GPIOB.split(ccdr.peripheral.GPIOB),
        dp.GPIOH.split(ccdr.peripheral.GPIOH),
        dp.I2C2,
        ccdr.peripheral.I2C2,
        &ccdr.clocks,
        sample_rate,
    )?;

    // ADC1 for the Pod knobs: knob 1 = PC4, knob 2 = PC0.
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
    let knob1_pin = gpioc.pc4.into_analog();
    let knob2_pin = gpioc.pc0.into_analog();
    let mut delay = cp.SYST.delay(ccdr.clocks);
    let adc_config = AdcConfig::default_knobs();
    let mut adc1: Adc<ADC1, adc::Disabled> =
        Adc::adc1(dp.ADC1, 4.MHz(), &mut delay, ccdr.peripheral.ADC12, &ccdr.clocks);
    adc1.set_sample_time(adc_config.sample_time);
    adc1.set_resolution(adc_config.resolution);
    let adc1 = adc1.enable();

    let sai1_rec = ccdr
        .peripheral
        .SAI1
        .kernel_clk_mux(stm32h7xx_hal::rcc::rec::Sai1ClkSel::Pll3P);
    let dma1_rec = ccdr.peripheral.DMA1;

    Ok(AudioBoardWithAdc {
        audio: AudioPeripherals {
            sample_rate,
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

/// Board with ADC initialized for control input reading.
///
/// Returned by [`init_audio_with_adc`]; provides the audio peripherals plus the enabled
/// ADC and knob pins.
#[cfg(feature = "pod")]
pub struct AudioBoardWithAdc {
    /// Audio peripherals for starting audio.
    pub audio: AudioPeripherals,
    /// Configured ADC1 for knob reading.
    pub adc1: Adc<ADC1, adc::Enabled>,
    /// Knob 1 pin (PC4, analog).
    pub knob1_pin: stm32h7xx_hal::gpio::gpioc::PC4<Analog>,
    /// Knob 2 pin (PC0, analog).
    pub knob2_pin: stm32h7xx_hal::gpio::gpioc::PC0<Analog>,
}

#[cfg(feature = "pod")]
impl AudioBoardWithAdc {
    /// Read knob 1 (raw ADC value).
    pub fn read_knob1(&mut self) -> u32 {
        self.adc1.read(&mut self.knob1_pin).unwrap_or(0)
    }

    /// Read knob 2 (raw ADC value).
    pub fn read_knob2(&mut self) -> u32 {
        self.adc1.read(&mut self.knob2_pin).unwrap_or(0)
    }
}

/// Initialize the audio hardware **and** the full Patch.Init() control surface.
///
/// Configures the PCM3060 codec over I2C2 (as in [`init_audio`]), then:
///
/// - **ADC1** for the four panel knobs (SM channels CV_1-4: PA3, PA6, PA2, PA7) and the four panel CV jacks (SM
///   channels CV_5-8: PC1, PC0, PB1, PC4)
/// - **B7** momentary button (PB8) and **B8** toggle (PB9), pull-up, active-low
/// - **Gate inputs** 1/2 (PG13/PG14, floating — the module's inverting input stage drives them; read active-low)
/// - **Gate outputs** 1/2 (PC14/PC13, push-pull), wrapped in [`GateOut`]
/// - **DAC1** both channels, buffer-calibrated and enabled: channel 1 (PA4) is the CV OUT jack, channel 2 (PA5) drives
///   the Patch.Init() front-panel LED
///
/// Everything must be claimed here in one shot: `pac::Peripherals::take()` is
/// single-use, so there is no adding peripherals after this returns.
///
/// # Errors
///
/// Returns [`BoardError::PeripheralsTaken`] if peripherals were already taken, or
/// [`BoardError::CodecInit`] if codec configuration fails.
#[cfg(feature = "patch_sm")]
pub fn init_audio_with_controls() -> Result<AudioBoardWithControls, BoardError> {
    let dp = pac::Peripherals::take().ok_or(BoardError::PeripheralsTaken)?;
    let cp = cortex_m::Peripherals::take().ok_or(BoardError::PeripheralsTaken)?;

    let sample_rate = SampleRate::Rate48000;
    let ccdr = ClockConfig::new(sample_rate).configure(dp.PWR, dp.RCC, &dp.SYSCFG);

    // SAI1 pins (PE2/PE5/PE4/PE6/PE3, AF6).
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let sai1_pins: Sai1Pins = (
        gpioe.pe2.into_alternate(),
        gpioe.pe5.into_alternate(),
        gpioe.pe4.into_alternate(),
        gpioe.pe6.into_alternate(),
        Some(gpioe.pe3.into_alternate()),
    );

    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
    let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
    let gpiog = dp.GPIOG.split(ccdr.peripheral.GPIOG);

    // PCM3060 codec over I2C2 (SCL=PB10, SDA=PB11, per libDaisy's
    // daisy_patch_sm — PH11/PH12 are SDRAM data lines on this module).
    let scl = gpiob.pb10.into_alternate().set_open_drain();
    let sda = gpiob.pb11.into_alternate().set_open_drain();
    let i2c2 = dp.I2C2.i2c((scl, sda), 400.kHz(), ccdr.peripheral.I2C2, &ccdr.clocks);
    let mut codec = Pcm3060::with_default_address(i2c2);
    codec.init(sample_rate).map_err(BoardError::CodecInit)?;

    // Panel knobs (SM channels CV_1-4, pin map per libDaisy's daisy_patch_sm.cpp).
    let knob1_pin = gpioa.pa3.into_analog();
    let knob2_pin = gpioa.pa6.into_analog();
    let knob3_pin = gpioa.pa2.into_analog();
    let knob4_pin = gpioa.pa7.into_analog();

    // Panel CV jacks (SM channels CV_5-8).
    let cv1_pin = gpioc.pc1.into_analog();
    let cv2_pin = gpioc.pc0.into_analog();
    let cv3_pin = gpiob.pb1.into_analog();
    let cv4_pin = gpioc.pc4.into_analog();

    // B7 momentary + B8 toggle (both active-low against the internal pull-up).
    let button_pin = gpiob.pb8.into_pull_up_input();
    let switch_pin = gpiob.pb9.into_pull_up_input();

    // Gate inputs: the module's transistor stage inverts and drives the pin,
    // so no pull is wanted; readers must treat these as active-low.
    let gate1_pin = gpiog.pg13.into_floating_input();
    let gate2_pin = gpiog.pg14.into_floating_input();

    // Gate outputs.
    let gate_out1 = GateOut::new(gpioc.pc14.into_push_pull_output());
    let gate_out2 = GateOut::new(gpioc.pc13.into_push_pull_output());

    // ADC1 (12-bit) shared by knobs and CV jacks.
    let mut delay = cp.SYST.delay(ccdr.clocks);
    let adc_config = AdcConfig::default_knobs();
    let mut adc1: Adc<ADC1, adc::Disabled> =
        Adc::adc1(dp.ADC1, 4.MHz(), &mut delay, ccdr.peripheral.ADC12, &ccdr.clocks);
    adc1.set_sample_time(adc_config.sample_time);
    adc1.set_resolution(adc_config.resolution);
    let adc1 = adc1.enable();

    // DAC1: channel 1 (PA4) = CV OUT jack, channel 2 (PA5) = panel LED.
    // Factory trim only: the HAL's calibrate_buffer() spins on a calibration
    // flag with no timeout, so a channel that never raises it would hang boot
    // before audio even starts. Factory trim is plenty for an LED and
    // gate-level CV; add calibration back only if CV-out precision demands it.
    let (cv_dac, led_dac) = dp.DAC.dac(
        (gpioa.pa4.into_analog(), gpioa.pa5.into_analog()),
        ccdr.peripheral.DAC12,
    );
    let cv_out = CvOut::new(cv_dac.enable());
    let led = CvOut::new(led_dac.enable());

    let sai1_rec = ccdr
        .peripheral
        .SAI1
        .kernel_clk_mux(stm32h7xx_hal::rcc::rec::Sai1ClkSel::Pll3P);
    let dma1_rec = ccdr.peripheral.DMA1;

    Ok(AudioBoardWithControls {
        audio: AudioPeripherals {
            sample_rate,
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
        knob3_pin,
        knob4_pin,
        cv1_pin,
        cv2_pin,
        cv3_pin,
        cv4_pin,
        button_pin,
        switch_pin,
        gate1_pin,
        gate2_pin,
        gate_out1,
        gate_out2,
        cv_out,
        led,
    })
}

/// Board with the full Patch.Init() control surface initialized.
///
/// Returned by [`init_audio_with_controls`]. Input pins are handed out raw so
/// the macro (or a custom main loop) chooses the wrapper semantics (debounce,
/// polarity); outputs come pre-wrapped and ready to drive.
#[cfg(feature = "patch_sm")]
pub struct AudioBoardWithControls {
    /// Audio peripherals for starting audio.
    pub audio: AudioPeripherals,
    /// Configured ADC1 shared by knobs and CV jacks.
    pub adc1: Adc<ADC1, adc::Enabled>,
    /// Panel knob 1 (SM CV_1, PA3, analog).
    pub knob1_pin: stm32h7xx_hal::gpio::gpioa::PA3<Analog>,
    /// Panel knob 2 (SM CV_2, PA6, analog).
    pub knob2_pin: stm32h7xx_hal::gpio::gpioa::PA6<Analog>,
    /// Panel knob 3 (SM CV_3, PA2, analog).
    pub knob3_pin: stm32h7xx_hal::gpio::gpioa::PA2<Analog>,
    /// Panel knob 4 (SM CV_4, PA7, analog).
    pub knob4_pin: stm32h7xx_hal::gpio::gpioa::PA7<Analog>,
    /// Panel CV jack 1 (SM CV_5, PC1, analog, bipolar/inverting).
    pub cv1_pin: stm32h7xx_hal::gpio::gpioc::PC1<Analog>,
    /// Panel CV jack 2 (SM CV_6, PC0, analog, bipolar/inverting).
    pub cv2_pin: stm32h7xx_hal::gpio::gpioc::PC0<Analog>,
    /// Panel CV jack 3 (SM CV_7, PB1, analog, bipolar/inverting).
    pub cv3_pin: stm32h7xx_hal::gpio::gpiob::PB1<Analog>,
    /// Panel CV jack 4 (SM CV_8, PC4, analog, bipolar/inverting).
    pub cv4_pin: stm32h7xx_hal::gpio::gpioc::PC4<Analog>,
    /// B7 momentary button pin (PB8, pull-up input, active-low).
    pub button_pin: stm32h7xx_hal::gpio::gpiob::PB8<Input>,
    /// B8 toggle pin (PB9, pull-up input, active-low).
    pub switch_pin: stm32h7xx_hal::gpio::gpiob::PB9<Input>,
    /// Gate input 1 pin (PG13, floating, active-low).
    pub gate1_pin: stm32h7xx_hal::gpio::gpiog::PG13<Input>,
    /// Gate input 2 pin (PG14, floating, active-low).
    pub gate2_pin: stm32h7xx_hal::gpio::gpiog::PG14<Input>,
    /// Gate output 1 (PC14).
    pub gate_out1: GateOut<stm32h7xx_hal::gpio::gpioc::PC14<Output<PushPull>>>,
    /// Gate output 2 (PC13).
    pub gate_out2: GateOut<stm32h7xx_hal::gpio::gpioc::PC13<Output<PushPull>>>,
    /// CV OUT jack (DAC1 channel 1, PA4), enabled and calibrated.
    pub cv_out: CvOut<dac::C1<DAC, dac::Enabled>>,
    /// Front-panel LED (DAC1 channel 2, PA5), enabled and calibrated.
    pub led: CvOut<dac::C2<DAC, dac::Enabled>>,
}
