//! Module providing aggregation indexes for computing aggregate values over collections.

mod count;
pub use count::Count;

mod sum;
pub use sum::Sum;

mod mean;
pub use mean::Mean;

mod stddev;
pub use stddev::StdDev;

mod boolean;
pub use boolean::Boolean;

mod generic;
pub use generic::{GenericAggregate, MonoidalAggregate};
