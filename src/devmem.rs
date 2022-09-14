use super::matrix::{Cell, MatrixData};
use libc::{O_RDWR, O_SYNC};
use memmap::MmapOptions;
use std::fs::OpenOptions;
use std::os::unix::prelude::OpenOptionsExt;

const MEMDEV: &str = "/dev/mem";

pub fn read_byte(offset: u64) -> Option<u8> {
    if let Ok(file) = OpenOptions::new().read(true).open(MEMDEV) {
        if let Ok(mmap) = unsafe { MmapOptions::new().offset(offset).len(1).map(&file) } {
            return Some(mmap[0]);
        } else {
            return None;
        }
    }
    None
}

pub fn write(offset: u64, bytes: Vec<u8>) {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(O_RDWR | O_SYNC)
        .open(MEMDEV);
    if file.is_err() {
        println!("fail to open /dev/mem");
        return;
    }

    let mmap = unsafe {
        MmapOptions::new()
            .offset(offset)
            .len(bytes.len())
            .map_mut(&file.unwrap())
    };
    if mmap.is_err() {
        println!("fail to map /dev/mem");
        return;
    }
    let mut mmap = mmap.unwrap();

    mmap.copy_from_slice(&bytes)
}

pub struct Devmem {
    pub inner: Vec<Option<u8>>,
    pub size: u16,
}

impl MatrixData for Devmem {
    fn new(size: u16) -> Self {
        let mut dm = Self {
            inner: vec![],
            size,
        };
        dm.update(0);
        dm
    }

    fn write(&self, offset: u64, bytes: Vec<u8>) {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(O_RDWR | O_SYNC)
            .open(MEMDEV);
        if file.is_err() {
            println!("fail to open /dev/mem");
            return;
        }

        let mmap = unsafe {
            MmapOptions::new()
                .offset(offset)
                .len(bytes.len())
                .map_mut(&file.unwrap())
        };
        if mmap.is_err() {
            println!("fail to map /dev/mem");
            return;
        }
        let mut mmap = mmap.unwrap();

        mmap.copy_from_slice(&bytes)
    }

    fn update(&mut self, start: u64) {
        self.inner.clear();
        for i in start..start + self.size as u64 {
            self.inner.push(read_byte(i));
        }
    }

    fn get(&self, index: usize) -> Option<Cell> {
        if index < self.size as usize {
            return Some(Cell {
                inner: self.inner[index],
            });
        }
        None
    }
}
