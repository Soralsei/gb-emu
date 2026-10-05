use std::{
    array,
    mem::{self, MaybeUninit},
};

#[derive(Debug)] // Removed Default
pub struct CircularBuffer<T, const SIZE: usize> {
    head: usize,
    tail: usize,
    size: usize,
    data: [T; SIZE],
}

impl<T, const SIZE: usize> Default for CircularBuffer<T, SIZE>
where
    T: Default,
{
    fn default() -> Self {
        const { assert!(SIZE > 0, "CircularBuffer SIZE must be non-zero") };
        Self {
            head: 0,
            tail: 0,
            size: 0,
            data: array::from_fn(|_| T::default()),
        }
    }
}

impl<T, const SIZE: usize> CircularBuffer<T, SIZE>
where
    T: Default,
{
    pub fn new() -> CircularBuffer<T, SIZE> {
        Self::default()
    }

    pub fn new_boxed() -> Box<Self> {
        const { assert!(SIZE > 0, "CircularBuffer SIZE must be non-zero") };

        // Allocate uninitialized heap memory for the struct
        let mut boxed: Box<MaybeUninit<Self>> = Box::new_uninit();

        // Initialize fields directly in heap memory
        let ptr = boxed.as_mut_ptr();
        unsafe {
            (*ptr).head = 0;
            (*ptr).tail = 0;
            (*ptr).size = 0;

            // Initialize `data` element by element directly in heap memory
            let data_ptr = std::ptr::addr_of_mut!((*ptr).data) as *mut T;
            for i in 0..SIZE {
                data_ptr.add(i).write(T::default());
            }

            // Cast to fully initialized Box<Self>
            boxed.assume_init()
        }
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.size = 0;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let item = mem::take(&mut self.data[self.head]);
        self.head = (self.head + 1) % SIZE;
        self.size -= 1;

        Some(item)
    }
}

impl<T, const SIZE: usize> CircularBuffer<T, SIZE> {
    pub fn fill(&mut self, items: [T; SIZE]) {
        self.data = items;
        self.head = 0;
        self.tail = SIZE - 1;
        self.size = SIZE;
    }

    pub fn push(&mut self, val: T) -> Result<(), T> {
        if self.is_full() {
            return Err(val);
        }
        self.push_impl(val);
        Ok(())
    }

    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        Some(&self.data[self.head])
    }

    pub fn push_overwrite(&mut self, val: T) {
        if self.is_full() {
            self.head = (self.head + 1) % SIZE;
            self.size -= 1;
        }
        self.push_impl(val);
    }

    fn push_impl(&mut self, val: T) {
        self.data[self.tail] = val;
        self.tail = (self.tail + 1) % SIZE;
        self.size += 1;
    }

    pub fn get_mut(&mut self, i: usize) -> Option<&mut T> {
        if i < self.size {
            Some(&mut self.data[(self.head + i) % SIZE])
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn is_full(&self) -> bool {
        self.size == SIZE
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn drain<'a>(&'a mut self) -> CircularIterator<'a, T, SIZE> {
        CircularIterator { buffer: self }
    }
}

pub struct CircularIterator<'a, T, const SIZE: usize> {
    buffer: &'a mut CircularBuffer<T, SIZE>,
}

impl<'a, T, const SIZE: usize> Iterator for CircularIterator<'a, T, SIZE>
where
    T: Default,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.pop()
    }
}

impl<'a, T, const SIZE: usize> ExactSizeIterator for CircularIterator<'a, T, SIZE>
where
    T: Default,
{
    fn len(&self) -> usize {
        self.buffer.len()
    }
}
