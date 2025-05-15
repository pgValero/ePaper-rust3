use embedded_svc::http::Headers;
mod display_interface;
mod wifi;

use std::sync::{Arc, Mutex};
use display_interface::DisplayInterface;
use esp_idf_svc::hal::gpio::{AnyInputPin, AnyOutputPin, PinDriver};
use esp_idf_svc::hal::prelude::Peripherals;

use esp_idf_svc::http::Method;
use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_svc::io::{Read, Write};
use display_interface::{ImageBuffer};
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
    
    let mut display_interface = Arc::new(Mutex::new(
        DisplayInterface::new(
            width,
            height,
            spi,
            rst,
            dc,
            pwr,
            busy,
            sclk,
            mosi,
            cs,
        ).expect("Error creation display interface.")
    ));

    let mut led_pin = PinDriver::output(peripherals.pins.gpio2).unwrap();

    led_pin.set_high().expect(" ");

    display_interface.lock().unwrap().init().expect("Error initializing display_interface.");
    display_interface.lock().unwrap().clear().expect("Error cleaning the display.");

    // display_interface.lock().unwrap().sleep().expect("TODO: panic message");

    let modem = peripherals.modem;
    let ssid = "DIGIFIBRA-h4hD";
    let password = "4R3uyNuQAh";

    let wifi = connect_wifi(modem, ssid, password).expect("Error connecting Wifi.");

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

        println!("Printing image...");

        let len = req.content_len().unwrap_or(0) as usize;
        let buffer_size = display_interface.lock().unwrap().buffer_size;

        if len != buffer_size {
            req.into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        let mut black_image = vec![0; len];
        req.read_exact(&mut black_image)?;


        let red_image: ImageBuffer = vec![0u8; buffer_size];

        display_interface.lock().unwrap().display(black_image, red_image).expect("Error displaying image.");
        // display_interface.lock().unwrap().sleep().expect("TODO: panic message");

        println!("Finished");

        Ok(())
    }).unwrap();

    core::mem::forget(wifi);
    core::mem::forget(server);

    led_pin.set_low().expect(" ");
    
}
