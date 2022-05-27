/// Extends iterable collections with a function to create a [`Computation`].
///
/// If the `rayon` feature is enabled, this will implemented and be implemented for `IntoParallelIterator`.
///
/// Otherwise, this will implement and be implemented for `IntoIterator`.
pub trait IntoComputation {
    type Item;
    #[cfg(not(feature = "rayon"))]
    type Iter: Iterator;
    #[cfg(feature = "rayon")]
    type Iter: rayon::iter::IndexedParallelIterator;

    fn into_computation(self) -> Computation<Self::Iter>;
}

/// An iterator that may be sequential or parallel depending on feature flags.
#[cfg(not(feature = "rayon"))]
#[repr(transparent)]
pub struct Computation<IT: Iterator>(IT);

#[cfg(not(feature = "rayon"))]
impl<I, IIT> IntoComputation for IIT
where
    IIT: IntoIterator<Item = I>,
{
    type Item = I;
    type Iter = IIT::IntoIter;

    fn into_computation(self) -> Computation<Self::Iter> {
        Computation(self.into_iter())
    }
}

#[cfg(not(feature = "rayon"))]
impl<IT: Iterator> Computation<IT> {
    /// Do a computation on all items.
    pub fn compute<O: Fn(IT::Item)>(self, op: O) {
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
    pub fn enumerate(self) -> Computation<std::iter::Enumerate<IT>> {
        Computation(self.0.enumerate())
    }

    /// Like iterator mapping.
    pub fn map<O, M: Fn(IT::Item) -> O>(self, map: M) -> Computation<std::iter::Map<IT, M>> {
        Computation(self.0.map(map))
    }

    /// Like iterator flat-mapping.
    pub fn flat_map<O: IntoIterator, M: Fn(IT::Item) -> O>(
        self,
        map: M,
    ) -> Computation<std::iter::FlatMap<IT, O, M>> {
        Computation(self.0.flat_map(map))
    }

    /// Get the inner iterator.
    pub fn into_inner(self) -> IT {
        self.0
    }
}

#[cfg(not(feature = "rayon"))]
impl<IT: Iterator> IntoIterator for Computation<IT> {
    type Item = IT::Item;
    type IntoIter = IT;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner()
    }
}

/// An iterator that may be sequential or parallel depending on feature flags.
#[cfg(feature = "rayon")]
#[repr(transparent)]
pub struct Computation<IT: rayon::iter::ParallelIterator>(IT);

#[cfg(feature = "rayon")]
impl<I, IIT> IntoComputation for IIT
where
    IIT: rayon::iter::IntoParallelIterator<Item = I>,
    <IIT as rayon::iter::IntoParallelIterator>::Iter: rayon::iter::IndexedParallelIterator,
{
    type Item = I;
    type Iter = IIT::Iter;

    fn into_computation(self) -> Computation<Self::Iter> {
        Computation(self.into_par_iter())
    }
}

#[cfg(feature = "rayon")]
impl<IT: rayon::iter::ParallelIterator> Computation<IT> {
    /// Do a computation on all items.
    pub fn compute<O: Fn(IT::Item) + Sync + Send>(self, op: O) {
        self.0.for_each(op)
    }

    /// Like iterator mapping.
    pub fn map<O: Send, M: Fn(IT::Item) -> O + Send + Sync>(
        self,
        map: M,
    ) -> Computation<rayon::iter::Map<IT, M>> {
        Computation(self.0.map(map))
    }

    /// Like iterator flat mapping.
    pub fn flat_map<O: rayon::iter::IntoParallelIterator, M: Fn(IT::Item) -> O + Send + Sync>(
        self,
        map: M,
    ) -> Computation<rayon::iter::FlatMap<IT, M>> {
        Computation(self.0.flat_map(map))
    }

    /// Get the inner parallel iterator.
    pub fn into_inner(self) -> IT {
        self.0
    }
}

#[cfg(feature = "rayon")]
impl<IT: rayon::iter::IndexedParallelIterator> Computation<IT> {
    pub fn enumerate(self) -> Computation<rayon::iter::Enumerate<IT>> {
        Computation(self.0.enumerate())
    }

    /// Process at least this many items sequentially (no-op unless `rayon` feature enabled).
    pub fn with_min_sequential(
        self,
        min_sequential: usize,
    ) -> Computation<rayon::iter::MinLen<IT>> {
        Computation(self.0.with_min_len(min_sequential))
    }
}

#[cfg(feature = "rayon")]
impl<IT: rayon::iter::ParallelIterator> rayon::iter::ParallelIterator for Computation<IT> {
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

#[cfg(test)]
mod tests {
    use crate::IntoComputation;

    #[test]
    #[cfg(not(feature = "rayon"))]
    fn test_sequential() {
        let a: Vec<i32> = (0..100).collect();
        a.into_computation()
            .with_min_sequential(2)
            .map(|n| -n)
            .enumerate()
            .flat_map(|(e, n)| vec![e as i32, n, n + 1000].into_computation())
            .compute(|item| {
                println!("par: {:?}", item);
            })
    }

    #[test]
    #[cfg(feature = "rayon")]
    fn test_rayon() {
        let a: Vec<i32> = (0..100).collect();
        a.into_computation()
            .with_min_sequential(2)
            .map(|n| -n)
            .enumerate()
            .flat_map(|(e, n)| vec![e as i32, n, n + 1000].into_computation())
            .compute(|item| {
                println!("par: {:?}", item);
            })
    }
}
