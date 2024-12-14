use std::cmp::PartialEq;
use std::fmt::Debug;
use std::time::Instant;

pub trait Hashable {
    fn hash(&self) -> usize;
}

impl Hashable for String {
    // http://www.cse.yorku.ca/~oz/hash.html
    fn hash(&self) -> usize {
        let mut hash: usize = 5381;
        for c in self.bytes() {
            hash = (hash << 5).wrapping_add(hash).wrapping_add(c.into()); /* hash * 33 + c */
        }
        hash
    }
}

impl Hashable for usize {
    fn hash(&self) -> usize {
        *self
    }
}

#[derive(Debug, Default, Clone)]
struct HashCell<K, V> {
    key: K,
    value: V,
    taken: bool,
}

#[derive(Debug)]
pub struct HashTable<K, V> {
    cells: Vec<HashCell<K, V>>,
    taken_count: usize,
}

impl<K, V> HashTable<K, V>
where
    K: Default + Clone + Hashable + PartialEq + Debug,
    V: Default + Clone + Debug,
{
    pub fn new() -> Self {
        const DEFAULT_VEC_CAPACITY: usize = 61; // prime in order to have good splits
        Self::with_capacity(DEFAULT_VEC_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cells: vec![HashCell::<K, V>::default(); capacity],
            taken_count: 0,
        }
    }

    fn extend(&mut self) {
        assert_eq!(self.taken_count, self.cells.len());
        assert_ne!(self.cells.len(), 0);

        let mut new_self = Self {
            cells: vec![HashCell::<K, V>::default(); self.cells.len() * 2 + 1],
            taken_count: 0,
        };

        for cell in self.cells.iter() {
            if cell.taken {
                new_self.insert(cell.key.clone(), cell.value.clone());
            }
        }

        *self = new_self;
    }

    pub fn insert(&mut self, key: K, new_value: V) {
        if let Some(old_val) = self.get_mut(&key) {
            *old_val = new_value;
            return;
        }

        if self.taken_count == self.cells.len() {
            self.extend();
        }

        assert!(self.cells.len() > self.taken_count);

        let mut idx = key.hash() % self.cells.len();
        while self.cells[idx].taken {
            idx = (idx + 1) % self.cells.len();
        }

        self.cells[idx].key = key;
        self.cells[idx].value = new_value;
        self.cells[idx].taken = true;
        self.taken_count += 1;
    }

    fn get_index(&self, key: &K) -> Option<usize> {
        let mut idx = key.hash() % self.cells.len();
        for _ in 0..self.cells.len() {
            if !self.cells[idx].taken {
                break;
            }

            if self.cells[idx].key == *key {
                return Some(idx);
            }

            idx = (idx + 1) % self.cells.len();
        }

        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some(idx) = self.get_index(key) {
            Some(&self.cells[idx].value)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(idx) = self.get_index(key) {
            Some(&mut self.cells[idx].value)
        } else {
            None
        }
    }

    pub fn debug_dump(&self) {
        println!("----------------------------------------------------------");
        println!("  Table Len {}", self.cells.len());
        println!("  Taken Count {}", self.taken_count);
        println!("  Data");
        for (i, c) in self.cells.iter().enumerate() {
            if c.taken {
                println!("    ({})      {:?} => {:?}", i, c.key, c.value);
            } else {
                println!("    ({})      X", i);
            }
        }
        println!("----------------------------------------------------------");
    }
}

fn benchmark_our_vergin_table(n: usize) {
    let start = Instant::now();

    let mut table = HashTable::<usize, usize>::new();
    for _ in 0..n {
        let n = rand::random::<usize>();
        if let Some(val) = table.get_mut(&n) {
            *val += 1; // this might overflow
        } else {
            table.insert(n, n);
        }
        assert_eq!(*table.get(&n).unwrap(), n);
    }

    let duration = start.elapsed();
    println!("Time elapsed in in vergin table is: {:?}", duration);
}

fn benchmark_std_chad_table(n: usize) {
    let start = Instant::now();

    let mut table = std::collections::HashMap::<usize, usize>::new();
    for _ in 0..n {
        let n = rand::random::<usize>();
        if let Some(val) = table.get_mut(&n) {
            *val += 1; // this might overflow
        } else {
            table.insert(n, n);
        }
        assert_eq!(*table.get(&n).unwrap(), n);
    }

    let duration = start.elapsed();
    println!("Time elapsed in in chad table is: {:?}", duration);
}

fn main() {
    let mut table = HashTable::<String, String>::with_capacity(11);
    for i in 0..11 {
        table.insert(format!("{}", i), format!("{}", 100_000 + i));
    }

    // table.debug_dump();
    table.insert("69".to_string(), "69".to_string());
    // table.debug_dump();

    for i in 0..11 {
        let key = format!("{}", i);
        assert_eq!(*table.get(&key).unwrap(), format!("{}", 100_000 + i));
    }

    assert_eq!(*table.get(&("69".to_string())).unwrap(), "69".to_string());

    const N: usize = 100_000;

    benchmark_our_vergin_table(N);
    benchmark_std_chad_table(N);
}
