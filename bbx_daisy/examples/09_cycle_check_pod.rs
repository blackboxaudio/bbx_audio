//! # 09_cycle_check_pod - On-device DSP cost measurement (Daisy Pod)
//!
//! Pod variant of `08_cycle_check`: runs the **real `bbx_dsp` blocks**
//! (polyblep sawtooth `OscillatorBlock` into a `LowPassFilterBlock`) inside the
//! audio ISR and measures the worst-case cycles per block with the DWT cycle
//! counter. Differences from the Seed variant: the WM8731 codec is configured
//! over I2C2, and the SAI transmits on Channel B (both selected by the `pod`
//! feature).
//!
//! The verdict reads out on the **Seed's onboard LED** (the seated Seed's PC7,
//! not the Pod's RGB LEDs):
//!
//! - **LED solid**       → max load under 50% of the audio budget. Comfortable.
//! - **LED slow blink**  → 50-90% of budget. Works, little headroom left.
//! - **LED fast blink**  → over 90% of budget. On the edge — expect underruns if anything else is added.
//!
//! The budget is `BLOCK_SIZE × 10_000` cycles (480 MHz core / 48 kHz sample
//! rate). You should also *hear* a 220 Hz filtered sawtooth on the Pod's audio
//! output — if it sounds clean, the polyblep pipeline survived the trip to
//! hardware.
//!
//! ## Building & Flashing
//!
//! ```bash
//! cd bbx_daisy
//! # Put the Daisy in DFU mode (hold BOOT, tap RESET, release BOOT), then:
//! cargo run --example 09_cycle_check_pod --features pod --release
//! ```

#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std, no_main)]

// See 01_blink.rs: ARM-only firmware with a host stub so the workspace still builds/tests
// on non-embedded targets.
#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod app {
    use core::{
        mem::MaybeUninit,
        sync::atomic::{AtomicU32, Ordering},
    };

    use bbx_daisy::{
        __internal::panic_halt as _,
        audio::{self, BLOCK_SIZE},
        buffer::FrameBuffer,
        clock::{ClockConfig, SampleRate},
        codec::{Codec, Wm8731},
        dsp::{
            ChannelLayout,
            block::Block,
            blocks::{LowPassFilterBlock, OscillatorBlock},
            context::DspContext,
            parameter::ModulationValues,
            waveform::Waveform,
        },
        peripherals::Led,
        prelude::*,
    };
    use cortex_m::peripheral::DWT;
    use stm32h7xx_hal::pac;

    /// Cycles available per audio block: 480 MHz / 48 kHz = 10_000 per sample.
    const BUDGET: u32 = (BLOCK_SIZE as u32) * 10_000;

    struct CycleCheckDsp {
        oscillator: OscillatorBlock<f32>,
        filter: LowPassFilterBlock<f32>,
        osc_buffer: [f32; BLOCK_SIZE],
        out_buffer: [f32; BLOCK_SIZE],
        context: DspContext,
    }

    impl CycleCheckDsp {
        fn new() -> Self {
            Self {
                oscillator: OscillatorBlock::new(220.0, Waveform::Sawtooth, None),
                filter: LowPassFilterBlock::new(2_000.0, 0.707),
                osc_buffer: [0.0; BLOCK_SIZE],
                out_buffer: [0.0; BLOCK_SIZE],
                context: DspContext {
                    sample_rate: audio::DEFAULT_SAMPLE_RATE as f64,
                    num_channels: 2,
                    buffer_size: BLOCK_SIZE,
                    current_sample: 0,
                    channel_layout: ChannelLayout::Stereo,
                },
            }
        }

        fn prepare(&mut self) {
            self.oscillator.prepare(&self.context);
            self.filter.prepare(&self.context);
        }

        fn process(&mut self, output: &mut FrameBuffer<BLOCK_SIZE>) {
            let no_inputs: [&[f32]; 0] = [];
            {
                let mut outputs: [&mut [f32]; 1] = [&mut self.osc_buffer];
                self.oscillator
                    .process(&no_inputs, &mut outputs, &ModulationValues::empty(), &self.context);
            }
            {
                let inputs: [&[f32]; 1] = [&self.osc_buffer];
                let mut outputs: [&mut [f32]; 1] = [&mut self.out_buffer];
                self.filter
                    .process(&inputs, &mut outputs, &ModulationValues::empty(), &self.context);
            }
            for i in 0..BLOCK_SIZE {
                let sample = self.out_buffer[i] * 0.5;
                output.set_frame(i, sample, sample);
            }
        }
    }

    /// Worst-case cycles observed for one audio block.
    static MAX_CYCLES: AtomicU32 = AtomicU32::new(0);

    static mut DSP: MaybeUninit<CycleCheckDsp> = MaybeUninit::uninit();

    fn audio_callback(_input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
        let start = DWT::cycle_count();

        // Safety: written once in main before the callback is registered;
        // only this ISR touches it afterwards (same discipline as the macros).
        let dsp = unsafe { (*core::ptr::addr_of_mut!(DSP)).assume_init_mut() };
        dsp.process(output);

        let elapsed = DWT::cycle_count().wrapping_sub(start);
        MAX_CYCLES.fetch_max(elapsed, Ordering::Relaxed);
    }

    #[bbx_daisy::__internal::entry]
    fn main() -> ! {
        // Hand-rolled init (mirrors board::init_audio for the Pod) because this
        // diagnostic needs the LED *and* the audio path, and init_audio consumes
        // the GPIO banks.
        let dp = pac::Peripherals::take().unwrap();
        let mut cp = cortex_m::Peripherals::take().unwrap();

        let sample_rate = SampleRate::Rate48000;
        let ccdr = ClockConfig::new(sample_rate).configure(dp.PWR, dp.RCC, &dp.SYSCFG);

        // Cycle counter for the ISR measurement.
        cp.DCB.enable_trace();
        cp.DWT.enable_cycle_counter();

        // SAI1 pins (PE2/PE5/PE4/PE6/PE3, AF6 — identical on all boards).
        let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
        let sai1_pins: audio::Sai1Pins = (
            gpioe.pe2.into_alternate(),
            gpioe.pe5.into_alternate(),
            gpioe.pe4.into_alternate(),
            gpioe.pe6.into_alternate(),
            Some(gpioe.pe3.into_alternate()),
        );

        // WM8731 (Pod / Seed 1.1): configure over I2C2 (SCL=PH4, SDA=PB11, AF4).
        let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioh = dp.GPIOH.split(ccdr.peripheral.GPIOH);
        let scl = gpioh.ph4.into_alternate().set_open_drain();
        let sda = gpiob.pb11.into_alternate().set_open_drain();
        let i2c = dp.I2C2.i2c((scl, sda), 400.kHz(), ccdr.peripheral.I2C2, &ccdr.clocks);
        let mut codec = Wm8731::with_default_address(i2c);
        codec.init(sample_rate).expect("codec init failed");

        // Verdict readout on the seated Seed's onboard LED (PC7).
        let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
        let mut led = Led::new(gpioc.pc7.into_push_pull_output());

        // DSP state must exist before the callback can fire.
        unsafe {
            (*core::ptr::addr_of_mut!(DSP)).write(CycleCheckDsp::new());
            (*core::ptr::addr_of_mut!(DSP)).assume_init_mut().prepare();
        }

        audio::set_callback(audio_callback).expect("audio already running");

        let sai1_rec = ccdr
            .peripheral
            .SAI1
            .kernel_clk_mux(stm32h7xx_hal::rcc::rec::Sai1ClkSel::Pll3P);
        audio::init_and_start(
            sample_rate,
            dp.SAI1,
            dp.DMA1,
            ccdr.peripheral.DMA1,
            sai1_pins,
            sai1_rec,
            &ccdr.clocks,
        )
        .expect("Failed to start audio streaming");

        // LED verdict loop: solid = <50% budget, slow blink = 50-90%,
        // fast blink = >90%.
        const HALF: u32 = BUDGET / 2;
        const NINETY_PERCENT: u32 = BUDGET / 10 * 9;
        const MS: u32 = 480_000;
        loop {
            let max = MAX_CYCLES.load(Ordering::Relaxed);
            if max < HALF {
                led.on();
                cortex_m::asm::delay(100 * MS);
            } else if max < NINETY_PERCENT {
                led.toggle();
                cortex_m::asm::delay(500 * MS);
            } else {
                led.toggle();
                cortex_m::asm::delay(100 * MS);
            }
        }
    }
}
