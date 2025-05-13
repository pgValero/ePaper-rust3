use std::sync::{Arc, Mutex};
use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::hal::gpio::{Gpio19, Gpio21, Gpio22, Gpio23, Input, Output, PinDriver};
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 480;
pub const N: usize = (HEIGHT * WIDTH / 8) as usize;

pub type ImageBuffer = Vec<u8>;

pub struct DisplayInterface<'d> {
    pub rst_pin: PinDriver<'d, Gpio21, Output>,
    pub dc_pin: PinDriver<'d, Gpio19, Output>,
    pub pwr_pin: PinDriver<'d, Gpio23, Output>,
    pub busy_pin: PinDriver<'d, Gpio22, Input>,
    pub spi: SpiDeviceDriver<'d, SpiDriver<'d>>,
    pub delay: Delay,
    // pub black_image: ImageBuffer,
    // pub red_image: ImageBuffer,
}

impl<'d> DisplayInterface<'d> {
    
    
    pub fn init(&mut self) {

        self.pwr_pin.set_high().unwrap();
        self.reset();

        self.send_command(0x01);
        self.send_data(0x07);
        self.send_data(0x07);
        self.send_data(0x3f);
        self.send_data(0x3f);

        self.send_command(0x06);
        self.send_data(0x17);
        self.send_data(0x17);
        self.send_data(0x28);
        self.send_data(0x17);

        self.send_command(0x04);
        self.delay.delay_ms(100);
        self.read_busy();

        self.send_command(0x00);
        self.send_data(0x0F);

        self.send_command(0x61);
        self.send_data(0x03);
        self.send_data(0x20);
        self.send_data(0x01);
        self.send_data(0xE0);

        self.send_command(0x15);
        self.send_data(0x00);

        self.send_command(0x50);
        self.send_data(0x11);
        self.send_data(0x07);

        self.send_command(0x60);
        self.send_data(0x22);
    }

    fn reset(&mut self) {
        self.rst_pin.set_high().unwrap();
        self.delay.delay_ms(200);
        self.rst_pin.set_low().unwrap();
        self.delay.delay_ms(4);
        self.rst_pin.set_high().unwrap();
        self.delay.delay_ms(200);
    }

    fn exit(&mut self) {
        self.rst_pin.set_low().unwrap();
        self.dc_pin.set_low().unwrap();
        self.pwr_pin.set_low().unwrap();
    }

    pub fn send_command(&mut self, command: u8) {
        self.dc_pin.set_low().unwrap();
        self.spi.write(&[command]).expect("TODO: panic message");
    }

    fn send_data(&mut self, data: u8) {
        self.dc_pin.set_high().unwrap();
        self.spi.write(&[data]).expect("TODO: panic message");
    }

    fn send_data_2(&mut self, data: ImageBuffer) {
        self.dc_pin.set_high().unwrap();

        for b in data {
            self.spi.write(&[b]).expect("TODO: panic message");
        }
    }

    pub fn read_busy(&mut self) {
        self.send_command(0x71);
        
        while self.busy_pin.is_low() {
            self.send_command(0x71);
            self.delay.delay_ms(200);
        }
    }

    pub fn sleep(&mut self) {
        self.send_command(0x02);
        self.read_busy();

        self.send_command(0x07);
        self.send_data(0xA5);

        self.delay.delay_ms(2000);
        self.exit();
    }

    pub fn display(&mut self, black_image: ImageBuffer, red_image: ImageBuffer) {
        println!("{}", black_image.len());
        self.send_command(0x10);
        self.send_data_2(black_image);

        println!("{}", red_image.len());
        self.send_command(0x13);
        self.send_data_2(red_image);
    
        self.send_command(0x12);
        self.delay.delay_ms(100);
        self.read_busy();
    }

    pub fn clear(&mut self) {
    
        let buf: ImageBuffer = vec![0u8; N];
        let buf2: ImageBuffer = vec![255u8; N];
    
        self.send_command(0x10);
        self.send_data_2(buf2);
    
        self.send_command(0x13);
        self.send_data_2(buf);
    
        self.send_command(0x12);
        self.delay.delay_ms(100);
        self.read_busy();
    }

}