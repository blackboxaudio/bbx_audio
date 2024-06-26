use rodio::{OutputStream, Source};

use crate::{constants::PLAYTIME_DURATION, signal::Signal};

pub struct Player {
    signal: Signal,
}

impl Player {
    pub fn new(signal: Signal,) -> Player {
        return Player { signal, };
    }
}

impl Player {
    pub fn play(self,) {
        let (_stream, stream_handle,) = OutputStream::try_default().unwrap();
        let _result = stream_handle.play_raw(self.signal.convert_samples(),);

        std::thread::sleep(std::time::Duration::from_secs(PLAYTIME_DURATION,),);
    }
}
