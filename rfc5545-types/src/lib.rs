pub mod request_status;
pub mod rrule;
pub mod set;
pub mod string;
pub mod time;
pub mod value;

pub mod primitive {
    use std::num::NonZero;

    pub type Integer = i32;
    pub type Float = f64;
    pub type PositiveInteger = NonZero<u32>;
}
