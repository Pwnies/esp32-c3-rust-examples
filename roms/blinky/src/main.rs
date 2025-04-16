#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
};
use esp_hal_embassy::main;
use esp_println::println;
// use esp_println::println;

#[main]
async fn main(_spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    println!(
        "Embassy initialized! Waiting a bit before configuring pins to avoid accidentally yeeting the usb connection"
    );

    Timer::after(Duration::from_secs(1)).await;

    println!("Blinking!");

    let mut led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());

    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
