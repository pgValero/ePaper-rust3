mod display_interface;
mod wifi;

use display_interface::DisplayInterface;
use embedded_svc::http::server::Request;
use embedded_svc::http::Headers;
use esp_idf_svc::hal::gpio::{AnyInputPin, AnyOutputPin, PinDriver};
use esp_idf_svc::hal::prelude::Peripherals;
use std::sync::{Arc, Mutex};

use display_interface::ImageBuffer;
use esp_idf_svc::http::server::{EspHttpConnection, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::{Read, Write};
use wifi::connect_wifi;

fn main() {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let spi = peripherals.spi2;

    let sclk = peripherals.pins.gpio14;
    let mosi = peripherals.pins.gpio13;
    let cs = peripherals.pins.gpio15;
    let rst = AnyOutputPin::from(peripherals.pins.gpio21);
    let dc = AnyOutputPin::from(peripherals.pins.gpio19);
    let pwr = AnyOutputPin::from(peripherals.pins.gpio23);
    let busy = AnyInputPin::from(peripherals.pins.gpio22);

    let width = 800usize;
    let height = 480usize;

    let mut display_interface =
        DisplayInterface::new(width, height, spi, rst, dc, pwr, busy, sclk, mosi, cs)
            .expect("Error creation display interface.");

    display_interface
        .init()
        .expect("Error initializing display.");
    display_interface
        .sleep()
        .expect("Error sleeping down display.");

    let display_interface = Arc::new(Mutex::new(display_interface));
    let led_pin = Arc::new(Mutex::new(
        PinDriver::output(peripherals.pins.gpio2).unwrap(),
    ));

    let modem = peripherals.modem;
    let ssid = "DIGIFIBRA-h4hD";
    let password = "4R3uyNuQAh";

    let wifi = connect_wifi(modem, ssid, password).expect("Error connecting Wifi.");

    let mut server = EspHttpServer::new(&Default::default()).unwrap();

    let display_function =
        move |mut request: Request<&mut EspHttpConnection>| -> Result<(), anyhow::Error> {
            led_pin.lock().unwrap().set_high()?;

            let len = request.content_len().unwrap() as usize;
            let buffer_size = display_interface.lock().unwrap().buffer_size;

            if len != buffer_size*2 {
                request
                    .into_status_response(413)?
                    .write_all("Request too big".as_bytes())?;
                return Ok(());
            }

            let mut buffer = vec![0; len];
            request.read_exact(&mut buffer)?;
            
            println!("Request received: {}", buffer.len());

            display_interface
                .lock()
                .unwrap()
                .init()
                .expect("Error initializing display.");
            display_interface
                .lock()
                .unwrap()
                .display(buffer)
                .expect("Error displaying image.");
            display_interface
                .lock()
                .unwrap()
                .sleep()
                .expect("Error sleeping down display.");

            led_pin.lock().unwrap().set_low()?;

            request.into_ok_response()?.write_all(b"Image displayed.")?;

            Ok(())
        };

    server
        .fn_handler::<anyhow::Error, _>("/display", Method::Post, display_function)
        .unwrap();

    core::mem::forget(wifi);
    core::mem::forget(server);
}
