mod iter;
pub(crate) use iter::{
    ItemIter, MergingIter, MergingIterBuilder, RewindableIterator, SeekableIterator, SliceIter,
};

mod data;
pub(crate) use data::{Index, Key, Range, Value};

mod codec;

mod base_page;
use base_page::PageBuilder;
pub(crate) use base_page::{PageBuf, PageKind, PageRef, PageTier};

mod sorted_page;
pub(crate) use sorted_page::{
    SortedPageBuilder, SortedPageIter, SortedPageKey, SortedPageRef, SortedPageValue,
};

pub(crate) type ValuePageRef<'a> = SortedPageRef<'a, Key<'a>, Value<'a>>;
pub(crate) type IndexPageRef<'a> = SortedPageRef<'a, &'a [u8], Index>;

#[cfg(test)]
mod tests {
    use std::{
        alloc::{alloc, Layout},
        marker::PhantomData,
        slice,
    };

    use super::*;

    pub(crate) fn alloc_page(size: usize) -> Box<[u8]> {
        let layout = Layout::from_size_align(size, 8).unwrap();
        unsafe {
            let ptr = alloc(layout);
            let buf = slice::from_raw_parts_mut(ptr, layout.size());
            Box::from_raw(buf)
        }
    }

    pub(crate) struct OwnedSortedPage<K, V> {
        buf: Box<[u8]>,
        _marker: PhantomData<(K, V)>,
    }

    impl<K, V> OwnedSortedPage<K, V>
    where
        K: SortedPageKey,
        V: SortedPageValue,
    {
        fn new(buf: Box<[u8]>) -> Self {
            Self {
                buf,
                _marker: PhantomData,
            }
        }

        pub(crate) fn from_iter<I>(iter: I) -> Self
        where
            I: RewindableIterator<Item = (K, V)>,
        {
            let builder = SortedPageBuilder::new(PageTier::Leaf, PageKind::Data).with_iter(iter);
            let mut buf = alloc_page(builder.size());
            let mut page = PageBuf::new(buf.as_mut());
            builder.build(&mut page);
            Self::new(buf)
        }

        pub(crate) fn from_item(item: (K, V)) -> Self {
            Self::from_iter(ItemIter::new(item))
        }

        pub(crate) fn from_slice(data: &[(K, V)]) -> Self {
            Self::from_iter(SliceIter::new(data))
        }

        pub(crate) fn as_ref(&self) -> SortedPageRef<'_, K, V> {
            self.buf.as_ref().into()
        }
    }
}
