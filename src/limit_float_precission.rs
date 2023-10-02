use num_traits::Float;
use serde::Serializer;

pub fn serialize_float_2dgt<T: Float, S>(x: &T, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let x = x.to_f32().unwrap();
    if x.is_finite() {
        let str = format!("{:.2}", x);
        let parsed = str.parse::<f32>().unwrap();
        s.serialize_f32(parsed)
    } else {
        s.serialize_f32(x)
    }
}

pub fn serialize_opt_float_2dgt<T: Float, S>(x: &Option<T>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(x) = x {
        serialize_float_2dgt(x, s)
    } else {
        s.serialize_none()
    }
}
