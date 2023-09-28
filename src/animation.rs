#[derive(Clone)]
pub enum AnimationTimingFunction {
    Linear,
    Ease,
}

impl AnimationTimingFunction {
    pub fn value(&self, duration_linear: f64) -> f64 {
        match self {
            AnimationTimingFunction::Linear => duration_linear,
            AnimationTimingFunction::Ease => {
                let t = duration_linear;
                t * t * (3.0 - 2.0 * t)
            }
        }
    }
}

pub trait Animation {
    fn duration(self, duration: f64) -> Self;
    fn timing_function(self, timing_function: AnimationTimingFunction) -> Self;

    fn tick_dt(&mut self, dt: f64) -> Self;
    fn reset(&mut self) -> Self;
    fn finish(&mut self) -> Self;
    fn is_done(&self) -> bool;
}

#[derive(Clone)]
pub struct AnimateValue {
    start: f64,
    end: f64,
    percent: f64,
    duration: f64,
    timing_function: AnimationTimingFunction,
}

impl AnimateValue {
    pub fn new() -> Self {
        AnimateValue {
            start: 0.0,
            end: 1.0,
            percent: 0.0,
            duration: 0.0,
            timing_function: AnimationTimingFunction::Linear,
        }
    }

    #[allow(dead_code)]
    pub fn start(mut self, start: f64) -> Self {
        self.start = start;
        self
    }

    #[allow(dead_code)]
    pub fn end(mut self, end: f64) -> Self {
        self.end = end;
        self
    }

    pub fn value(&self) -> f64 {
        self.start + (self.end - self.start) * self.timing_function.value(self.percent)
    }
}

impl Animation for AnimateValue {
    fn duration(mut self, duration: f64) -> Self {
        self.duration = duration;
        self
    }

    fn timing_function(mut self, timing_function: AnimationTimingFunction) -> Self {
        self.timing_function = timing_function;
        self
    }

    fn tick_dt(&mut self, dt: f64) -> Self {
        let increment = dt / self.duration;

        let v = self.percent + increment;
        let v = if v > 1.0 { 1.0 } else { v };

        self.percent = v;
        self.clone()
    }

    fn reset(&mut self) -> Self {
        self.percent = 0.0;
        self.clone()
    }

    fn finish(&mut self) -> Self {
        self.percent = 1.0;
        self.clone()
    }

    fn is_done(&self) -> bool {
        self.percent >= 1.0
    }
}

#[derive(Clone)]
pub struct AnimatePosition {
    pub start: (f64, f64),
    pub end: (f64, f64),
    animation: AnimateValue,
}

impl AnimatePosition {
    pub fn new() -> Self {
        AnimatePosition {
            start: (0.0, 0.0),
            end: (0.0, 0.0),
            animation: AnimateValue::new(),
        }
    }

    #[allow(dead_code)]
    pub fn start(mut self, start: (f64, f64)) -> Self {
        self.start = start;
        self
    }

    #[allow(dead_code)]
    pub fn end(mut self, end: (f64, f64)) -> Self {
        self.end = end;
        self
    }

    pub fn pos(&self) -> (f64, f64) {
        let p = self.animation.value();
        let x = self.start.0 + (self.end.0 - self.start.0) * p;
        let y = self.start.1 + (self.end.1 - self.start.1) * p;
        (x, y)
    }
}

impl Animation for AnimatePosition {
    fn duration(mut self, duration: f64) -> Self {
        self.animation = self.animation.clone().duration(duration);
        self
    }

    fn timing_function(mut self, timing_function: AnimationTimingFunction) -> Self {
        self.animation = self.animation.clone().timing_function(timing_function);
        self
    }

    fn tick_dt(&mut self, dt: f64) -> Self {
        self.animation.tick_dt(dt);
        self.clone()
    }

    fn reset(&mut self) -> Self {
        self.animation.reset();
        self.clone()
    }

    fn finish(&mut self) -> Self {
        self.animation.finish();
        self.clone()
    }

    fn is_done(&self) -> bool {
        self.animation.is_done()
    }
}
