use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::hal::gpio::{AnyInputPin, AnyOutputPin, Input, Output, OutputPin, PinDriver};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::spi::config::Config as SpiConfig;
use esp_idf_svc::hal::spi::config::DriverConfig;
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver, SPI2};

pub type ImageBuffer = Vec<u8>;
type CustomError = anyhow::Error;

pub struct DisplayInterface<'d> {
    pub buffer_size: usize,
    rst_pin: PinDriver<'d, AnyOutputPin, Output>,
    dc_pin: PinDriver<'d, AnyOutputPin, Output>,
    pwr_pin: PinDriver<'d, AnyOutputPin, Output>,
    busy_pin: PinDriver<'d, AnyInputPin, Input>,
    spi: SpiDeviceDriver<'d, SpiDriver<'d>>,
    delay: Delay,
}

impl<'d> DisplayInterface<'d> {
    pub fn new(
        width: usize,
        height: usize,
        spi: SPI2,
        rst: AnyOutputPin,
        dc: AnyOutputPin,
        pwr: AnyOutputPin,
        busy: AnyInputPin,
        sclk: impl Peripheral<P = impl OutputPin> + 'd,
        mosi: impl Peripheral<P = impl OutputPin> + 'd,
        cs: impl Peripheral<P = impl OutputPin> + 'd,
    ) -> Result<Self, CustomError> {
        let sdo = mosi;
        let sdi: Option<AnyInputPin> = None;
        let cs = Some(cs);

        let bus_config = DriverConfig::default();
        let config = SpiConfig::default();

        let delay: Delay = Default::default();

        let spi = SpiDeviceDriver::new_single(spi, sclk, sdo, sdi, cs, &bus_config, &config)?;

        Ok(DisplayInterface {
            buffer_size: width * height / 8,
            rst_pin: PinDriver::output(rst)?,
            dc_pin: PinDriver::output(dc)?,
            pwr_pin: PinDriver::output(pwr)?,
            busy_pin: PinDriver::input(busy)?,
            spi,
            delay,
        })
    }

    pub fn init(&mut self) -> Result<(), CustomError> {
        self.pwr_pin.set_high()?;
        self.reset();

        self.send_command(0x01)?;
        self.send_data(0x07)?;
        self.send_data(0x07)?;
        self.send_data(0x3f)?;
        self.send_data(0x3f)?;

        self.send_command(0x06)?;
        self.send_data(0x17)?;
        self.send_data(0x17)?;
        self.send_data(0x28)?;
        self.send_data(0x17)?;

        self.send_command(0x04)?;
        self.delay.delay_ms(100);
        self.read_busy()?;

        self.send_command(0x00)?;
        self.send_data(0x0F)?;

        self.send_command(0x61)?;
        self.send_data(0x03)?;
        self.send_data(0x20)?;
        self.send_data(0x01)?;
        self.send_data(0xE0)?;

        self.send_command(0x15)?;
        self.send_data(0x00)?;

        self.send_command(0x50)?;
        self.send_data(0x11)?;
        self.send_data(0x07)?;

        self.send_command(0x60)?;
        self.send_data(0x22)?;
        Ok(())
    }

    fn reset(&mut self) {
        self.rst_pin.set_high().unwrap();
        self.delay.delay_ms(200);
        self.rst_pin.set_low().unwrap();
        self.delay.delay_ms(4);
        self.rst_pin.set_high().unwrap();
        self.delay.delay_ms(200);
    }

    fn exit(&mut self) -> Result<(), CustomError> {
        self.rst_pin.set_low()?;
        self.dc_pin.set_low()?;
        self.pwr_pin.set_low()?;
        Ok(())
    }

    fn send_command(&mut self, command: u8) -> Result<(), CustomError> {
        self.dc_pin.set_low()?;
        self.spi.write(&[command])?;
        Ok(())
    }

    fn send_data(&mut self, data: u8) -> Result<(), CustomError> {
        self.dc_pin.set_high()?;
        self.spi.write(&[data])?;
        Ok(())
    }

    fn send_data_2(&mut self, data: ImageBuffer) -> Result<(), CustomError> {
        self.dc_pin.set_high()?;

        for b in data {
            self.spi.write(&[b])?;
        }
        Ok(())
    }

    pub fn read_busy(&mut self) -> Result<(), CustomError> {
        self.send_command(0x71)?;

        while self.busy_pin.is_low() {
            self.send_command(0x71)?;
            self.delay.delay_ms(200);
        }
        Ok(())
    }

    pub fn sleep(&mut self) -> Result<(), CustomError> {
        self.send_command(0x02)?;
        self.read_busy()?;

        self.send_command(0x07)?;
        self.send_data(0xA5)?;

        self.delay.delay_ms(2000);
        self.exit()?;
        Ok(())
    }

    pub fn display(
        &mut self,
        black_image: ImageBuffer,
        red_image: ImageBuffer,
    ) -> Result<(), CustomError> {
        self.send_command(0x10)?;
        self.send_data_2(black_image)?;

        self.send_command(0x13)?;
        self.send_data_2(red_image)?;

        self.send_command(0x12)?;
        self.delay.delay_ms(100);
        self.read_busy()?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), CustomError> {
        let black_image: ImageBuffer = vec![255u8; self.buffer_size];
        let red_image: ImageBuffer = vec![0u8; self.buffer_size];

        self.display(black_image, red_image)?;
        Ok(())
    }
}
