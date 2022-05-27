#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator};
use std::iter::Enumerate;

trait Computation {
    type Item;
    type MinSequential;
    type Enumerated;

    fn compute<O: Fn(Self::Item) + Sync + Send>(self, op: O);
    fn with_min_sequential(self, size: usize) -> Self::MinSequential;
    fn enumerate(self) -> Self::Enumerated;
}

trait IntoComputation {
    type Item;
    type Computation: Computation<Item = Self::Item>;

    fn into_computation(self) -> Self::Computation;
}

#[cfg(not(feature = "rayon"))]
impl<I, IIT> IntoComputation for IIT
where
    IIT: IntoIterator<Item = I>,
{
    type Item = I;
    type Computation = SequentialComputation<IIT::IntoIter>;

    fn into_computation(self) -> Self::Computation {
        SequentialComputation(self.into_iter())
    }
}

pub struct SequentialComputation<IT: Iterator>(IT);

impl<IT: Iterator> Computation for SequentialComputation<IT> {
    type Item = IT::Item;
    type MinSequential = Self;
    type Enumerated = SequentialComputation<Enumerate<IT>>;

    fn compute<O: Fn(Self::Item) + Sync + Send>(self, op: O) {
        self.0.for_each(op)
    }

    fn with_min_sequential(self, _: usize) -> Self::MinSequential {
        self
    }

    fn enumerate(self) -> Self::Enumerated {
        SequentialComputation(self.0.enumerate())
    }
}

#[cfg(feature = "rayon")]
impl<I, IIT> IntoComputation for IIT
where
    IIT: IntoParallelIterator<Item = I>,
    <IIT as IntoParallelIterator>::Iter: IndexedParallelIterator,
{
    type Item = I;
    type Computation = RayonComputation<IIT::Iter>;

    fn into_computation(self) -> Self::Computation {
        RayonComputation(self.into_par_iter())
    }
}

#[cfg(feature = "rayon")]
pub struct RayonComputation<IT: IndexedParallelIterator>(IT);

#[cfg(feature = "rayon")]
impl<IT: IndexedParallelIterator> Computation for RayonComputation<IT> {
    type Item = IT::Item;
    type MinSequential = RayonComputation<rayon::iter::MinLen<IT>>;
    type Enumerated = RayonComputation<rayon::iter::Enumerate<IT>>;

    fn compute<O: Fn(Self::Item) + Sync + Send>(self, op: O) {
        self.0.for_each(op)
    }

    fn with_min_sequential(self, min_sequential: usize) -> Self::MinSequential {
        RayonComputation(self.0.with_min_len(min_sequential))
    }

    fn enumerate(self) -> Self::Enumerated {
        RayonComputation(self.0.enumerate())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Computation, IntoComputation};

    #[test]
    #[cfg(not(feature = "rayon"))]
    fn test_sequential() {
        let a: Vec<i32> = (0..100).collect();
        a.into_computation()
            .enumerate()
            .with_min_sequential(2)
            .compute(|item| {
                println!("seq: {:?}", item);
            });
    }

    #[test]
    #[cfg(feature = "rayon")]
    fn test_rayon() {
        let a: Vec<i32> = (0..100).collect();
        a.into_computation()
            .enumerate()
            .with_min_sequential(2)
            .compute(|item| {
                println!("par: {:?}", item);
            })
    }
}
