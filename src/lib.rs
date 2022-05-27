use std::cmp::Ordering;

/// Extends iterable collections with a function to create a [`Computation`].
///
/// If the `rayon` feature is enabled, this will implemented and be implemented for `IntoParallelIterator`.
///
/// Otherwise, this will implement and be implemented for `IntoIterator`.
pub trait IntoMaybeParallelIterator {
    type Item;
    #[cfg(not(feature = "rayon"))]
    type Iter: Iterator<Item = Self::Item>;
    #[cfg(feature = "rayon")]
    type Iter: rayon::iter::ParallelIterator<Item = Self::Item>;

    fn into_maybe_parallel_iter(self) -> MaybeParallelIterator<Self::Iter>;
}

/// Like [`IntoMaybeParallelIterator`] but borrows.
pub trait IntoMaybeParallelRefIterator<'a> {
    #[cfg(not(feature = "rayon"))]
    type Iter: Iterator;
    #[cfg(feature = "rayon")]
    type Iter: rayon::iter::ParallelIterator;

    fn maybe_par_iter(&'a self) -> MaybeParallelIterator<Self::Iter>;
}

/// Like [`IntoMaybeParallelIterator`] but borrows mutably.
pub trait IntoMaybeParallelRefMutIterator<'a> {
    #[cfg(not(feature = "rayon"))]
    type Iter: Iterator;
    #[cfg(feature = "rayon")]
    type Iter: rayon::iter::ParallelIterator;

    fn maybe_par_iter_mut(&'a mut self) -> MaybeParallelIterator<Self::Iter>;
}

/// An iterator that may be sequential or parallel depending on feature flags.
#[cfg(not(feature = "rayon"))]
#[repr(transparent)]
pub struct MaybeParallelIterator<IT: Iterator>(IT);

#[cfg(not(feature = "rayon"))]
impl<IIT: IntoIterator> IntoMaybeParallelIterator for IIT {
    type Item = IIT::Item;
    type Iter = IIT::IntoIter;

    fn into_maybe_parallel_iter(self) -> MaybeParallelIterator<Self::Iter> {
        MaybeParallelIterator(self.into_iter())
    }
}

#[cfg(not(feature = "rayon"))]
impl<'a, IIT: 'a> IntoMaybeParallelRefIterator<'a> for IIT
where
    &'a IIT: IntoIterator,
{
    type Iter = <&'a IIT as IntoIterator>::IntoIter;

    fn maybe_par_iter(&'a self) -> MaybeParallelIterator<Self::Iter> {
        MaybeParallelIterator(self.into_iter())
    }
}

#[cfg(not(feature = "rayon"))]
impl<'a, IIT: 'a> IntoMaybeParallelRefMutIterator<'a> for IIT
where
    &'a mut IIT: IntoIterator,
{
    type Iter = <&'a mut IIT as IntoIterator>::IntoIter;

    fn maybe_par_iter_mut(&'a mut self) -> MaybeParallelIterator<Self::Iter> {
        MaybeParallelIterator(self.into_iter())
    }
}

#[cfg(not(feature = "rayon"))]
impl<IT: Iterator> MaybeParallelIterator<IT> {
    /// Do a computation on all items.
    pub fn for_each<O: Fn(IT::Item)>(self, op: O) {
        self.0.for_each(op)
    }

    /// Process at least this many items sequentially (no-op unless `rayon` feature enabled).
    ///
    /// # Foot-gun
    ///
    /// Without `rayon` feature, this can be called in places `IndexedParallelIterator` would not
    /// apply. These uses won't compile under `rayon`.
    pub fn with_min_sequential(self, _: usize) -> Self {
        self
    }

    /// Map to a tuple containing the index of each item along with that item.
    ///
    /// # Foot-gun
    ///
    /// Without `rayon` feature, this can be called in places `IndexedParallelIterator` would not
    /// apply. These uses won't compile under `rayon`.
    pub fn enumerate(self) -> MaybeParallelIterator<std::iter::Enumerate<IT>> {
        MaybeParallelIterator(self.0.enumerate())
    }

    /// Like iterator mapping.
    pub fn map<O, M: Fn(IT::Item) -> O>(
        self,
        map: M,
    ) -> MaybeParallelIterator<std::iter::Map<IT, M>> {
        MaybeParallelIterator(self.0.map(map))
    }

    /// Like iterator flat-mapping.
    pub fn flat_map<O: IntoIterator, M: Fn(IT::Item) -> O>(
        self,
        map: M,
    ) -> MaybeParallelIterator<std::iter::FlatMap<IT, O, M>> {
        MaybeParallelIterator(self.0.flat_map(map))
    }

    /// Like iterator find (but won't necessarily return the first match).
    pub fn find_any<F: Fn(&IT::Item) -> bool>(mut self, f: F) -> Option<IT::Item> {
        self.0.find(f)
    }

    /// Get the inner iterator.
    pub fn into_inner(self) -> IT {
        self.0
    }
}

#[cfg(not(feature = "rayon"))]
impl<IT: Iterator> IntoIterator for MaybeParallelIterator<IT> {
    type Item = IT::Item;
    type IntoIter = IT;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner()
    }
}

/// An iterator that may be sequential or parallel depending on feature flags.
#[cfg(feature = "rayon")]
#[repr(transparent)]
pub struct MaybeParallelIterator<IT: rayon::iter::ParallelIterator>(IT);

#[cfg(feature = "rayon")]
impl<IIT> IntoMaybeParallelIterator for IIT
where
    IIT: rayon::iter::IntoParallelIterator,
    <IIT as rayon::iter::IntoParallelIterator>::Iter: rayon::iter::IndexedParallelIterator,
{
    type Item = IIT::Item;
    type Iter = IIT::Iter;

    fn into_maybe_parallel_iter(self) -> MaybeParallelIterator<Self::Iter> {
        MaybeParallelIterator(self.into_par_iter())
    }
}

#[cfg(feature = "rayon")]
impl<'a, IIT: 'a> IntoMaybeParallelRefIterator<'a> for IIT
where
    IIT: rayon::iter::IntoParallelRefIterator<'a>,
{
    type Iter = IIT::Iter;

    fn maybe_par_iter(&'a self) -> MaybeParallelIterator<Self::Iter> {
        MaybeParallelIterator(self.par_iter())
    }
}

#[cfg(feature = "rayon")]
impl<'a, IIT: 'a> IntoMaybeParallelRefMutIterator<'a> for IIT
where
    IIT: rayon::iter::IntoParallelRefMutIterator<'a>,
{
    type Iter = IIT::Iter;

    fn maybe_par_iter_mut(&'a mut self) -> MaybeParallelIterator<Self::Iter> {
        MaybeParallelIterator(self.par_iter_mut())
    }
}

#[cfg(feature = "rayon")]
impl<IT: rayon::iter::ParallelIterator> MaybeParallelIterator<IT> {
    /// Do a computation on all items.
    pub fn for_each<O: Fn(IT::Item) + Sync + Send>(self, op: O) {
        self.0.for_each(op)
    }

    /// Like iterator mapping.
    pub fn map<O: Send, M: Fn(IT::Item) -> O + Send + Sync>(
        self,
        map: M,
    ) -> MaybeParallelIterator<rayon::iter::Map<IT, M>> {
        MaybeParallelIterator(self.0.map(map))
    }

    /// Like iterator flat mapping.
    pub fn flat_map<O: rayon::iter::IntoParallelIterator, M: Fn(IT::Item) -> O + Send + Sync>(
        self,
        map: M,
    ) -> MaybeParallelIterator<rayon::iter::FlatMap<IT, M>> {
        MaybeParallelIterator(self.0.flat_map(map))
    }

    /// Like iterator find (but won't necessarily return the first match).
    pub fn find_any<F: Fn(&IT::Item) -> bool + Send + Sync>(self, f: F) -> Option<IT::Item> {
        self.0.find_any(f)
    }

    /// Get the inner parallel iterator.
    pub fn into_inner(self) -> IT {
        self.0
    }
}

#[cfg(feature = "rayon")]
impl<IT: rayon::iter::IndexedParallelIterator> MaybeParallelIterator<IT> {
    pub fn enumerate(self) -> MaybeParallelIterator<rayon::iter::Enumerate<IT>> {
        MaybeParallelIterator(self.0.enumerate())
    }

    /// Process at least this many items sequentially (no-op unless `rayon` feature enabled).
    pub fn with_min_sequential(
        self,
        min_sequential: usize,
    ) -> MaybeParallelIterator<rayon::iter::MinLen<IT>> {
        MaybeParallelIterator(self.0.with_min_len(min_sequential))
    }
}

#[cfg(feature = "rayon")]
impl<IT: rayon::iter::ParallelIterator> rayon::iter::ParallelIterator
    for MaybeParallelIterator<IT>
{
    type Item = IT::Item;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        self.0.drive_unindexed(consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        self.0.opt_len()
    }
}

pub trait MaybeParallelSort<T: Send> {
    fn maybe_par_sort(&mut self)
    where
        T: Ord;

    fn maybe_par_sort_unstable(&mut self)
    where
        T: Ord;

    fn maybe_par_sort_by<F>(&mut self, compare: F)
    where
        F: Fn(&T, &T) -> Ordering + Sync;

    fn maybe_par_sort_unstable_by<F>(&mut self, compare: F)
    where
        F: Fn(&T, &T) -> Ordering + Sync;

    fn maybe_par_sort_by_key<F, K: Ord>(&mut self, f: F)
    where
        F: Fn(&T) -> K + Sync;

    fn maybe_par_sort_unstable_by_key<F, K: Ord>(&mut self, f: F)
    where
        F: Fn(&T) -> K + Sync;
}

#[cfg(not(feature = "rayon"))]
impl<T: Send> MaybeParallelSort<T> for [T] {
    fn maybe_par_sort(&mut self)
    where
        T: Ord,
    {
        self.sort()
    }

    fn maybe_par_sort_unstable(&mut self)
    where
        T: Ord,
    {
        self.sort_unstable()
    }

    fn maybe_par_sort_by<F>(&mut self, compare: F)
    where
        F: Fn(&T, &T) -> Ordering + Sync,
    {
        self.sort_by(compare)
    }

    fn maybe_par_sort_unstable_by<F>(&mut self, compare: F)
    where
        F: Fn(&T, &T) -> Ordering + Sync,
    {
        self.sort_unstable_by(compare)
    }

    fn maybe_par_sort_by_key<F, K: Ord>(&mut self, f: F)
    where
        F: Fn(&T) -> K + Sync,
    {
        self.sort_by_key(f)
    }

    fn maybe_par_sort_unstable_by_key<F, K: Ord>(&mut self, f: F)
    where
        F: Fn(&T) -> K + Sync,
    {
        self.sort_unstable_by_key(f)
    }
}

#[cfg(feature = "rayon")]
impl<T: Send, C> MaybeParallelSort<T> for C
where
    C: rayon::slice::ParallelSliceMut<T> + ?Sized,
{
    fn maybe_par_sort(&mut self)
    where
        T: Ord,
    {
        self.par_sort()
    }

    fn maybe_par_sort_unstable(&mut self)
    where
        T: Ord,
    {
        self.par_sort_unstable()
    }

    fn maybe_par_sort_by<F>(&mut self, compare: F)
    where
        F: Fn(&T, &T) -> Ordering + Sync,
    {
        self.par_sort_by(compare)
    }

    fn maybe_par_sort_unstable_by<F>(&mut self, compare: F)
    where
        F: Fn(&T, &T) -> Ordering + Sync,
    {
        self.par_sort_unstable_by(compare)
    }

    fn maybe_par_sort_by_key<F, K: Ord>(&mut self, f: F)
    where
        F: Fn(&T) -> K + Sync,
    {
        self.par_sort_by_key(f)
    }

    fn maybe_par_sort_unstable_by_key<F, K: Ord>(&mut self, f: F)
    where
        F: Fn(&T) -> K + Sync,
    {
        self.par_sort_unstable_by_key(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        IntoMaybeParallelIterator, IntoMaybeParallelRefIterator, IntoMaybeParallelRefMutIterator,
        MaybeParallelSort,
    };

    #[test]
    #[cfg(not(feature = "rayon"))]
    fn test_sequential() {
        let mut a: Vec<i32> = (0..100).collect();
        a.maybe_par_iter().for_each(|item| println!("{}", item));
        a.maybe_par_iter_mut().for_each(|item| *item -= 5);
        println!("{:?}", a);
        a.into_maybe_parallel_iter()
            .with_min_sequential(2)
            .map(|n| -n)
            .enumerate()
            .flat_map(|(e, n)| vec![e as i32, n, n + 1000].into_maybe_parallel_iter())
            .for_each(|item| {
                println!("seq: {:?}", item);
            });

        let mut to_sort: Vec<i32> = vec![5, 2, 2, 6, 1, 6];
        to_sort.maybe_par_sort();
        println!("{:?}", to_sort);
    }

    #[test]
    #[cfg(feature = "rayon")]
    fn test_rayon() {
        let mut a: Vec<i32> = (0..100).collect();
        a.maybe_par_iter()
            .with_min_sequential(5)
            .for_each(|item| println!("{}", item));
        a.maybe_par_iter_mut().for_each(|item| *item -= 5);
        println!("{:?}", a);
        a.into_maybe_parallel_iter()
            .with_min_sequential(2)
            .map(|n| -n)
            .enumerate()
            .flat_map(|(e, n)| vec![e as i32, n, n + 1000].into_maybe_parallel_iter())
            .for_each(|item| {
                println!("par: {:?}", item);
            });

        let mut to_sort: Vec<i32> = vec![5, 2, 2, 6, 1, 6];
        to_sort.maybe_par_sort();
        println!("{:?}", to_sort);
    }
}
