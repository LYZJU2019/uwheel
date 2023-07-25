use super::{entry::TopNEntry, state::TopNState, KeyBounds};
use crate::Aggregator;
use core::{cmp::Ordering, fmt::Debug};
use hashbrown::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BinaryHeap;
#[cfg(feature = "std")]
use std::collections::BinaryHeap;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(bound = "A: Default"))]
#[derive(Clone, Debug, Default)]
pub struct TopNMap<Key, A>
where
    Key: KeyBounds,
    A: Aggregator,
    A::PartialAggregate: Ord,
{
    table: HashMap<Key, A::PartialAggregate>,
}

impl<Key, A> TopNMap<Key, A>
where
    Key: KeyBounds,
    A: Aggregator,
    A::PartialAggregate: Ord,
{
    #[inline]
    pub fn insert(&mut self, key: Key, delta: A::PartialAggregate) {
        self.table
            .entry(key)
            .and_modify(|curr_delta| {
                *curr_delta = A::combine(*curr_delta, delta);
            })
            .or_insert(delta);
    }
    pub fn to_state<const N: usize>(mut self, order: Ordering) -> TopNState<Key, N, A> {
        let mut heap = BinaryHeap::with_capacity(N);

        // build Top N state from table
        for (key, agg) in self.table.drain() {
            let entry = TopNEntry::new(key, agg);
            if heap.len() < N {
                heap.push(entry);
            } else if let Some(top) = heap.peek() {
                if entry.cmp(top) == order {
                    let _ = heap.pop();
                    heap.push(entry);
                }
            }
        }

        let mut sorted_vec: Vec<_> = heap.into_sorted_vec().into_iter().map(Some).collect();

        // fill remainder if needed...
        let rem = N - sorted_vec.len();
        for _i in 0..rem {
            sorted_vec.push(None);
        }

        TopNState::from(sorted_vec)
    }
}