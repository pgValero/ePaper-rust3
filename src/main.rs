mod display_interface;
// mod api;

// use api::API;
use display_interface::DisplayInterface;
use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::hal::gpio::{Gpio12, Gpio15, PinDriver};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::spi::config::{Config, DriverConfig};
use esp_idf_svc::hal::spi::SpiDeviceDriver;


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

    let mut display_interface = DisplayInterface{
        rst_pin:rst,
        dc_pin: dc,
        pwr_pin: pwr,
        busy_pin: busy,
        spi,
        delay,
    };

    let mut led_pin = PinDriver::output(peripherals.pins.gpio2).unwrap();

    led_pin.set_high().expect(" ");

    display_interface.init();
    display_interface.display();
    // display_interface.clear();

    display_interface.sleep();
    
    // let api = API::new("", "", peripherals.modem);

    led_pin.set_low().expect(" ");
    
}
