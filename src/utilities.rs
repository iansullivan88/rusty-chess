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
}

impl<T, const N: usize> Extend<T> for StackVector<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}