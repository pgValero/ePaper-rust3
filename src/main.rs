use embedded_svc::http::Headers;
mod display_interface;

use std::sync::{Arc, Mutex};
use display_interface::DisplayInterface;
use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::hal::gpio::{Gpio12, Gpio15, PinDriver};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::spi::config::{Config, DriverConfig};
use esp_idf_svc::hal::spi::SpiDeviceDriver;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::http::Method;
use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_svc::io::{Read, Write};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::ping::EspPing;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, AuthMethod, ClientConfiguration, Configuration, EspWifi, PmfConfiguration, ScanMethod};
use log::info;
use crate::display_interface::{ImageBuffer, N};

async fn connect_wifi(wifi: &mut AsyncWifi<EspWifi<'static>>, ssid: &str, password: &str) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration{
        ssid: ssid.parse().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: password.parse().unwrap(),
        channel: None,
        scan_method: ScanMethod::FastScan,
        pmf_cfg: PmfConfiguration::NotCapable,
    });

    wifi.set_configuration(&wifi_configuration).expect(" ");

    wifi.start().await?;
    info!("Wifi started");

    wifi.connect().await?;
    info!("Wifi connected");

    wifi.wait_netif_up().await?;
    info!("Wifi netif up");

    Ok(())
}

const RST_PIN: u8 = 21;
const DC_PIN: u8 = 19;
const BUSY_PIN: u8 = 22;
const PWR_PIN: u8 = 23;

const MOSI_PIN: u8 = 13;
const SCLK_PIN: u8 = 14;
const CS_PIN: u8 = 15;

fn main() {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let spi = peripherals.spi2;

    let sclk = peripherals.pins.gpio14;
    let sdo = peripherals.pins.gpio13;
    let sdi: Option<Gpio12> = None;
    let cs: Option<Gpio15> = Some(peripherals.pins.gpio15);

    let bus_config = DriverConfig::default();
    let config = Config::default();


    let mut spi = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sdo,
        sdi,
        cs,
        &bus_config,
        &config,
    ).unwrap();

    let rst = PinDriver::output(peripherals.pins.gpio21).unwrap();
    let dc = PinDriver::output(peripherals.pins.gpio19).unwrap();
    let pwr = PinDriver::output(peripherals.pins.gpio23).unwrap();
    
    let busy = PinDriver::input(peripherals.pins.gpio22).unwrap();

    let delay: Delay = Default::default();
    
    let mut display_interface = Arc::new(Mutex::new(DisplayInterface{
        rst_pin:rst,
        dc_pin: dc,
        pwr_pin: pwr,
        busy_pin: busy,
        spi,
        delay,
    }));

    let mut led_pin = PinDriver::output(peripherals.pins.gpio2).unwrap();

    led_pin.set_high().expect(" ");

    display_interface.lock().unwrap().init();
    // display_interface.display();
    // display_interface.clear();

    // display_interface.sleep();
    
    ///////////////////////////////////////////////////////////////////////////////////////////////
    
    let sysloop = EspSystemEventLoop::take().unwrap();
    let timer_service = EspTaskTimerService::new().unwrap();
    
    let modem = peripherals.modem;
    let ssid = "DIGIFIBRA-h4hD";
    let password = "4R3uyNuQAh";

    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), Some(EspDefaultNvsPartition::take().unwrap())).unwrap(),
        sysloop,
        timer_service.clone(),
    ).unwrap();

    block_on(connect_wifi(&mut wifi, ssid, password)).expect("TODO: panic message");

    let ip_info = wifi.wifi().sta_netif().get_ip_info().unwrap();
    println!("Wifi DHCP info: {:?}", ip_info);

    EspPing::default().ping(ip_info.subnet.gateway, &esp_idf_svc::ping::Configuration::default()).unwrap();

    let mut server = EspHttpServer::new(&Default::default()).unwrap();

    server.fn_handler::<anyhow::Error, _>("/", Method::Get, move |req| {
        req.into_ok_response().expect(" ")
            .write_all(b"Hello world from ESP32!")?;
        Ok(())
    }).unwrap();

    // server.fn_handler::<anyhow::Error, _>("/clear", Method::Get, move |req| {
    //     display_interface.lock().unwrap().clear();
    //     Ok(())
    // }).unwrap();
    
    server.fn_handler::<anyhow::Error, _>("/display", Method::Post, move |mut req| {

        let len = req.content_len().unwrap_or(0) as usize;

        if len != N {
            req.into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        let mut black_image = vec![0; len];
        req.read_exact(&mut black_image)?;

        println!("Response len: {}", len);

        let red_image: ImageBuffer = vec![0u8; N];

        display_interface.lock().unwrap().display(black_image, red_image);

        Ok(())
    }).unwrap();

    core::mem::forget(wifi);
    core::mem::forget(server);
    
    ///////////////////////////////////////////////////////////////////////////////////////////////
    
    led_pin.set_low().expect(" ");
    
}
