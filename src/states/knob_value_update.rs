#[derive(Debug, Copy, Clone)]
pub enum KnobValueUpdate {
    Increment,
    Decrement,
}

impl From<KnobValueUpdate> for u8 {
    fn from(value: KnobValueUpdate) -> Self {
        match value {
            KnobValueUpdate::Increment => 1,
            KnobValueUpdate::Decrement => 127,
        }
    }
}

impl From<KnobValueUpdate> for f32 {
    fn from(value: KnobValueUpdate) -> Self {
        match value {
            KnobValueUpdate::Increment => 1.0,
            KnobValueUpdate::Decrement => -1.0,
        }
    }
}

impl From<KnobValueUpdate> for i8 {
    fn from(value: KnobValueUpdate) -> Self {
        match value {
            KnobValueUpdate::Increment => 1,
            KnobValueUpdate::Decrement => -1,
        }
    }
}

impl From<u8> for KnobValueUpdate {
    fn from(value: u8) -> Self {
        if value > 63 {
            KnobValueUpdate::Decrement
        } else {
            KnobValueUpdate::Increment
        }
    }
}

impl From<i128> for KnobValueUpdate {
    fn from(value: i128) -> Self {
        if value.is_positive() {
            KnobValueUpdate::Increment
        } else {
            KnobValueUpdate::Decrement
        }
    }
}

impl From<i64> for KnobValueUpdate {
    fn from(value: i64) -> Self {
        if value.is_positive() {
            KnobValueUpdate::Increment
        } else {
            KnobValueUpdate::Decrement
        }
    }
}

impl From<i32> for KnobValueUpdate {
    fn from(value: i32) -> Self {
        if value.is_positive() {
            KnobValueUpdate::Increment
        } else {
            KnobValueUpdate::Decrement
        }
    }
}

impl From<i16> for KnobValueUpdate {
    fn from(value: i16) -> Self {
        if value.is_positive() {
            KnobValueUpdate::Increment
        } else {
            KnobValueUpdate::Decrement
        }
    }
}

impl From<i8> for KnobValueUpdate {
    fn from(value: i8) -> Self {
        if value.is_positive() {
            KnobValueUpdate::Increment
        } else {
            KnobValueUpdate::Decrement
        }
    }
}

impl From<usize> for KnobValueUpdate {
    fn from(value: usize) -> Self {
        if value > 63 {
            KnobValueUpdate::Increment
        } else {
            KnobValueUpdate::Decrement
        }
    }
}
