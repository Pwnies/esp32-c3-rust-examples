#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, dma, dma_buffers, rng::Rng, spi, time::Rate};
use esp_hal_embassy::main;
use esp_println::println;

const NUM_PIXELS: usize = 16;

#[derive(Copy, Clone, Debug)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

impl Pixel {
    const BLACK: Pixel = Pixel { r: 0, g: 0, b: 0 };
}

type PixelArray = [Pixel; NUM_PIXELS];
// 3 colors per pixel, 4 bits of spi data per bit of color data
type PulseCodeArray = [u8; NUM_PIXELS * 3 * 4];

// This corresponds to 350ns of high followed by 1050s of low
const ZERO_PULSE: u8 = 0b1000;
// This corresponds to 700 ns of high followed by 700 ns of low
const ONE_PULSE: u8 = 0b1100;

#[main]
async fn main(_spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    println!("Embassy initialized!");

    Timer::after(Duration::from_secs(1)).await;

    let mut rng = Rng::new(peripherals.RNG);

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(3 * 3 * NUM_PIXELS);
    let dma_rx_buf = dma::DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = dma::DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();
    // The frequency 2857kHz is chosen because 1/2857kHz ~= 350.018 ns, which is pretty close to
    // our desired pulse length
    //
    // However since this exact frequency is not supported by the spi and instead esp-hal will
    // choose the closest matching frequency. This closest frequency is 80MHz/28, which corresponds
    // to bit-length of exactly 350 ns.
    let spi_config = spi::master::Config::default().with_frequency(Rate::from_khz(2857));

    let mut spidma = spi::master::Spi::new(peripherals.SPI2, spi_config)
        .unwrap()
        .with_mosi(peripherals.GPIO10)
        .with_dma(peripherals.DMA_CH0)
        .with_buffers(dma_rx_buf, dma_tx_buf)
        .into_async();

    let mut pixels: PixelArray = [Pixel::BLACK; NUM_PIXELS];
    let mut pulsecodes: PulseCodeArray = [0; NUM_PIXELS * 3 * 4];

    // When starting the spi, it will idle as high, which means that
    // from the point of view of the ws2812b we have already started
    // transmissing.
    //
    // While esp-hal configures the spi to idle as low, but this only takes
    // effect after the first transmission.
    //
    // To fix this, we transmit a burst of zeros. To also get the ws2812b to
    // abort the current transmission, we wait for 50 µs to reset it.
    spidma.write_async(&[0]).await.unwrap();
    Timer::after(Duration::from_micros(50)).await;

    loop {
        for p in &mut pixels {
            let d = rng.random();
            p.r = (d as u8) & 0x7;
            p.g = ((d >> 8) as u8) & 0x7;
            p.b = ((d >> 16) as u8) & 0x7;
        }

        for (pixel, pulsecode) in pixels.iter_mut().zip(pulsecodes.chunks_mut(3 * 4)) {
            pulsecode[0..4].copy_from_slice(&pixel_to_pulsecodes(pixel.g));
            pulsecode[4..8].copy_from_slice(&pixel_to_pulsecodes(pixel.r));
            pulsecode[8..12].copy_from_slice(&pixel_to_pulsecodes(pixel.b));
        }

        spidma.write_async(&pulsecodes).await.unwrap();
        Timer::after(Duration::from_secs(1)).await;
    }
}

fn pixel_to_pulsecodes(byte: u8) -> [u8; 4] {
    let mut out = [0; 4];
    for b in 0..8 {
        let bitpos = 7 - b;
        let bit = byte & (1 << bitpos) != 0;
        match (b & 1 != 0, bit) {
            (false, false) => out[b / 2] |= ZERO_PULSE << 4,
            (false, true) => out[b / 2] |= ONE_PULSE << 4,
            (true, false) => out[b / 2] |= ZERO_PULSE,
            (true, true) => out[b / 2] |= ONE_PULSE,
        }
    }
    out
}
