use num_traits::Float;

pub trait IDataPoint<T> {
    fn x(&self) -> T;
    fn y(&self) -> T;
}

#[derive(Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct DataPoint<T: Float + serde::Serialize> {
    pub(crate) x: T,
    pub(crate) y: T,
}

impl<T: Float + serde::Serialize> DataPoint<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn nan() -> Self {
        Self {
            x: T::nan(),
            y: T::nan(),
        }
    }
}

impl<T: Float + serde::Serialize> IDataPoint<T> for DataPoint<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }
}
