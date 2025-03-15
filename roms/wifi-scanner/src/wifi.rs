use embassy_time::{Duration, Timer};
use esp_hal::{
    peripheral::Peripheral,
    peripherals::{RADIO_CLK, WIFI},
    rng::Rng,
};
use esp_println::println;
use esp_wifi::{EspWifiController, wifi::ScanConfig};

pub(crate) async fn run_scanner(
    timg0_timer0: esp_hal::timer::timg::Timer,
    rng: Rng,
    radio_clk: impl Peripheral<P = RADIO_CLK> + 'static,
    wifi: impl Peripheral<P = WIFI> + 'static,
) -> ! {
    let init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timg0_timer0, rng, radio_clk).unwrap()
    );

    let (mut controller, _) = esp_wifi::wifi::new(&init, wifi).unwrap();

    println!("start connection task");
    println!("Device capabilities: {:?}", controller.capabilities());
    loop {
        if !matches!(controller.is_started(), Ok(true)) {
            println!("Starting wifi");
            controller.start_async().await.unwrap();
            println!("Wifi started!");
        }

        println!("Starting scan...");
        match controller
            .scan_with_config_async::<20>(ScanConfig::default())
            .await
        {
            Ok((res, count)) => {
                println!("Got {count} results");
                for r in res {
                    println!(
                        "ssid={:?} strength={} channel={} auth={:?}",
                        r.ssid, r.signal_strength, r.channel, r.auth_method
                    );
                }
            }
            Err(e) => {
                println!("Error while scanning {e:?}");
            }
        }
        Timer::after(Duration::from_millis(5000)).await
    }
}
