use core::net::{IpAddr, SocketAddr};

use embassy_net::{
    Runner, Stack,
    dns::DnsQueryType,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use embassy_time::{Duration, Timer};
use esp_hal::rtc_cntl::Rtc;
use esp_radio::wifi::{
    ClientConfig, ModeConfig, WifiController, WifiDevice, WifiEvent, WifiStaState,
};
use sntpc::{NtpContext, NtpTimestampGenerator, get_time};
use vector_apps::apps::clock::TimeSource;

const NTP_SERVER: &str = "pool.ntp.org";

const SSID: &str = "doggirl daycare";
const PASSWORD: &str = "puppykittenT4T";

pub type SharedRtc = Mutex<CriticalSectionRawMutex, Rtc<'static>>;

const USEC_IN_SEC: u64 = 1_000_000;

#[derive(Clone, Copy)]
struct Timestamp {
    rtc: &'static SharedRtc,
    current_time_us: u64,
}

impl NtpTimestampGenerator for Timestamp {
    fn init(&mut self) {
        self.rtc
            .lock(|rtc| self.current_time_us = rtc.current_time_us());
    }

    fn timestamp_sec(&self) -> u64 {
        self.current_time_us / USEC_IN_SEC
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        (self.current_time_us % USEC_IN_SEC) as u32
    }
}

pub struct RtcTimeSource {
    rtc: &'static SharedRtc,
}

impl RtcTimeSource {
    pub fn new(rtc: &'static SharedRtc) -> Self {
        Self { rtc }
    }
}

impl TimeSource for RtcTimeSource {
    fn now(&self) -> u64 {
        self.rtc.lock(|rtc| rtc.current_time_us() / USEC_IN_SEC)
    }
}

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>) {
    // println!("start connection task");
    // println!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_radio::wifi::sta_state() {
            WifiStaState::Connected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(SSID.into())
                    .with_password(PASSWORD.into()),
            );
            controller.set_config(&client_config).unwrap();
            // println!("Starting wifi");
            controller.start_async().await.unwrap();
            // println!("Wifi started!");

            // println!("Scan");
            // let scan_config = ScanConfig::default().with_max(10);
            // let result = controller
            //     .scan_with_config_async(scan_config)
            //     .await
            //     .unwrap();
            // for ap in result {
            //     println!("{:?}", ap);
            // }
        }
        // println!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => {
                // println!("Wifi connected!")
            }
            Err(_) => {
                // println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

pub async fn get_time_ntp(stack: &Stack<'_>, rtc: &'static SharedRtc) {
    let ntp_addrs = stack.dns_query(NTP_SERVER, DnsQueryType::A).await.unwrap();

    if ntp_addrs.is_empty() {
        panic!("Failed to resolve DNS. Empty result");
    }

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];

    let mut socket = UdpSocket::new(
        *stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    socket.bind(123).unwrap();

    loop {
        let addr: IpAddr = ntp_addrs[0].into();
        let result = get_time(
            SocketAddr::from((addr, 123)),
            &socket,
            NtpContext::new(Timestamp {
                rtc: &rtc,
                current_time_us: 0,
            }),
        )
        .await;

        match result {
            Ok(time) => {
                // Set time immediately after receiving to reduce time offset.
                {
                    rtc.lock(|rtc| {
                        rtc.set_current_time_us(
                            (time.sec() as u64 * USEC_IN_SEC)
                                + ((time.sec_fraction() as u64 * USEC_IN_SEC) >> 32),
                        );
                    });
                }

                return;
            }
            Err(_) => {
                // error!("Error getting time: {e:?}");
            }
        }
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
