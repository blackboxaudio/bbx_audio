use bbx_midi::stream::{MidiInputStream};

fn main() {
    let stream = MidiInputStream::new(vec![], |message| {
        println!("{:#}", message);
    });
    stream.init();

    println!("\nDoing DSP operations...");
    loop {}
}
