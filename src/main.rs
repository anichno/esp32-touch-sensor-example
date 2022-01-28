use anyhow::Result;
use embedded_hal::prelude::*;
use esp_idf_hal::peripherals;
use esp_idf_sys as _;

mod touch;

fn main() -> Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    let mut rtos = esp_idf_hal::delay::FreeRtos;

    let peripherals = peripherals::Peripherals::take().unwrap();
    let touch_pin = peripherals.pins.gpio14;

    let mut touch_builder = touch::TouchControllerBuilder::new()?;
    let touch_pin = touch_builder.add_pin(touch_pin)?;

    let touch_controller = touch_builder.build()?;

    loop {
        println!("{}", touch_controller.read(&touch_pin)?);
        if touch_controller.touched(&touch_pin) {
            println!("Touched");
        }
        rtos.delay_ms(100u32);
    }
}
