use bbx_midi::stream::MidiInputStream;

fn main() {
    // Create a new MIDI input stream with a callback for when
    // a MIDI message is received (requires specifying a MIDI input
    // via the console)
    let stream = MidiInputStream::new(vec![], |message| {
        println!("{:#}", message);
    });

    // Initialize the stream and listen for incoming MIDI events
    let handle = stream.init();

    println!("\nDoing DSP operations...");

    // Wait for the user to cancel the program
    handle.join().unwrap();
}
