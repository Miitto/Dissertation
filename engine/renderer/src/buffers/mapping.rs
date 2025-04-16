use super::{MappingAddr, RawBuffer};

pub struct Mapping<'a, B: RawBuffer> {
    buffer: &'a mut B,
    ptr: MappingAddr,
    size: usize,
    needs_flush: bool,
    first_written: usize,
    last_written: usize,
    coherant: bool,
}

impl<'a, B: RawBuffer> Mapping<'a, B> {
    pub fn new(buffer: &'a mut B, loc: MappingAddr, size: usize, coherant: bool) -> Self {
        Self {
            buffer,
            ptr: loc,
            size,
            needs_flush: false,
            first_written: usize::MAX,
            last_written: usize::MIN,
            coherant,
        }
    }

    /// # Parameters
    /// - src: Pointer to data to write
    /// - size: Size of data to write
    /// - offset: Offset into mapping to write to
    ///
    /// # Safety
    /// src must be a valid pointer for the length of size
    pub unsafe fn write(&mut self, src: *const u8, size: usize, offset: usize) {
        assert!(
            size + offset <= self.size,
            "{} + {} <= {}",
            size,
            offset,
            self.size
        );

        let dst = self.ptr.lock().unwrap();

        unsafe { std::ptr::copy_nonoverlapping(src, dst.add(offset) as *mut u8, size) }

        if !self.coherant {
            self.needs_flush = true;
        }
        self.first_written = self.first_written.min(offset);
        self.last_written = self.last_written.max(offset + size);
    }
}

impl<'a, B: RawBuffer> Drop for Mapping<'a, B> {
    fn drop(&mut self) {
        if !self.needs_flush {
            return;
        }

        if !self.coherant {
            unsafe {
                gl::FlushMappedNamedBufferRange(
                    self.buffer.id(),
                    self.first_written as isize,
                    (self.last_written - self.first_written) as isize,
                )
            }
        }

        self.buffer.on_map_flush();
    }
}
