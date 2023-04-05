use rand::{Rng, prelude::StdRng, SeedableRng};

use crate::services::weather::scale::*;

/// Dimension = 336
pub struct NumberGen {
    pub val: f32,
    rng: StdRng,
}

impl Default for NumberGen {
    #[inline]
    #[rustfmt::skip]
    fn default() -> Self {
        Self { val: 0., rng: StdRng::from_entropy(), }
    }
}

impl NumberGen {
    #[allow(clippy::should_implement_trait)]
    #[rustfmt::skip]
    #[inline]
    pub fn next(&mut self) -> f32 {
        self.val += 0.2;
        if self.val > 360. { self.val = 0.; };
        self.val
    }

    #[inline]
    #[rustfmt::skip]
    pub fn rand(&mut self) -> f32 { self.rng.gen::<f32>() }
}

/// Dimension 368
pub struct MockSimulation {
    // for simulation/testing
    scale_x: Scale,
    scale_temp: Scale,
    scale_press: Scale,
    sequence: NumberGen,
}

impl MockSimulation {
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        Self {
            sequence: NumberGen::default(),
            scale_x: Scale {
                min: 0.9,
                max: 1.1,
            },
            scale_temp: Scale {
                min: -16.,
                max: 50.,
            },
            scale_press: Scale {
                min: 870.,
                max: 1085.,
            },
        }
    }
}

pub trait MockSim {
    fn next(&mut self) -> f32;
    fn rand(&mut self) -> f32;
    fn get_x(&mut self) -> f32;
    fn get_temp(&self) -> f32;
    fn get_press(&self) -> f32;
    fn span_temp(&self) -> f32;
    fn span_press(&self) -> f32;
}

#[rustfmt::skip]
impl MockSim for Option<MockSimulation> {
    #[inline]
    fn next(&mut self) -> f32 { self.as_mut().unwrap().sequence.next() }

    #[inline]
    fn rand(&mut self) -> f32 { self.as_mut().unwrap().sequence.rand() }

    #[inline]
    fn get_x(&mut self) -> f32 {
        let val = self.rand();
        self.as_mut().unwrap().scale_x.get(val)
    }

    #[inline]
    fn get_temp(&self) -> f32 { self.as_ref().unwrap().scale_temp.middle() }

    #[inline]
    fn get_press(&self) -> f32 { self.as_ref().unwrap().scale_press.middle() }

    #[inline]
    fn span_temp(&self) -> f32 { self.as_ref().unwrap().scale_temp.span() }

    #[inline]
    fn span_press(&self) -> f32 { self.as_ref().unwrap().scale_press.span() }
}

pub static mut MOCK_SIM: Option<MockSimulation> = None; 

unsafe impl Sync for MockSimulation {}