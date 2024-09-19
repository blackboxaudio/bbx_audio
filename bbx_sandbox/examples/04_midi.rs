use bbx_midi::stream::{create_midi_input_stream};

fn main() {
    match create_midi_input_stream() {
        Ok(_) => (),
        Err(err) => println!("Error : {}", err),
    }
}
