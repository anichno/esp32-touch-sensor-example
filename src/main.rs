use core::ffi;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use embedded_hal::prelude::*;
use esp_idf_sys as sys;
use esp_idf_sys::esp; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

const TOUCH_PIN: u32 = 6;
static TOUCH_PIN_TOUCHED: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn handle_touch(_: *mut ffi::c_void) {
    let pad_intr = sys::touch_pad_get_status();
    if esp!(sys::touch_pad_clear_status()).is_ok() && (pad_intr >> TOUCH_PIN) & 1 > 0 {
        TOUCH_PIN_TOUCHED.store(true, Ordering::SeqCst);
    }
}

fn main() -> Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    let mut rtos = esp_idf_hal::delay::FreeRtos;

    esp!(unsafe { sys::touch_pad_init() })?;
    esp!(unsafe { sys::touch_pad_set_fsm_mode(sys::touch_fsm_mode_t_TOUCH_FSM_MODE_TIMER) })?;
    esp!(unsafe {
        sys::touch_pad_set_voltage(
            sys::touch_high_volt_t_TOUCH_HVOLT_2V7,
            sys::touch_low_volt_t_TOUCH_LVOLT_0V5,
            sys::touch_volt_atten_t_TOUCH_HVOLT_ATTEN_1V,
        )
    })?;
    esp!(unsafe { sys::touch_pad_config(TOUCH_PIN, 0) })?;
    esp!(unsafe { sys::touch_pad_filter_start(10) })?;

    let mut touch_value = 0;
    esp!(unsafe { sys::touch_pad_read_filtered(TOUCH_PIN, &mut touch_value) })?;
    let threshold = touch_value * 2 / 3;
    esp!(unsafe { sys::touch_pad_set_thresh(TOUCH_PIN, threshold) })?;

    esp!(unsafe { sys::touch_pad_isr_register(Some(handle_touch), ptr::null_mut()) })?;
    esp!(unsafe { sys::touch_pad_clear_status() })?;
    esp!(unsafe { sys::touch_pad_intr_enable() })?;

    println!("Waiting for touch");

    loop {
        if TOUCH_PIN_TOUCHED.load(Ordering::SeqCst) {
            println!("Touched");
            TOUCH_PIN_TOUCHED.store(false, Ordering::SeqCst);
        }
        rtos.delay_ms(10u32);
    }
}
