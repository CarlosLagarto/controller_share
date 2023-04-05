use rustc_hash::FxHashSet;

/// Dimension = 64 bytes + mais o que estiver em cache no heap
pub struct CacheFx {
    pub set: FxHashSet<String>,
    pub order: Vec<String>,    
    pub capacity: u8,          
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[repr(u8)]
#[derive(PartialEq)]
pub enum CacheResult {
    Ok,
    Duplicate,
}

impl CacheFx {
    #[allow(dead_code)] //Pensei em retirar antes de ir para produção, mas são 4 funções e o API fica assim completo para referência futura.
    #[inline]
    #[rustfmt::skip]
    pub fn contains(&self, elem: &str) -> bool { self.set.contains(elem) }

    #[inline]
    fn insert(&mut self, elem: &str) {
        self.set.insert(elem.to_owned());
        self.order.push(elem.to_owned());
    }

    #[allow(dead_code)] //Pensei em retirar antes de ir para produção, mas são 4 funções e o API fica assim completo para referência futura.
    #[inline]
    #[rustfmt::skip]
    pub fn is_empty(&self) -> bool { self.order.is_empty() }

    #[allow(dead_code)] //Pensei em retirar antes de ir para produção, mas são 4 funções e o API fica assim completo para referência futura.
    #[inline]
    #[rustfmt::skip]
    pub fn len(&self) -> usize { self.order.len() }

    #[allow(dead_code)] //Pensei em retirar antes de ir para produção, mas são 4 funções e o API fica assim completo para referência futura.
    #[inline]
    pub fn pop_oldest(&mut self) {
        if !self.order.is_empty() {
            let key = &self.order[0];
            self.set.remove(key);
            self.order.remove(0);
        }
    }

    #[inline]
    pub fn push(&mut self, elem: &str) -> CacheResult {
        if !self.set.contains(elem) {
            if self.order.len() < self.capacity as usize {
                self.insert(elem);
                CacheResult::Ok
            } else {
                let key = &self.order[0];
                self.set.remove(key);
                self.order.remove(0);
                self.insert(elem);
                CacheResult::Ok
            }
        } else {
            CacheResult::Duplicate
        }
    }

    #[inline]
    pub fn with_capacity(capacity: u8) -> CacheFx {
        CacheFx { set: FxHashSet::default(), order: Vec::with_capacity(capacity as usize), capacity }
    }
}
