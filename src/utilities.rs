use std::ops::Index;

#[derive(Copy, Clone, Debug)]
pub struct StackVector<T, const N: usize> {
    items: [T; N],
    size: usize
}

impl<T, const N: usize> StackVector<T, N> {
    pub fn new(default_value: T) -> Self where T: Copy {
        Self {
            items: [default_value; N],
            size: 0,
        }
    }

    pub fn push(&mut self, new_item: T) -> () {
        if self.size == N {
            panic!("Stack vector size exceeded");
        }
        self.items[self.size] = new_item;
        self.size+=1;
    }

    pub fn to_slice(&self) -> &[T] {
        &self.items[0..self.size]
    }

    pub fn len(&self) -> usize {
        self.size
    }
}

impl<T, const N: usize> Extend<T> for StackVector<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<T, const N: usize> Index<usize> for StackVector<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= N {
            panic!("StackVector index out of bounds {}", index);
        }
        &self.items[index]
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct NonNanFloat(f64);

impl Eq for NonNanFloat {}

impl NonNanFloat {
    pub fn new(val: f64) -> Self {
        if val.is_nan() {
            panic!("Cannot create NonNanFloat with NaN");
        }

        Self(val)
    }

    pub fn val(self) -> f64 {
        self.0
    }
}

impl PartialOrd for NonNanFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NonNanFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}