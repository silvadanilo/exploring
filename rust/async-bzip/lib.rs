//! Non-blocking, buffered compression and decompression for bzip2

use std::cmp;
use std::io;
use std::io::{Read, Write};
use std::mem;

use bzip2_upstream::{Decompress, Status};
use futures::task;
use tokio_core::io::{EasyBuf, Io};

enum ParkedTask {
    ReadTask(task::Task),
    WriteTask(task::Task),
    None,
}

impl ParkedTask {
    pub fn take(&mut self) -> Self {
        mem::replace(self, ParkedTask::None)
    }
}

pub struct BzDecoder {
    data: Decompress,
    capacity: usize,
    buf: EasyBuf,
    done: bool,
    task: ParkedTask,
}

impl BzDecoder {
    pub fn new(buf_size: usize) -> BzDecoder {
        BzDecoder {
            data: Decompress::new(false),
            buf: EasyBuf::with_capacity(buf_size),
            capacity: buf_size,
            done: false,
            task: ParkedTask::None,
        }
    }

    pub fn total_in(&self) -> u64 {
        self.data.total_in()
    }
}

impl Write for BzDecoder {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if self.done || data.len() == 0 {
            return Ok(0);
        }

        if self.capacity == self.buf.len() {
            self.task = ParkedTask::WriteTask(task::park());
            return Err(io::Error::new(io::ErrorKind::WouldBlock, "buffer full"));
        }

        loop {
            let before = self.total_in();
            // TODO: look at doing this on a separate thread if necessary
            let res = self.data.decompress_vec(data, &mut self.buf.get_mut());
            let written = (self.total_in() - before) as usize;

            let res = res.map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

            if res == Status::StreamEnd {
                self.done = true;
            }

            if written > 0 || data.len() == 0 || self.done {
                self.task = match self.task.take() {
                    ParkedTask::ReadTask(ref t) => { (*t).unpark(); ParkedTask::None },
                    pt => pt,
                };
                return Ok(written);
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        // We've passed everything through the decompressor already.
        Ok(())
    }
}

impl Read for BzDecoder {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.buf.len() == 0 {
            if self.done {
                return Ok(0);
            }
            self.task = ParkedTask::ReadTask(task::park());
            return Err(io::Error::new(io::ErrorKind::WouldBlock, "buffer empty"));
        }

        let len = {
            let slice = self.buf.as_slice();
            let len = cmp::min(slice.len(), buf.len());
            if len == 0 {
                return Ok(0);
            }
            let slice = &slice[..len];
            let buf = &mut buf[..len];
            buf.copy_from_slice(slice);
            len
        };

        self.buf.drain_to(len);
        self.task = match self.task.take() {
            ParkedTask::WriteTask(ref t) => { (*t).unpark(); ParkedTask::None },
            pt => pt,
        };
        Ok(len)
    }
}

impl Io for BzDecoder {
}
