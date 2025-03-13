#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Pull},
};
use esp_hal_embassy::main;
use esp_println::println;

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    println!(
        "Embassy initialized! Waiting a bit before configuring pins to avoid accidentally yeeting the usb connection"
    );

    Timer::after(Duration::from_secs(1)).await;
    println!("Setting up pins");

    macro_rules! pins {
        ($(($cfg_up:tt, $cfg_down:tt, $cfg_none:tt, $pin:ident)),* $(,)?) => {
            [
                $(
                    #[cfg(feature = $cfg_up)]
                    (Input::new(peripherals.$pin, InputConfig::default().with_pull(Pull::Up)), Pull::Up),
                    #[cfg(feature = $cfg_down)]
                    (Input::new(peripherals.$pin, InputConfig::default().with_pull(Pull::Down)), Pull::Down),
                    #[cfg(feature = $cfg_none)]
                    (Input::new(peripherals.$pin, InputConfig::default().with_pull(Pull::None)), Pull::None),
                )*
            ]
        };
    }

    let pins = pins!(
        ("gpio0-up", "gpio0-down", "gpio0-none", GPIO0),
        ("gpio1-up", "gpio1-down", "gpio1-none", GPIO1),
        ("gpio2-up", "gpio2-down", "gpio2-none", GPIO2),
        ("gpio3-up", "gpio3-down", "gpio3-none", GPIO3),
        ("gpio4-up", "gpio4-down", "gpio4-none", GPIO4),
        ("gpio5-up", "gpio5-down", "gpio5-none", GPIO5),
        ("gpio6-up", "gpio6-down", "gpio6-none", GPIO6),
        ("gpio7-up", "gpio7-down", "gpio7-none", GPIO7),
        ("gpio8-up", "gpio8-down", "gpio8-none", GPIO8),
        ("gpio9-up", "gpio9-down", "gpio9-none", GPIO9),
        ("gpio10-up", "gpio10-down", "gpio10-none", GPIO10),
        ("gpio18-up", "gpio18-down", "gpio18-none", GPIO18),
        ("gpio19-up", "gpio19-down", "gpio19-none", GPIO19),
        ("gpio20-up", "gpio20-down", "gpio20-none", GPIO20),
        ("gpio21-up", "gpio21-down", "gpio21-none", GPIO21),
    );

    println!("Pins ready!");
    for (pin, pull) in &pins {
        println!(
            "Testing pin {} in with pull mode set to {pull:?}",
            pin.peripheral_input().number()
        );
    }

    let _ = spawner;

    let mut high_pins = heapless::Vec::new();
    loop {
        test_pins(&pins, &mut high_pins);
        println!("Pins: {high_pins:?}");
        Timer::after(Duration::from_secs(1)).await;
    }
}

fn test_pins<const N: usize>(
    pins: &[(Input<'_>, Pull); N],
    high_pins: &mut heapless::Vec<(u8, &'static str), N>,
) {
    high_pins.clear();
    for (pin, pull) in pins {
        match pull {
            Pull::Up if pin.is_low() => {
                let _ = high_pins.push((pin.peripheral_input().number(), "low"));
            }
            Pull::None | Pull::Down if pin.is_high() => {
                let _ = high_pins.push((pin.peripheral_input().number(), "high"));
            }
            _ => (),
        }
    }
}
