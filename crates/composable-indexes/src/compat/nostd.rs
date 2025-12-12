pub type DefaultHashBuilder = hashbrown::DefaultHashBuilder;

pub type HashMap<K, V, S = DefaultHashBuilder> = hashbrown::HashMap<K, V, S>;
pub type HashSet<K, S = DefaultHashBuilder> = hashbrown::HashSet<K, S>;
