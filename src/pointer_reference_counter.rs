use std::collections::HashMap;
use std::hash::Hash;

pub trait Destroy {
    fn destroy(&self);
}

pub struct PointerReferenceCounter<P: Destroy + Hash + Eq> {
    map: HashMap<P, usize>,
}

impl<P: Destroy + Hash + Eq> PointerReferenceCounter<P> {
    pub fn new() -> Self {
        PointerReferenceCounter {
            map: HashMap::new(),
        }
    }

    pub fn incr(&mut self, ptr: P) {
        let entry = self.map.entry(ptr).or_insert(0);
        *entry += 1;
    }

    pub fn decr(&mut self, ptr: P) {
        let count = self.map.get(&ptr).expect("too many dereferences");
        if *count == 1 {
            self.map.remove(&ptr);
            ptr.destroy();
        } else {
            *count -= 1;
        }
    }
}
