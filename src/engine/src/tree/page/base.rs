use std::marker::PhantomData;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PagePtr {
    Mem(u64),
    Disk(u64),
}

const MEM_DISK_MASK: u64 = 1 << 63;

impl From<u64> for PagePtr {
    fn from(addr: u64) -> Self {
        assert!(addr != 0);
        if addr & MEM_DISK_MASK == 0 {
            Self::Mem(addr)
        } else {
            Self::Disk(addr & !MEM_DISK_MASK)
        }
    }
}

impl<'a> Into<u64> for PagePtr {
    fn into(self) -> u64 {
        match self {
            Self::Mem(addr) => addr,
            Self::Disk(addr) => addr | MEM_DISK_MASK,
        }
    }
}

pub struct PageBuf {
    page: PageInner,
    size: usize,
}

impl PageBuf {
    pub unsafe fn new(ptr: *mut u8, size: usize) -> Self {
        Self {
            page: (ptr as u64).into(),
            size,
        }
    }

    pub unsafe fn into_raw(self) -> *mut u8 {
        self.page.0 as *mut u8
    }

    pub fn ver(&self) -> u64 {
        self.page.ver()
    }

    pub fn set_ver(&mut self, ver: u64) {
        self.page.set_ver(ver);
    }

    pub fn len(&self) -> u8 {
        self.page.len()
    }

    pub fn set_len(&mut self, len: u8) {
        self.page.set_len(len);
    }

    pub fn kind(&self) -> PageKind {
        self.page.kind().into()
    }

    pub fn set_kind(&mut self, kind: PageKind) {
        self.page.set_kind(kind as u8);
    }

    pub fn set_next(&mut self, next: PagePtr) {
        self.page.set_next(next.into());
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn as_ptr(&self) -> PagePtr {
        self.page.into()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PageRef<'a> {
    page: PageInner,
    mark: PhantomData<&'a ()>,
}

impl<'a> PageRef<'a> {
    fn new(page: PageInner) -> Self {
        Self {
            page,
            mark: PhantomData,
        }
    }

    pub fn ver(&self) -> u64 {
        self.page.ver()
    }

    pub fn len(&self) -> u8 {
        self.page.len()
    }

    pub fn kind(&self) -> PageKind {
        self.page.kind().into()
    }

    pub fn next(&self) -> Option<PagePtr> {
        let ptr = self.page.next();
        if ptr == 0 {
            None
        } else {
            Some(ptr.into())
        }
    }
}

impl<'a> From<u64> for PageRef<'a> {
    fn from(ptr: u64) -> Self {
        Self::new(ptr.into())
    }
}

impl<'a> Into<u64> for PageRef<'a> {
    fn into(self) -> u64 {
        self.page.into()
    }
}

impl<'a> Into<PagePtr> for PageRef<'a> {
    fn into(self) -> PagePtr {
        self.page.into()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PageKind {
    Data = 0,
    Index = 1,
}

impl PageKind {
    pub fn is_data(self) -> bool {
        self < Self::Index
    }
}

impl From<u8> for PageKind {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Data,
            1 => Self::Index,
            _ => panic!("invalid page kind"),
        }
    }
}

// Page format: | ver (6B) | len (1B) | kind (1B) | next (8B) |

#[derive(Copy, Clone, Debug)]
struct PageInner(u64);

impl PageInner {
    fn ver(&self) -> u64 {
        unsafe {
            let ptr = self.0 as *const u64;
            ptr.read() >> 16
        }
    }

    fn set_ver(&mut self, ver: u64) {
        unsafe {
            let ptr = self.0 as *mut u64;
            ptr.write(ver << 16 | (self.len() as u64) << 8 | self.kind() as u64);
        }
    }

    fn len(&self) -> u8 {
        unsafe {
            let ptr = self.0 as *const u8;
            ptr.add(6).read()
        }
    }

    fn set_len(&mut self, len: u8) {
        unsafe {
            let ptr = self.0 as *mut u8;
            ptr.add(6).write(len);
        }
    }

    fn kind(&self) -> u8 {
        unsafe {
            let ptr = self.0 as *const u8;
            ptr.add(7).read()
        }
    }

    fn set_kind(&mut self, kind: u8) {
        unsafe {
            let ptr = self.0 as *mut u8;
            ptr.add(7).write(kind);
        }
    }

    fn next(&self) -> u64 {
        unsafe {
            let ptr = self.0 as *const u64;
            ptr.add(1).read()
        }
    }

    fn set_next(&mut self, next: u64) {
        unsafe {
            let ptr = self.0 as *mut u64;
            ptr.add(1).write(next);
        }
    }
}

impl From<u64> for PageInner {
    fn from(ptr: u64) -> Self {
        Self(ptr)
    }
}

impl Into<u64> for PageInner {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<PagePtr> for PageInner {
    fn into(self) -> PagePtr {
        PagePtr::Mem(self.0)
    }
}
