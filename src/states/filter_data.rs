use biquad::{DirectForm1, Type};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct FilterData {
    pub previous_filter_percentage: f32,
    pub filter_type: Type<f32>,
    pub filter: Arc<Mutex<DirectForm1<f32>>>,
}
