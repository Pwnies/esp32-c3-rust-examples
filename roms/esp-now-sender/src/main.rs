#![no_std]
#![no_main]

#[macro_use]
mod macros;

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, rng::Rng, timer::timg::TimerGroup};
use esp_println::println;
use esp_wifi::{EspWifiController, esp_now::BROADCAST_ADDRESS};

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let timg1 = TimerGroup::new(peripherals.TIMG1);
    let rng = Rng::new(peripherals.RNG);

    esp_hal_embassy::init(timg1.timer0);

    let init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK).unwrap()
    );

    let mut esp_now = esp_wifi::esp_now::EspNow::new(&init, peripherals.WIFI).unwrap();

    loop {
        if let Err(e) = esp_now.send_async(&BROADCAST_ADDRESS, b"test").await {
            println!("Error while sending: {e:?}");
        } else {
            println!("Ping sent");
        }
        Timer::after_secs(1).await;
    }
}
