#![no_std]
#![no_main]

use core::option::Option::Some;
use defmt::{panic, *};
use defmt_rtt as _; // global logger
use embassy_executor::Spawner;
use embassy_stm32::adc::*;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::i2c;
use embassy_stm32::i2c::I2c;
use embassy_stm32::spi::Spi;
use embassy_stm32::rcc::*;
use embassy_stm32::time::{khz, Hertz};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel as PWMChannel;
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{adc, bind_interrupts, peripherals, usb, Config};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};

use embassy_stm32::timer::OutputPolarity;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use futures::future::join3;
use panic_probe as _;

extern crate alloc;
extern crate alloc_cortex_m;

mod protobuf;
use protobuf::coms::{QControl, QRequest, QResponse, QState};
use quick_protobuf::{self, MessageWrite};

use alloc::borrow::Cow;

use alloc_cortex_m::CortexMHeap;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

bind_interrupts!(struct Irqs {
    USB => usb::InterruptHandler<peripherals::USB>;
    ADC1_COMP => adc::InterruptHandler<ADC>;
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});
use embassy_stm32::peripherals::*;

struct LoadControl {
    sdn: i32,
    pwm: i32,
    dac0: i32,
    dac1: i32,
    dac2: i32,
    dac3: i32,
}

struct LoadState {
    ch0: i32,
    ch1: i32,
    ch2: i32,
    ch3: i32,
    cal: i32,
    v: i32,
    temp: i32,
    sdn: i32,
}

static LOAD_CONTROL: Channel<ThreadModeRawMutex, LoadControl, 1> = Channel::new();

static LOAD_STATE: Mutex<ThreadModeRawMutex, LoadState> = Mutex::new(LoadState {
    ch0: 0,
    ch1: 0,
    ch2: 0,
    ch3: 0,
    cal: 0,
    v: 0,
    temp: 0,
    sdn: 0,
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    // Initialize the allocator before using it
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024;
    unsafe { ALLOCATOR.init(start, size) }

    let mut config = Config::default();
    config.rcc.hsi48 = Some(Hsi48Config {
        sync_from_usb: true,
    }); // needed for USB
    config.rcc.mux = ClockSrc::PLL1_R;
    config.rcc.hsi = true;
    config.rcc.pll = Some(Pll {
        source: PllSource::HSI,
        div: PllDiv::DIV3,
        mul: PllMul::MUL6,
    });
    config.rcc.clk48_src = Clk48Src::HSI48;
    config.rcc.apb2_pre = APBPrescaler::DIV1;

    let mut p = embassy_stm32::init(config);

    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.max_packet_size_0 = 64;
    config.manufacturer = Some("microengineer");
    config.product = Some("E-Load");
    config.serial_number = Some("rev1");

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state_usb_ctrl = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    let mut class_usb_ctrl = CdcAcmClass::new(&mut builder, &mut state_usb_ctrl, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    let ch2 = PwmPin::new_ch2(p.PB3, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM2,
        None,
        Some(ch2),
        None,
        None,
        khz(10),
        Default::default(),
    );
    pwm.set_polarity(PWMChannel::Ch2, OutputPolarity::ActiveHigh);
    pwm.set_duty(PWMChannel::Ch2, pwm.get_max_duty());
    pwm.enable(PWMChannel::Ch2);

    let i2c = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        NoDma,
        NoDma,
        Hertz(100_000),
        Default::default(),
    );

    let mut spi_config = embassy_stm32::spi::Config::default();
    spi_config.mode = embassy_stm32::spi::MODE_2;
    spi_config.frequency = Hertz(100_000);

    let dac_spi = Spi::new(p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA1_CH3, p.DMA1_CH2, spi_config);

    let dac_cs0 = Output::new(p.PA4, Level::High, Speed::VeryHigh);
    let dac_cs1 = Output::new(p.PA9, Level::High, Speed::VeryHigh);
    let dac_cs2 = Output::new(p.PA10, Level::High, Speed::VeryHigh);
    let dac_cs3 = Output::new(p.PA15, Level::High, Speed::VeryHigh);

    let led1 = Output::new(p.PB0, Level::High, Speed::Low);
    //let led2 = Output::new(p.PB1, Level::High, Speed::Low);

    let eload_sdn = Output::new(p.PB4, Level::High, Speed::Low);

    let mut adc = Adc::new(p.ADC, Irqs, &mut Delay);
    adc.set_sample_time(SampleTime::Cycles160_5);

    let mut vrefint = adc.enable_vref(&mut Delay);

    let adc_fut = async {
        loop {
            let mut samples = [0u16; 6];
            for i in 0..6 {
                let sample = match i {
                    0 => adc.read(&mut p.PA0).await,
                    1 => adc.read(&mut p.PA1).await,
                    2 => adc.read(&mut p.PA2).await,
                    3 => adc.read(&mut p.PA3).await,
                    4 => adc.read(&mut vrefint).await,
                    5 => adc.read(&mut p.PB1).await,
                    _ => 0,
                };
                samples[i] = sample;
            }
            let mut state = LOAD_STATE.lock().await;
            state.ch0 = samples[0] as i32;
            state.ch1 = samples[1] as i32;
            state.ch2 = samples[2] as i32;
            state.ch3 = samples[3] as i32;
            state.cal = samples[4] as i32;
            state.v = samples[5] as i32;
            drop(state);
        }
    };

    unwrap!(spawner.spawn(load_control_channel(led1, eload_sdn, pwm, dac_spi, dac_cs0, dac_cs1, dac_cs2, dac_cs3)));
    unwrap!(spawner.spawn(temp_monitoring_task(i2c)));

    let protobuf_rpc_fut = async {
        loop {
            class_usb_ctrl.wait_connection().await;
            info!("Connected");
            let _ = json_rpc(&mut class_usb_ctrl).await;
            info!("Disconnected");
        }
    };

    let _ = join3(usb_fut, protobuf_rpc_fut, adc_fut).await;
}

#[embassy_executor::task]
async fn temp_monitoring_task(mut i2c: I2c<'static, I2C1>) {
    loop {
        let mut data = [0u8; 2];
        if let Err(e) = i2c.blocking_read(0x48, &mut data) {
            error!("i2c error: {:?}", e);
            continue;
        }

        let mut temp_data = ((data[0] as u16) << 4) | ((data[1] as u16) >> 4);

        if temp_data > 2047 {
            temp_data -= 4096
        }

        info!("read temp: {}", temp_data);

        let mut status = LOAD_STATE.lock().await;
        status.temp = temp_data as i32;
        drop(status);

        Timer::after_millis(5000).await;
    }
}

#[embassy_executor::task]
async fn load_control_channel(
    mut led1: Output<'static>,
    mut sdn: Output<'static>,
    mut pwm: SimplePwm<'static, TIM2>,
    mut dac: Spi<'static, SPI1, DMA1_CH3, DMA1_CH2>,
    mut cs0: Output<'static>,
    mut cs1: Output<'static>,
    mut cs2: Output<'static>,
    mut cs3: Output<'static>,
) {
    let mut buf = [0u8; 3];
    loop {
        let control = LOAD_CONTROL.receive().await;

        match control.sdn {
            0 => {
                sdn.set_low();
                led1.set_low();
            }
            _ => {
                sdn.set_high();
                led1.set_high();
            }
        };

        // set DAC
        let cs = [&mut cs0, &mut cs1, &mut cs2, &mut cs3];
        let dac_val = [control.dac0, control.dac1, control.dac2, control.dac3];

        for i in 0..4 {
            cs[i].set_low();
            let data = (dac_val[i] & 0xffff) << 6;

            buf[0] = ((data & 0x00ff0000) >> 16) as u8;
            buf[1] = ((data & 0x0000ff00) >> 8) as u8;
            buf[2] =  (data & 0x000000ff) as u8;

            let _ = dac.write(&mut buf).await;

            cs[i].set_high();
        }


        // set PWM
        pwm.set_duty(
            PWMChannel::Ch2,
            (pwm.get_max_duty() as u32 * control.pwm as u32 / 100) as u16,
        );
    }
}
struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

enum Errors {
    None = 0,
    InvalidCommand = 1,
    ErrorDeserializingRequest = 2,
    ErrorSerializingResponse = 3,
    ErrorDeserializingRequestData = 4,
    ErrorSerializingResponseData = 5,
}

impl Errors {
    fn to_string(error: &Errors) -> &'static str {
        match error {
            Errors::InvalidCommand => "invalid command",
            Errors::ErrorDeserializingRequest => "error deserializing request",
            Errors::ErrorSerializingResponse => "error serializing response",
            Errors::ErrorDeserializingRequestData => "error deserializing request data",
            Errors::ErrorSerializingResponseData => "error serializing response data",
            _ => "unknown error",
        }
    }
}
enum Commands {
    NOP = 0,
    Control = 1,
    Status = 2,
}

impl Commands {
    fn from_i32(value: i32) -> Option<Commands> {
        match value {
            0 => Some(Commands::NOP),
            1 => Some(Commands::Control),
            2 => Some(Commands::Status),
            _ => None,
        }
    }
}

impl QResponse<'_> {
    fn default() -> QResponse<'static> {
        QResponse {
            id: 0,
            error: 0,
            data: Cow::Borrowed(&[0u8]),
        }
    }
}

// The response_bytes should be a mutable slice of u8, not a slice of a mutable slice.
async fn process_request<'a>(
    request: &QRequest<'_>,
    response: &mut QResponse<'_>,
) -> Result<usize, Errors> {
    let mut response_data = [0u8; 32];
    let mut response_len = 0;
    let error = Errors::None as i32;

    let op = Commands::from_i32(request.op);
    if op.is_none() {
        return Err(Errors::InvalidCommand);
    }

    match op.unwrap() {
        Commands::NOP => {
            // nop
        }
        Commands::Control => {
            let cmd: QControl = quick_protobuf::deserialize_from_slice(&request.data)
                .map_err(|_| Errors::ErrorDeserializingRequestData)?;

            info!(
                "receiving ctrl sdn: {}, pwm: {}, dac0: {}, dac1: {}, dac2: {}, dac3: {}",
                cmd.sdn, cmd.pwm, cmd.dac0, cmd.dac1, cmd.dac2, cmd.dac3
            );

            let control = LoadControl {
                sdn: cmd.sdn,
                pwm: cmd.pwm,
                dac0: cmd.dac0,
                dac1: cmd.dac1,
                dac2: cmd.dac2,
                dac3: cmd.dac3,
            };
            LOAD_CONTROL.send(control).await;
        }
        Commands::Status => {
            let state = LOAD_STATE.lock().await;
            let qstate = QState {
                ch0: state.ch0,
                ch1: state.ch1,
                ch2: state.ch2,
                ch3: state.ch3,
                cal: state.cal,
                v: state.v,
                temp: state.temp,
                sdn: state.sdn,
            };
            drop(state);

            info!("sending state - ch0: {}, ch1: {}, ch2: {}, ch3: {}, cal: {}, v: {}, temp: {}, sdn: {}", qstate.ch0, qstate.ch1, qstate.ch2, qstate.ch3, qstate.cal, qstate.v, qstate.temp, qstate.sdn);

            response_len = qstate.get_size() + 1 /* varint */;
            quick_protobuf::serialize_into_slice(&qstate, &mut response_data[..])
                .map_err(|_| Errors::ErrorSerializingResponseData)?;
        }
    };

    response.id = request.id;
    response.error = error;
    response.data = Cow::Owned(response_data[..response_len].to_vec());
    debug!(
        "response.id: {}, response.error:{}, response.data: {:?}",
        response.id,
        response.error,
        response_data[..response_len]
    );
    Ok(response_len)
}

async fn json_rpc<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
) -> Result<(), Disconnected> {
    let mut request_bytes = [0u8; 64];
    let mut response_bytes = [0u8; 64];

    loop {
        let n = class.read_packet(&mut request_bytes).await?;

        let mut response = QResponse::default();

        let request = match quick_protobuf::deserialize_from_slice(&request_bytes[..n]) {
            Ok(req) => Some(req),
            Err(_) => {
                error!("{}", Errors::to_string(&Errors::ErrorDeserializingRequest));
                response = QResponse::default();
                response.error = Errors::ErrorDeserializingRequest as i32;
                None
            }
        };

        // if request is some then we can process the request
        if request.is_some() {
            if let Err(e) = process_request(&request.unwrap(), &mut response).await {
                error!("{}", Errors::to_string(&e));
                response = QResponse::default();
                response.error = e as i32;
            }
        }

        let serialized_len = response.get_size() + 1 /* varint */;
        if let Err(_) = quick_protobuf::serialize_into_slice(&response, &mut response_bytes) {
            error!("{}", Errors::to_string(&Errors::ErrorSerializingResponse));
            continue;
        }

        class
            .write_packet(&response_bytes[..serialized_len])
            .await?;
    }
}
