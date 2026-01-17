#![no_std]
#![no_main]

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

fn forms(mut pos: usize, pattern: char, bright: u8) -> RGB8 {

    let brightness: f32 = bright as f32 / 255 as f32;

    pos = if pos.div_euclid(8).rem_euclid(2) == 1 {
        7 - pos.rem_euclid(8) + (&pos / 8)*8
    } else {
        pos
    };

    let test: [RGB8; 184] = [
        0,0,0,0,0,0,0,0, //0
        
        9,0,0,0,0,0,0,0, //1 | 8
        0,0,0,9,0,0,0,0,
        0,0,9,0,0,9,9,0,
        0,0,9,0,0,0,0,0,
        0,0,9,0,0,9,9,0,
        0,0,0,9,0,0,0,0,
        0,0,0,0,0,0,0,0,

        0,0,0,1,0,0,0,0, //8 | 64
        0,0,0,1,0,0,0,0,
        1,0,0,1,0,0,0,1,
        0,1,0,0,0,0,1,0,
        0,0,1,0,0,1,0,0,

        0,0,9,0,0,0,0,0, //13 | 102
        0,9,0,0,0,0,0,0,
        9,0,0,0,0,0,0,0,
        0,9,0,0,0,0,0,0,
        0,0,9,0,0,0,0,0,

        0,0,0,9,0,0,0,9, //19
        0,0,0,0,9,0,9,0,
        0,0,0,0,0,9,0,0,
        0,0,0,0,9,0,9,0,
        0,0,0,9,0,0,0,9,
    ]
    .map(|val| {
        match val {
            0 => [0,0,0]
                .map(|x| (x as f32 * brightness) as u8)
                .into(),
            1 => [255,255,255]
                .map(|x| (x as f32 * brightness) as u8)
                .into(),
            9 => wheel(pos as u8, bright),
            _ => [0,0,0].into(),
        }
    });

    let cuteeyes: [u8; 32] = [
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        13,
        14,
        15,
        16,
        17,
        0,
        0,
        13,
        14,
        15,
        16,
        17,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];

    let whiskers: [u8; 32] = [
        0,
        0,
        0,
        8,
        9,
        10,
        11,
        12,
        0,
        0,
        13,
        14,
        15,
        16,
        17,
        0,
        0,
        13,
        14,
        15,
        16,
        17,
        0,
        0,
        12,
        11,
        10,
        9,
        8,
        0,
        0,
        0,
    ];

    let dead: [u8; 32] = [
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        19,
        20,
        21,
        22,
        23,
        0,
        0,
        19,
        20,
        21,
        22,
        23,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];

    let offsets = if pattern == 'b' {
        whiskers
    } else if pattern == 'c' {
        dead
    } else {
        cuteeyes
    };
    
    pos = (8*offsets[pos / 8] + (pos.rem_euclid(8) as u8)) as usize;

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
    const BRIGHTNESS: usize = 255;
    let mut data = [RGB8::default(); NUM_LEDS];

    let program = PioWs2812Program::new(&mut common);
    let mut ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_26, &program);

    for i in 0..256 {
        data[i] = forms(i, 'b', BRIGHTNESS as u8);
    }
    ws2812.write(&data).await;
    
}
