use core::net::{IpAddr, SocketAddr};

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec::Vec,
};
use embassy_net::{
    Runner, Stack,
    dns::DnsQueryType,
    tcp::TcpSocket,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use embassy_time::{Duration, Timer};
use embedded_tls::{Aes128GcmSha256, NoVerify, TlsConfig, TlsConnection, TlsContext};
use esp_hal::{
    rng::{Rng, Trng},
    rtc_cntl::Rtc,
};
use esp_radio::wifi::{
    ClientConfig, ModeConfig, WifiController, WifiDevice, WifiEvent, WifiStaState,
};
use rand_core::{CryptoRng, RngCore};
use serde_json::Value;
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

pub struct DangerouslyInsecureRng {
    rng: Rng,
}

impl RngCore for DangerouslyInsecureRng {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.rng.try_fill_bytes(dest)
    }
}

impl CryptoRng for DangerouslyInsecureRng {}

pub async fn get_mastodon_status(stack: &Stack<'_>) -> String {
    // --- DNS ---
    let addrs = stack
        .dns_query("tacobelllabs.net", DnsQueryType::A)
        .await
        .unwrap();

    if addrs.is_empty() {
        panic!();
    }

    let ip = addrs[0];

    // --- TCP socket buffers ---
    let mut rx_buffer = [0u8; 4096];
    let mut tx_buffer = [0u8; 4096];

    let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);

    socket.connect((ip, 443)).await.unwrap();

    // --- TLS ---
    let config = TlsConfig::<Aes128GcmSha256>::new().with_server_name("tacobelllabs.net");

    let mut read_record_buffer = [0; 16384];
    let mut write_record_buffer = [0; 16384];

    let mut tls = TlsConnection::new(socket, &mut read_record_buffer, &mut write_record_buffer);

    let mut rng = Trng::try_new().unwrap();

    tls.open::<_, NoVerify>(TlsContext::new(&config, &mut rng))
        .await
        .unwrap();

    // --- HTTP request ---
    let req = concat!(
        "GET /api/v1/statuses/115940750467337748 HTTP/1.1\r\n",
        "Host: tacobelllabs.net\r\n",
        "User-Agent: esp32\r\n",
        "Accept: application/json\r\n",
        "Connection: close\r\n",
        "\r\n",
    );

    let mut written = 0;
    let bytes = req.as_bytes();

    while written < bytes.len() {
        let n = tls.write(&bytes[written..]).await.unwrap();

        if n == 0 {
            // socket closed unexpectedly
            panic!();
        }

        written += n;
    }

    tls.flush().await.unwrap();

    // --- Read response ---
    let mut response = Vec::new();
    let mut buf = [0u8; 512];
    let mut content_length: Option<usize> = None;
    let mut body_start = None;

    loop {
        let n = tls.read(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }

        response.extend_from_slice(&buf[..n]);

        if body_start.is_none() {
            if let Some(pos) = response.windows(4).position(|w| w == b"\r\n\r\n") {
                body_start = Some(pos + 4);

                let headers = &response[..pos];
                let headers_str = core::str::from_utf8(headers).unwrap();

                for line in headers_str.lines() {
                    if let Some(v) = line.strip_prefix("Content-Length:") {
                        content_length = Some(v.trim().parse().unwrap());
                    }
                }

                break;
            }
        }
    }

    let body_start = body_start.unwrap();
    let content_length = content_length.unwrap();

    while response.len() < body_start + content_length {
        let n = tls.read(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }
        response.extend_from_slice(&buf[..n]);
    }

    let body = &response[body_start..];

    let json: Value = serde_json::from_slice(body).unwrap();

    let content = json
        .as_object()
        .unwrap()
        .get("content")
        .unwrap()
        .as_str()
        .unwrap()
        .replace("<p>", "")
        .replace("</p>", "");
    let author = json
        .as_object()
        .unwrap()
        .get("account")
        .unwrap()
        .as_object()
        .unwrap()
        .get("display_name")
        .unwrap()
        .as_str()
        .unwrap();

    format!("{} {}", author, content)
    // String::from_utf8_lossy(body).to_string()
}

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
