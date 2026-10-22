/// The master-data shape used to select an adapter's expiration policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheKind {
    Individual,
    List,
}
