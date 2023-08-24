use std::collections::{HashSet, VecDeque};

// A bounded set is made by combining a hash set with a VecDeque.
pub struct FIFOCache<T> {
	set: HashSet<T>,
	queue: VecDeque<T>,
	limit: usize,
}

impl<T: Clone + Eq + std::hash::Hash> FIFOCache<T> {
	pub fn new(limit: usize) -> Self {
		FIFOCache {
			set: HashSet::new(),
			queue: VecDeque::new(),
			limit,
		}
	}

	pub fn insert(&mut self, value: T) {
		// Check if the value is already in the set
		if self.set.contains(&value) {
			return;
		}

		// Check if the cache is full and remove the oldest element if necessary
		if self.queue.len() == self.limit {
			if let Some(removed_value) = self.queue.pop_front() {
				self.set.remove(&removed_value);
			}
		}

		// Insert the new value into both the set and the queue
		self.set.insert(value.clone());
		self.queue.push_back(value);
	}

	pub fn contains(&self, value: &T) -> bool {
		self.set.contains(value)
	}

	// Other methods as needed
}
