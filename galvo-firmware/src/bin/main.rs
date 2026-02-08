#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use embassy_executor::Spawner;
use embassy_net::{DhcpConfig, StackResources};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{Ledc, LowSpeed, timer};
use esp_hal::otg_fs::{Usb, UsbBus};
use esp_hal::rng::Rng;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::Controller;
use galvo_driver::network::{
    RtcTimeSource, SharedRtc, connection, get_mastodon_status, get_time_ntp, net_task,
};
use galvo_driver::nunchuck::Nunchuck;
use galvo_driver::protocol::{Command, Response};
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use vector_apps::apps::clock::Clock;
use vector_apps::apps::{self, VectorApp};

use log::info;

use embassy_sync::blocking_mutex::Mutex;
use galvo_driver::lasers::Lasers;
use galvo_driver::led::IndicatorLed;
use vector_apps::apps::alphabet::AlphabetDemo;
use vector_apps::apps::asteroids::Asteroids;
use vector_apps::apps::cube::CubeDemo;
use vector_apps::apps::cycle::Cycle;
use vector_apps::apps::maps::Maps;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 139264);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    let delay = Delay::new();

    // Adafruit QtPy
    // let mut indicator = IndicatorLed::new(peripherals.GPIO38, peripherals.RMT, peripherals.GPIO39);
    // TinyS2
    let mut indicator = IndicatorLed::new(peripherals.GPIO2, peripherals.RMT, peripherals.GPIO1);
    indicator.set_color(smart_leds::colors::RED);

    let ledc = Ledc::new(peripherals.LEDC);
    let mut timer = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    timer
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty8Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(24),
        })
        .unwrap();

    let mut lasers = Lasers::new(
        peripherals.GPIO4,
        peripherals.GPIO5,
        peripherals.GPIO6,
        ledc,
        &timer,
        peripherals.DAC2,
        peripherals.GPIO18,
        peripherals.DAC1,
        peripherals.GPIO17,
    );

    let mut nunchuck = Nunchuck::new(peripherals.I2C0, peripherals.GPIO8, peripherals.GPIO9);

    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    let usb_bus = UsbBus::new(usb, unsafe { &mut *core::ptr::addr_of_mut!(EP_MEMORY) });

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x303A, 0x3001))
        .device_class(USB_CLASS_CDC)
        .build();

    let rtc = mk_static!(SharedRtc, Mutex::new(Rtc::new(peripherals.LPWR)));

    let esp_radio_ctrl = &*mk_static!(Controller<'static>, esp_radio::init().unwrap());

    let (controller, interfaces) =
        esp_radio::wifi::new(esp_radio_ctrl, peripherals.WIFI, Default::default()).unwrap();

    let wifi_interface = interfaces.sta;

    let mut dhcp_options: DhcpConfig = Default::default();
    dhcp_options.hostname = Some(heapless::String::try_from("laser-esp32").unwrap());
    let config = embassy_net::Config::dhcpv4(dhcp_options);

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<8>, StackResources::<8>::new()),
        seed,
    );

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(runner)).ok();

    stack.wait_config_up().await;

    indicator.set_color(smart_leds::colors::YELLOW);

    get_time_ntp(&stack, rtc).await;

    // let post = get_mastodon_status(&stack).await;

    let mut serial_buffer: [u8; 2048] = [0; 2048];
    let mut serial_rx_length: usize = 0;

    let mut apps: Vec<Box<dyn VectorApp>> = Vec::with_capacity(5);
    // apps.push(Box::new(AlphabetDemo::new(String::from("ABCDEFGH"))));
    // apps.push(Box::new(CubeDemo::new()));
    // apps.push(Box::new(Asteroids::new()));
    apps.push(Box::new(Maps::new()));
    // apps.push(Box::new(Clock::new(RtcTimeSource::new(rtc))));

    let mut active_demo: Box<dyn apps::VectorApp> = Box::new(Cycle::new(apps));
    // let mut active_demo: Box<dyn apps::VectorApp> = Box::new(Asteroids::new());

    let mut frameno: u64 = 0;

    indicator.set_color(smart_leds::colors::GREEN);

    loop {
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];

            if let Ok(count) = serial.read(&mut buf)
                && count > 0
            {
                // Append to RX buffer
                if serial_rx_length + count <= serial_buffer.len() {
                    serial_buffer[serial_rx_length..serial_rx_length + count]
                        .copy_from_slice(&buf[..count]);
                    serial_rx_length += count;
                }

                // Look for newline delimiter
                if let Some(pos) = serial_buffer[..serial_rx_length]
                    .iter()
                    .position(|&b| b == b'\n')
                {
                    let json_bytes = &serial_buffer[..pos];

                    if let Ok(s) = core::str::from_utf8(json_bytes)
                        && let Ok(cmd) = serde_json::from_str::<Command>(s)
                    {
                        match cmd {
                            Command::SetIndicatorLight { r, g, b } => {
                                indicator.set_color(smart_leds::RGB { r, g, b });
                            }
                        }

                        let result = Response { success: true };

                        let response = serde_json::to_string(&result).unwrap();
                        let _ = serial.write(response.as_bytes());
                    }

                    // Remove processed message
                    let remaining = serial_rx_length - (pos + 1);
                    serial_buffer.copy_within(pos + 1..serial_rx_length, 0);
                    serial_rx_length = remaining;
                }
            }
        }

        if frameno % 4 == 0 {
            let controls = nunchuck.get_input();
            // info!("controls state: {:?}", controls);
            active_demo.handle_controls(controls);
        }

        frameno += 1;

        for p in active_demo.get_path(frameno) {
            // Output coordinates
            lasers.display(p);

            // DAC/galvo settling time
            delay.delay_micros(p.delay as u32);
        }

        // yield to other tasks
        Timer::after(Duration::from_millis(1)).await;
    }
}
