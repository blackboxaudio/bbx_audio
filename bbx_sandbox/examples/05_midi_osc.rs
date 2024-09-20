use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{SystemTime};
use bbx_midi::message::MidiMessage;
use bbx_midi::stream::{create_midi_input_stream};

fn main() {
    let (tx, rx): (Sender<MidiMessage>, Receiver<MidiMessage>) = mpsc::channel();

    let _message_handler = thread::spawn(move || {
        loop {
            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let message = rx.recv().unwrap();
            println!("[{}] {:#}", now, message);
        }
    });

    match create_midi_input_stream(tx) {
        Ok(_) => (),
        Err(err) => println!("Error : {}", err),
    }
}
