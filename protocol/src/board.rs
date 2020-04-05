use crate::types::*;

#[cfg(feature = "sorted")]
type Map<K, V> = std::collections::BTreeMap<K, V>;

#[cfg(not(feature = "sorted"))]
type Map<K, V> = std::collections::HashMap<K, V>;

pub struct Board {
    pub sessions: Map<Username, bool>,
}
