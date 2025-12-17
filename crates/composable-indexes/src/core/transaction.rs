use crate::Collection;

pub struct Transaction<In, Ix, S> {
    pub collection: Collection<In, Ix, S>,
}
