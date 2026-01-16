#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use alloc::boxed::Box;
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::otg_fs::{Usb, UsbBus};
use esp_hal::timer::timg::TimerGroup;
use galvo_driver::protocol::{Command, Response};
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use vector_apps::apps;

use log::info;

use galvo_driver::lasers::Lasers;
use galvo_driver::led::IndicatorLed;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

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

    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let delay = Delay::new();

    let mut indicator = IndicatorLed::new(peripherals.GPIO38, peripherals.RMT, peripherals.GPIO39);

    let mut lasers = Lasers::new(
        peripherals.GPIO9,
        peripherals.GPIO8,
        peripherals.GPIO7,
        peripherals.DAC2,
        peripherals.GPIO18,
        peripherals.DAC1,
        peripherals.GPIO17,
    );

    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    let usb_bus = UsbBus::new(usb, unsafe { &mut *core::ptr::addr_of_mut!(EP_MEMORY) });

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x303A, 0x3001))
        .device_class(USB_CLASS_CDC)
        .build();

    // TODO: Spawn some tasks
    let _ = spawner;

    let mut serial_buffer: [u8; 2048] = [0; 2048];
    let mut serial_rx_length: usize = 0;

    // let mut active_demo: Box<dyn apps::VectorApp> = Box::new(apps::alphabet::AlphabetDemo::new());
    let mut active_demo: Box<dyn apps::VectorApp> = Box::new(apps::cube::CubeDemo::new());

    let mut frameno: u64 = 0;

    loop {
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];

            if let Ok(count) = serial.read(&mut buf) {
                if count > 0 {
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

                        if let Ok(s) = core::str::from_utf8(json_bytes) {
                            if let Ok(cmd) = serde_json::from_str::<Command>(s) {
                                match cmd {
                                    Command::SetIndicatorLight { r, g, b } => {
                                        indicator.set_color(smart_leds::RGB { r, g, b });
                                    }
                                }

                                let result = Response { success: true };

                                let response = serde_json::to_string(&result).unwrap();
                                let _ = serial.write(response.as_bytes());
                            }
                        }

                        // Remove processed message
                        let remaining = serial_rx_length - (pos + 1);
                        serial_buffer.copy_within(pos + 1..serial_rx_length, 0);
                        serial_rx_length = remaining;
                    }
                }
            }
        }

        frameno += 1;

        for p in active_demo.get_path(frameno).points {
            // Output coordinates
            lasers.display(&p);

            // DAC/galvo settling time
            delay.delay_micros(p.delay as u32);
        }
    }
}
