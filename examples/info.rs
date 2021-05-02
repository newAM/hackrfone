use hackrfone::{HackRfOne, UnknownMode};

fn main() {
    let radio: HackRfOne<UnknownMode> = HackRfOne::new().expect("Failed to open HackRF One");
    println!("Board ID: {:?}", radio.board_id());
    println!("Version: {:?}", radio.version());
    println!("Device version: {:?}", radio.device_version());
}
