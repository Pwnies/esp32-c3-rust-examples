#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
// use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
};
use esp_hal_embassy::main;
// use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    esp_hal::system::software_reset()
}

#[main]
async fn main(spawner: Spawner) {
    //  esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    // println!(
    //     "Embassy initialized! Waiting a bit before configuring pins to avoid accidentally yeeting the usb connection"
    // );

    Timer::after(Duration::from_secs(1)).await;
    // println!("Setting up pins");

    let mut led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());

    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
