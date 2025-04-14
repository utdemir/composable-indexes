//! Module providing aggregation indexes for computing aggregate values over collections.
//! Includes common aggregations like count, sum, and mean.

mod stats;
pub use stats::*;

mod generic;
pub use generic::AggregateIndex;
