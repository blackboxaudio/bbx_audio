use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{SystemTime};
use bbx_midi::message::MidiMessage;
use bbx_midi::stream::{MidiInputStream};

fn main() {
    let (tx, rx): (Sender<MidiMessage>, Receiver<MidiMessage>) = mpsc::channel();

    let _message_handler = thread::spawn(move || {
        loop {
            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let message = rx.recv().unwrap();
            println!("[{}] {:#}", now, message);
        }
    });

    let stream = MidiInputStream::new(tx, None);
    match stream.init() {
        Ok(_) => (),
        Err(err) => println!("Error : {}", err),
    }
}
