#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    config::WatchdogConfig,
    gpio::{Input, InputConfig, Output, OutputConfig, Pull},
};
use esp_hal_embassy::main;
use esp_println::println;

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(
        esp_hal::Config::default()
            .with_cpu_clock(CpuClock::max())
            .with_watchdog(WatchdogConfig::default().with_timg0(
                esp_hal::config::WatchdogStatus::Enabled(esp_hal::time::Duration::millis_at_least(
                    1000,
                )),
            )),
    );

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);

    let mut wdt = timer0.wdt;
    wdt.feed();

    esp_hal_embassy::init(timer0.timer0);

    println!("Embassy initialized! Waiting for potential flashing signal");

    wdt.feed();

    let mut led = Output::new(
        peripherals.GPIO8,
        OutputConfig::default().with_level(esp_hal::gpio::Level::High),
    )
    .unwrap();

    wdt.feed();

    for _ in 0..20 {
        led.toggle();
        wdt.feed();
        Timer::after(Duration::from_millis(50)).await;
    }

    for _ in 0..20 {
        wdt.feed();
        Timer::after(Duration::from_millis(100)).await;
    }

    core::future::pending::<()>().await;
}
