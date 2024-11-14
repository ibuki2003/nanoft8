const HASH_BITS: usize = 10;

pub const fn table_size(n: usize) -> usize {
    n << HASH_BITS
}

// intended to store Callsign-Hash pairs
// capacity = N * 1024
// M is the max address shift, i.e. the number of entries with the same hash
// generally, M should be greater than N
#[derive(Debug)]
pub struct HashTable<T: Sized, const N: usize, const M: usize>
where
    [T; table_size(N)]: Sized,
{
    table: [Option<HashTableEntry<T>>; table_size(N)],
    count: usize,
    gen: u32,
}

impl<T: Sized, const N: usize, const M: usize> HashTable<T, N, M>
where
    [T; table_size(N)]: Sized,
{
    const _ASSERT: () = {
        assert!(M > N, "M greater than N is useless");
        assert!(N > 0, "N must be greater than 0");
    };

    pub const SIZE: usize = table_size(N);

    pub const fn new() -> Self {
        #![allow(path_statements)]
        Self::_ASSERT;
        Self {
            table: [const { none() }; table_size(N)],
            count: 0,
            gen: 1,
        }
    }

    const fn idx(key: u32) -> usize {
        (key as usize) >> (22 - 10)
    }

    pub fn set(&mut self, key: u32, value: T) {
        let base = Self::idx(key);

        let mut idx = base;
        let mut v = self.table[base].as_ref().map_or(0, |x| x.gen);

        for i in 0..M {
            let d = (base + i) % Self::SIZE;
            let entry = self.table[d].as_mut();
            match entry {
                Some(entry) if entry.key == key => {
                    // update now
                    entry.value = value;
                    entry.gen = self.gen;
                    self.gen = self.gen.wrapping_add(1);
                    if self.gen == 0 {
                        self.gen = 1;
                    }
                    return;
                }
                None => {
                    if v > 0 {
                        idx = d;
                        v = 0;
                    }
                }
                Some(entry) => {
                    // TODO: check for wrap-around
                    if entry.gen < v {
                        v = entry.gen;
                        idx = d;
                    }
                }
            }
        }

        if self.table[idx].is_none() {
            self.count += 1;
        }

        self.table[idx] = Some(HashTableEntry {
            key,
            value,
            gen: self.gen,
        });
        self.gen = self.gen.wrapping_add(1);
        if self.gen == 0 {
            self.gen = 1;
        }
    }

    pub fn get(&self, key: u32) -> Option<&T> {
        let base = Self::idx(key);
        for entry in (0..M).filter_map(|i| self.table[(base + i) % Self::SIZE].as_ref()) {
            if entry.key == key {
                return Some(&entry.value);
            }
        }
        None
    }

    // returns all entries with the same hash
    pub fn get_partial(&self, key: u32) -> impl Iterator<Item = (&u32, &T)> {
        let base = Self::idx(key);
        (0..M)
            .filter_map(move |i| self.table[(base + i) % Self::SIZE].as_ref())
            .map(|entry| (&entry.key, &entry.value))
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl<T, const N: usize, const M: usize> Default for HashTable<T, N, M>
where
    [T; table_size(N)]: Sized,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct HashTableEntry<T> {
    key: u32,
    value: T,
    gen: u32,
}

const fn none<T>() -> Option<T> {
    None
}
