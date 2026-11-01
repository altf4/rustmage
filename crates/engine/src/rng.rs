/// Per-state SplitMix64. Cloning the state also clones all future randomness.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rng(pub u64);

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, bound: u64) -> u64 {
        assert!(bound > 0);
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let x = self.next();
            if x >= threshold {
                return x % bound;
            }
        }
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            items.swap(i, self.below((i + 1) as u64) as usize);
        }
    }
}
