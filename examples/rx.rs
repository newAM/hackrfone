use hackrfone::{
    iq_to_cplx_f32,         // build this example with "--features num-complex"
    num_complex::Complex32, // build this example with "--features num-complex"
    HackRfOne,
    RxMode,
    UnknownMode,
};
use std::{
    sync::mpsc::{self, TryRecvError},
    thread,
};

fn main() {
    let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().expect("Failed to open HackRF One");

    const FC: u64 = 915_000_000;
    const FS: u32 = 10_000_000;
    const DIV: u32 = 2;
    radio
        .set_sample_rate(FS * DIV, DIV)
        .expect("Failed to set sample rate");
    radio.set_freq(FC).expect("Failed to set frequency");
    radio
        .set_amp_enable(false)
        .expect("Failed to disable amplifier");
    radio
        .set_antenna_enable(0)
        .expect("Failed to disable antenna");
    radio.set_lna_gain(20).expect("Failed to set LNA gain");
    radio.set_vga_gain(32).expect("Failed to set VGA gain");
    let mut radio: HackRfOne<RxMode> = radio.into_rx_mode().expect("Failed to enter RX mode");

    let (data_tx, data_rx) = mpsc::channel();
    let (exit_tx, exit_rx) = mpsc::channel();

    let sample_thread = thread::Builder::new()
        .name("sample".to_string())
        .spawn(move || -> Result<(), hackrfone::Error> {
            println!("Spawned sample thread");

            loop {
                let samples: Vec<u8> = radio.rx()?;
                data_tx
                    .send(samples)
                    .expect("Failed to send buffer from sample thread");

                match exit_rx.try_recv() {
                    Ok(_) => {
                        radio.stop_rx()?;
                        return Ok(());
                    }
                    Err(TryRecvError::Disconnected) => {
                        println!("Main thread disconnected");
                        return Ok(());
                    }
                    Err(TryRecvError::Empty) => {}
                }
            }
        })
        .expect("Failed to spawn sample thread");

    const NUM_SAMPLES: usize = 1024 * 1024;
    let mut capture_buf: Vec<Complex32> = Vec::with_capacity(NUM_SAMPLES);

    loop {
        match data_rx.try_recv() {
            Ok(buf) => buf.chunks_exact(2).for_each(|iq| {
                capture_buf.push(iq_to_cplx_f32(iq[0], iq[1]));
            }),
            Err(TryRecvError::Disconnected) => {
                println!("Sample thread disconnected");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }

        // ... do signal processing with capture buf in the loop

        // ... or wait for the buffer to fill and do processing outside
        if capture_buf.len() >= NUM_SAMPLES {
            break;
        }
    }

    println!("Shutting down sample thread");

    exit_tx
        .send(())
        .expect("Failed to send exit event to sample thread");
    sample_thread
        .join()
        .expect("Failed to join sample thread")
        .expect("Sample thread returned an error");

    println!("Done");
}
