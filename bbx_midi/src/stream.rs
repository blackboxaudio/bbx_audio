use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc::Sender;
use midir::{Ignore, MidiInput};
use crate::message::MidiMessage;

pub fn create_midi_input_stream(tx: Sender<MidiMessage>) -> Result<(), Box<dyn Error>> {
    println!("Creating new MIDI input stream");
    let mut input = String::new();

    let mut midi_in = MidiInput::new("Reading MIDI input")?;
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("No input port found".into()),
        1 => {
            println!("Choosing the only available MIDI input port:\n{}", midi_in.port_name(&in_ports[0]).unwrap());
            &in_ports[0]
        },
        _ => {
            println!("\nAvailable MIDI input ports:");
            for (idx, port) in in_ports.iter().enumerate() {
                println!("{}: {}", idx, midi_in.port_name(port).unwrap());
            }
            println!("\nPlease select input port: ");
            stdout().flush()?;

            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports.get(input.trim().parse::<usize>()?).ok_or("Invalid input port selected")?
        }
    };

    println!("\nOpening MIDI input stream for port");
    let in_port_name = midi_in.port_name(in_port)?;
    let _connection = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_stamp, message_bytes, _| {
            let message = MidiMessage::from(message_bytes);
            tx.send(message).unwrap();
        },
        (),
    )?;

    println!("Connection open, reading MIDI input from '{}' (press enter to exit) ...", in_port_name);

    input.clear();
    stdin().read_line(&mut input)?;

    println!("Closing connection");
    Ok(())
}
