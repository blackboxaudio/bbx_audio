use std::{
    error::Error,
    io::{stdin, stdout, Write},
    sync::{mpsc, mpsc::Sender},
    thread,
    thread::JoinHandle,
};

use midir::{Ignore, MidiInput, MidiInputPort};

use crate::message::{MidiMessage, MidiMessageStatus};

pub struct MidiInputStream {
    tx: Sender<MidiMessage>,
    filters: Vec<MidiMessageStatus>,
}

impl MidiInputStream {
    pub fn new(filters: Vec<MidiMessageStatus>, message_handler: fn(MidiMessage) -> ()) -> Self {
        let (tx, rx) = mpsc::channel::<MidiMessage>();
        thread::spawn(move || loop {
            message_handler(rx.recv().unwrap());
        });
        MidiInputStream { tx, filters }
    }
}

impl MidiInputStream {
    pub fn init(self) -> JoinHandle<()> {
        println!("Creating new MIDI input stream");
        let mut midi_in = MidiInput::new("Reading MIDI input").unwrap();
        midi_in.ignore(Ignore::None);

        let in_ports = midi_in.ports();
        let in_port: Option<MidiInputPort> = match in_ports.len() {
            0 => None,
            1 => {
                println!(
                    "Choosing the only available MIDI input port:\n{}",
                    midi_in.port_name(&in_ports[0]).unwrap()
                );
                Some(in_ports[0].clone())
            }
            _ => {
                println!("\nAvailable MIDI input ports:");
                for (idx, port) in in_ports.iter().enumerate() {
                    println!("{}: {}", idx, midi_in.port_name(port).unwrap());
                }
                println!("\nPlease select input port: ");
                stdout().flush().unwrap();

                let mut input = String::new();
                stdin().read_line(&mut input).unwrap();
                Some(
                    in_ports
                        .get(input.trim().parse::<usize>().unwrap())
                        .ok_or("Invalid input port selected")
                        .unwrap()
                        .clone(),
                )
            }
        };
        thread::spawn(move || match self.create_midi_input_stream(midi_in, in_port.unwrap()) {
            Ok(_) => (),
            Err(err) => println!("Error : {}", err),
        })
    }

    fn create_midi_input_stream(
        self,
        midi_in: MidiInput,
        in_port: MidiInputPort,
    ) -> std::result::Result<(), Box<dyn Error>> {
        println!("\nOpening MIDI input stream for port");
        let in_port_name = midi_in.port_name(&in_port)?;
        let _connection = midi_in.connect(
            &in_port,
            "midir-read-input",
            move |_stamp, message_bytes, _| {
                let message = MidiMessage::from(message_bytes);
                if self.is_passed_through_filters(&message) {
                    self.tx.send(message).unwrap();
                } else {
                    // Message was "filtered" - do nothing
                }
            },
            (),
        )?;

        println!(
            "Connection open, reading MIDI input from '{}' (press enter to exit) ...",
            in_port_name
        );

        let mut input = String::new();
        input.clear();
        stdin().read_line(&mut input).unwrap();
        println!("Closing connection");

        Ok(())
    }

    fn is_passed_through_filters(&self, message: &MidiMessage) -> bool {
        if self.filters.is_empty() {
            true
        } else {
            self.filters.contains(&message.get_status())
        }
    }
}
