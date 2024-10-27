/// The context in which a window is displayed.
pub struct DisplayContext {
    pub width: f32,
    pub height: f32,
    pub domain: (f32, f32),
    pub range: (f32, f32),
    pub padding: (f32, f32),
    pub buffer_size: usize,
}

impl DisplayContext {
    pub fn new(width: f32, height: f32, buffer_size: usize) -> DisplayContext {
        DisplayContext {
            width,
            height,
            buffer_size,
            domain: (-(width / 2.0), width / 2.0),
            range: (-(height / 2.0), height / 2.0),
            padding: (width * 0.05, height * 0.05),
        }
    }
}

impl Default for DisplayContext {
    fn default() -> Self {
        Self::new(1280.0, 720.0, 256)
    }
}

#[macro_export]
macro_rules! ctx {
    ($name:ident, $width:tt, $height:tt, $buffer_size:tt) => {
        $name {
            width: $width,
            height: $height,
            buffer_size: $buffer_size,
            domain: (-($width / 2.0), $width / 2.0),
            range: (-($height / 2.0), $height / 2.0),
            padding: ($width * 0.05, $height * 0.05),
        }
    };
}
