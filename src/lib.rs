#![feature(test)]
extern crate test;
use rstar::{AABB, Envelope, RTreeObject, SelectionFunction};

#[derive(Clone, Debug, Copy)]
pub struct Range(pub i32, pub i32);

impl RTreeObject for Range
{
    type Envelope = AABB<(i32, i32)>;
    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners((self.0, 0), (self.1, 0))
    }
}

pub struct RangeSelector(pub Range);

impl SelectionFunction<Range> for RangeSelector {
    fn should_unpack_parent(&self, envelope: &<Range as RTreeObject>::Envelope) -> bool {
        envelope.contains_envelope(&self.0.envelope())
    }

    fn should_unpack_leaf(&self, leaf: &Range) -> bool {
        leaf.envelope().contains_envelope(&self.0.envelope())
    }
}

#[cfg(test)]
mod tests {
    use std::cmp;
    use test::{Bencher, black_box};

    use rand::Rng;
    use rand_chacha::ChaCha8Rng;
    use rand_chacha::rand_core::SeedableRng;
    use rstar::{Envelope, RTree, RTreeObject};

    use crate::{Range, RangeSelector};

    const NUM_RANGES: i32 = 1000;

    fn get_insert_ranges() -> Vec<Range> {
        get_ranges(31)
    }

    fn get_lookup_ranges() -> Vec<Range> {
        get_ranges(64)
    }

    fn get_ranges(seed: u64) -> Vec<Range> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        (0..NUM_RANGES).into_iter().map(|_| {
            let chunk_start = rng.gen_range(0..64000);
            let chunk_size = rng.gen_range(0..64);
            let chunk_end = cmp::min(chunk_start + chunk_size, 64000);
            Range(chunk_start, chunk_end)
        }).collect::<Vec<_>>()
    }

    #[bench]
    fn benchmark_rtree_insert(b: &mut Bencher) {
        let ranges = get_insert_ranges();
        let mut rtree = RTree::new();
        b.iter(|| {
            for range in &ranges {
                black_box(rtree.insert(range.clone()));
            }
        })
    }

    #[bench]
    fn benchmark_rtree_bulk_insert(b: &mut Bencher) {
        let ranges = get_insert_ranges();
        b.iter(|| {
            black_box(RTree::bulk_load(ranges.clone()));
        })
    }


    #[bench]
    fn benchmark_rtree_lookup(b: &mut Bencher) {
        let ranges = get_insert_ranges();
        let mut rtree = RTree::new();
        for range in &ranges {
            rtree.insert(range.clone());
        }
        let lookup_ranges = get_lookup_ranges();
        b.iter(|| {
            for range in &lookup_ranges {
                black_box(rtree.locate_with_selection_function(RangeSelector(range.clone())));
            }
        })
    }

    #[bench]
    fn benchmark_list_insert(b: &mut Bencher) {
        let ranges = get_insert_ranges();
        let mut list = vec![];
        b.iter(|| {
            for range in &ranges {
                black_box(list.push(range.clone()));
            }
        })
    }

    #[bench]
    fn benchmark_list_bulk_insert(b: &mut Bencher) {
        let ranges = get_insert_ranges();
        b.iter(|| {
            let _ = black_box(Vec::from(ranges.clone()));
        })
    }

    #[bench]
    fn benchmark_list_lookup(b: &mut Bencher) {
        let ranges = get_insert_ranges();
        let mut list = vec![];
        for range in &ranges {
            list.push(range.clone());
        }
        let lookup_ranges = get_lookup_ranges();
        b.iter(|| {
            for range in &lookup_ranges {
                black_box(list.iter().find(|r| r.envelope().contains_envelope(&range.envelope())));
            }
        })
    }
}
