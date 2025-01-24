use std::{
    alloc::{alloc, handle_alloc_error, realloc, Layout},
    mem::ManuallyDrop,
    ptr::{copy_nonoverlapping, read, NonNull},
};

#[derive(Debug)]
pub struct AnyVec {
    ptr: NonNull<u8>,
    layout: Layout,
    len: usize,
    cap: usize,
}

impl AnyVec {
    pub fn new(layout: Layout) -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
            layout,
        }
    }

    fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, self.layout)
        } else {
            (
                self.cap * 2,
                Layout::from_size_align(self.layout.size() * self.cap * 2, self.layout.align())
                    .unwrap(),
            )
        };

        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) }
        } else {
            let old_layout =
                Layout::from_size_align(self.layout.size() * self.cap, self.layout.align())
                    .unwrap();
            unsafe { realloc(self.ptr.as_ptr(), old_layout, new_layout.size()) }
        };

        self.cap = new_cap;
        self.ptr = match NonNull::new(new_ptr) {
            Some(ptr) => ptr,
            None => handle_alloc_error(new_layout),
        }
    }

    pub fn push_raw(&mut self, bytes: *const u8) {
        if self.len == self.cap {
            self.grow();
        }

        let dst = unsafe { self.ptr.as_ptr().add(self.len * self.layout.size()) };

        unsafe {
            copy_nonoverlapping(bytes, dst, self.layout.size());
        }

        self.len += 1;
    }

    pub fn push<T>(&mut self, element: T) {
        if self.len == self.cap {
            self.grow();
        }

        let ptr = ManuallyDrop::new(element);
        let src = NonNull::from(&*ptr).as_ptr() as *const u8;

        let dst = unsafe { self.ptr.as_ptr().add(self.len * self.layout.size()) };

        unsafe {
            copy_nonoverlapping(src, dst, self.layout.size());
        };

        self.len += 1;
    }

    pub fn pop<T>(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        let src = unsafe { self.ptr.as_ptr().add(self.len * self.layout.size()) as *const T };
        Some(unsafe { read(src) })
    }

    pub fn get_raw(&self, index: usize) -> Option<*const u8> {
        if index >= self.len {
            return None;
        }

        Some(unsafe { self.ptr.as_ptr().add(index * self.layout.size()) })
    }

    pub fn get<T>(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }

        let ptr = unsafe { self.ptr.as_ptr().add(index * self.layout.size()) };
        let ptr = ptr.cast::<T>();

        Some(unsafe { &*ptr })
    }

    pub fn get_mut<T>(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }

        let ptr = unsafe { self.ptr.as_ptr().add(index * self.layout.size()) };
        let ptr = ptr.cast::<T>();

        Some(unsafe { &mut *ptr })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn first<T>(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }

        self.get(0)
    }
}
