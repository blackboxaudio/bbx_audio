pub struct Phasor {
    ix: f32,
    iy: f32,
}

impl Phasor {
    pub fn new() -> Phasor {
        Phasor {
            ix: 0.5,
            iy: 0.5,
        }
    }
}

impl Phasor {
    pub fn apply(&self, phase: f32) -> f32 {
        if phase <= self.ix {
            phase * self.iy / self.ix
        } else {
            let slope = (1.0 - self.iy) / (1.0 - self.ix);
            ((phase - self.ix) * slope) + self.iy
        }
    }

    // pub fn add_pivot(&mut self, pivot: (f32, f32)) {
    //     self.inflections.push(pivot);
    //     self.inflections.sort_by(|p1, p2| p1.0.partial_cmp(&p2.0).unwrap());
    // }

    pub fn get_inflection(&self) -> (f32, f32) {
        (self.ix, self.iy)
    }

    pub fn set_pivot(&mut self, x: f32, y: f32) {
        self.ix = x.clamp(0.0, 1.0);
        self.iy = y.clamp(0.0, 1.0);
    }
}
