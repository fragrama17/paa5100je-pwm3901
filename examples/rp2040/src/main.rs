#![no_std]
#![no_main]

use defmt::{debug, error};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi::Spi;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};
use paa5100je_pmw3901::{PixArtSensor, RotationDegrees};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let spi_config = embassy_rp::spi::Config::default();

    let spi = Spi::new(
        p.SPI0, p.PIN_2, p.PIN_3, p.PIN_4, p.DMA_CH0, p.DMA_CH1, spi_config,
    );

    let spi_bus = Mutex::<NoopRawMutex, _>::new(spi);
    let cs = Output::new(p.PIN_5, Level::High);
    let spi_device = SpiDevice::new(&spi_bus, cs);

    let mut sensor = PixArtSensor::new_paa5100je(spi_device, &mut Delay)
        .await
        .unwrap();

    sensor.set_rotation(RotationDegrees::_0).await.unwrap();

    loop {
        match sensor.get_motion().await {
            Ok(delta) => {
                debug!("Motion Delta: (x: {}, y: {})", delta.x, delta.y);
            }
            Err(_) => {
                error!("sensor error detected");
            }
        }
        Timer::after_secs(1).await;
    }
}
