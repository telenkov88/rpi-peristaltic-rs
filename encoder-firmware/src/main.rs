#![no_std]
#![no_main]

use defmt::{debug, error, info};
use embassy_executor::{Executor, Spawner};
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::multicore::{spawn_core1, Stack};
use portable_atomic::{AtomicI32, Ordering};
use rotary_encoder_embedded::{Direction, InitalizeMode, RotaryEncoder};
use static_cell::StaticCell;

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, BufferedUartRx, Config};
use embedded_io_async::{Read, Write};

use encoder_protocol::{
    deserialize_with_crc, serialize_packet, Packet, SensorDataPacket, BUFFER_SIZE, MAX_ENCODERS,
};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

struct Encoders {
    encoders: [RotaryEncoder<InitalizeMode, Input<'static>, Input<'static>>; MAX_ENCODERS],
}

static ENCODER_COUNTS: [AtomicI32; MAX_ENCODERS] = [const { AtomicI32::new(0) }; MAX_ENCODERS];

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let (tx_pin, rx_pin, uart) = (p.PIN_0, p.PIN_1, p.UART0);

    static TX_BUF: StaticCell<[u8; BUFFER_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; BUFFER_SIZE])[..];
    static RX_BUF: StaticCell<[u8; BUFFER_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; BUFFER_SIZE])[..];
    let uart = BufferedUart::new(
        uart,
        tx_pin,
        rx_pin,
        Irqs,
        tx_buf,
        rx_buf,
        Config::default(),
    );
    let (mut tx, rx) = uart.split();

    spawner.must_spawn(reader(rx));

    let encoders = Encoders {
        encoders: [
            RotaryEncoder::new(Input::new(p.PIN_2, Pull::Up), Input::new(p.PIN_3, Pull::Up)),
            RotaryEncoder::new(Input::new(p.PIN_4, Pull::Up), Input::new(p.PIN_5, Pull::Up)),
            RotaryEncoder::new(Input::new(p.PIN_6, Pull::Up), Input::new(p.PIN_7, Pull::Up)),
            RotaryEncoder::new(Input::new(p.PIN_8, Pull::Up), Input::new(p.PIN_9, Pull::Up)),
            RotaryEncoder::new(
                Input::new(p.PIN_10, Pull::Up),
                Input::new(p.PIN_11, Pull::Up),
            ),
            RotaryEncoder::new(
                Input::new(p.PIN_12, Pull::Up),
                Input::new(p.PIN_13, Pull::Up),
            ),
            RotaryEncoder::new(
                Input::new(p.PIN_14, Pull::Up),
                Input::new(p.PIN_15, Pull::Up),
            ),
            RotaryEncoder::new(
                Input::new(p.PIN_27, Pull::Up),
                Input::new(p.PIN_26, Pull::Up),
            ),
        ],
    };

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| spawner.must_spawn(core1_task(encoders)));
        },
    );

    let mut sequence = 0u32;

    loop {
        embassy_time::Timer::after_millis(1000).await;

        let encoder_counts = ENCODER_COUNTS.each_ref().map(|c| c.load(Ordering::SeqCst));
        let sensor_data_packet = SensorDataPacket {
            seq: sequence,
            encoders: encoder_counts,
        };
        let packet = Packet::SensorData(sensor_data_packet);
        let buf = serialize_packet(&packet);

        debug!("TX Seq: {:?} Counts: {:?}", sequence, encoder_counts);
        debug!("TX Buffer {:?}", buf);

        #[cfg(debug_assertions)]
        {
            if let Some(packet) = deserialize_with_crc(&buf) {
                match packet {
                    Packet::SensorData(deserialized_packet) => {
                        debug!("✓ Deserialized data successfully");
                        debug!(
                            "  Original  seq: {}, encoders: {:?}",
                            sequence, encoder_counts
                        );
                        debug!(
                            "  Parsed    seq: {}, encoders: {:?}",
                            deserialized_packet.seq, deserialized_packet.encoders
                        );
                    }
                    _ => error!("Failed to deserialize"),
                }
            } else {
                error!("CRC error or invalid packet")
            }
        }

        tx.write_all(&buf).await.unwrap();
        sequence += 1;
    }
}

#[embassy_executor::task]
async fn core1_task(encoders: Encoders) {
    info!("Encoder samling started.");

    let mut encoders = encoders.encoders.map(|e| e.into_standard_mode());

    loop {
        for (i, en) in encoders.iter_mut().enumerate() {
            match en.update() {
                Direction::Clockwise => {
                    ENCODER_COUNTS[i].fetch_add(1, Ordering::SeqCst);
                    debug!(
                        "Encoder{} Count: {}",
                        i,
                        ENCODER_COUNTS[i].load(Ordering::SeqCst)
                    );
                }
                Direction::Anticlockwise => {
                    ENCODER_COUNTS[i].fetch_sub(1, Ordering::SeqCst);
                    debug!(
                        "Encoder{} Count: {}",
                        i,
                        ENCODER_COUNTS[i].load(Ordering::SeqCst)
                    );
                }
                Direction::None => {}
            }
        }
    }
}

#[embassy_executor::task]
async fn reader(mut rx: BufferedUartRx) {
    info!("Reading...");
    loop {
        let mut buf = [0; 1];
        rx.read_exact(&mut buf).await.unwrap();
        info!("RX {:?}", buf);
    }
}
