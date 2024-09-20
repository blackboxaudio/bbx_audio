use bbx_midi::stream::MidiInputStream;

fn main() {
    let stream = MidiInputStream::new(vec![], |message| {
        println!("{:#}", message);
    });
    let handle = stream.init();

    println!("\nDoing DSP operations...");

    handle.join().unwrap();
}
