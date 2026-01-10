#![no_std]
#![no_main]

// THA PLANo
// ITERATorS of SECTIONS
// WHISKER
// Eye
// MOUTH
// THEn push them to the screen in order depending
// each will have to be hardcoded with rgb8 thingies

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self, Config};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_time::{Duration, Ticker};
use embedded_hal_1::i2c::I2c;
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8, brightness: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    let brightness: f32 = brightness as f32 / 255 as f32;
    if wheel_pos < 85 {
        return [255 - wheel_pos * 3, 0, wheel_pos * 3]
            .map(|x| (x as f32 * brightness) as u8)
            .into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return [0, wheel_pos * 3, 255 - wheel_pos * 3]
            .map(|x| (x as f32 * brightness) as u8)
            .into();
    }
    wheel_pos -= 170;
    [wheel_pos * 3, 255 - wheel_pos * 3, 0 as u8]
        .map(|x| (x as f32 * brightness) as u8)
        .into()
}

fn forms(pos: usize) -> RGB8 {

    let mut data = [RGB8::default(); 56];
    
    let test: [RGB8; 56] = [
        0,0,0,0,0,0,0,0,
        0,0,0,1,0,0,0,0,
        0,1,1,0,0,1,0,0,
        0,0,1,0,0,0,0,0,
        0,1,1,0,0,1,0,0,
        0,0,0,1,0,0,0,0,
        0,0,0,0,0,0,0,0,
    ]
    .map(|val| {
        match val {
            0 => (0,0,0).into(),
            1 => (255,255,255).into(),
            _ => (0,0,0).into(),
        }
    });

    test[pos]
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    // This is the number of leds in the string. Helpfully, the sparkfun thing plus and adafruit
    // feather boards for the 2040 both have one built in.
    const NUM_LEDS: usize = 256;
    const BRIGHTNESS: usize = 5;
    let mut data = [RGB8::default(); NUM_LEDS];

    let program = PioWs2812Program::new(&mut common);
    let mut ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_26, &program);

    for i in 0..56 {
        data[i] = forms(i);
    }
    ws2812.write(&data).await;
    
}
