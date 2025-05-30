mod bump;
mod dummy;
mod frame;
mod linked_list;

pub use bump::*;
pub use dummy::*;
pub use frame::*;
pub use linked_list::*;

#[global_allocator]
pub static ALLOCATOR: Allocator<LinkedListAllocator> =
    Allocator::new(LinkedListAllocator::new());

pub struct Allocator<T> {
    inner: spin::Mutex<T>,
}

impl<T> Allocator<T> {
    pub const fn new(value: T) -> Self {
        Allocator {
            inner: spin::Mutex::new(value),
        }
    }
    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.inner.lock()
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
