pub struct Phasor {
    inflections: Vec<(f32, f32)>,
}

impl Phasor {
    pub fn new() -> Phasor {
        Phasor {
            inflections: vec![(0.0, 0.0), (1.0, 1.0)],
        }
    }
}

impl Phasor {
    pub fn apply(&self, phase: f32) -> f32 {
        for idx in 0..self.inflections.len() - 1 {
            let (x1, y1) = self.inflections[idx];
            let (x2, y2) = self.inflections[idx + 1];
            if x1 <= phase && phase <= x2 {
                let slope = (y2 - y1) / (x2 - x1);
                return slope * (phase - x1) + y1;
            }
        }

        0.0
    }

    pub fn add_inflection(&mut self, x: f32, y: f32) {
        self.inflections.push((
            x.clamp(0.0, 1.0),
            y.clamp(0.0, 1.0)
        ));
        self.inflections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    pub fn get_inflection(&self, index: usize) -> &(f32, f32) {
        self.inflections.get(index).unwrap_or(&(0.0, 0.0))
    }

    pub fn get_inflections(&self) -> &[(f32, f32)] {
        self.inflections.as_slice()
    }

    pub fn set_inflection(&mut self, index: usize, x: f32, y: f32) {
        if index < self.inflections.len() {
            self.inflections[index] = (x, y);
            self.inflections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }
    }
}
