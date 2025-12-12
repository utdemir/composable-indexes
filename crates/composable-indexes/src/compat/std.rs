pub type DefaultHashBuilder = std::hash::RandomState;

pub type HashMap<K, V, S = DefaultHashBuilder> = std::collections::HashMap<K, V, S>;
pub type HashSet<K, S = DefaultHashBuilder> = std::collections::HashSet<K, S>;
