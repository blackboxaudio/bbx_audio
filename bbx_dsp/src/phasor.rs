pub struct Phasor {
    px: f32,
    py: f32,
}

impl Phasor {
    pub fn new() -> Phasor {
        Phasor {
            px: 0.0,
            py: 0.0,
        }
    }
}

impl Phasor {
    pub fn apply(&self, phase: f32) -> f32 {
        if phase <= self.px {
            phase * self.py / self.px
        } else {
            let slope = (1.0 - self.py) / (1.0 - self.px);
            ((phase - self.px) * slope) + self.py
        }
    }

    pub fn set_pivot(&mut self, x: f32, y: f32) {
        self.px = x.clamp(0.0, 1.0);
        self.py = y.clamp(0.0, 1.0);
    }
}
