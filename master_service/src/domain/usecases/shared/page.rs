/// Offset pagination requested by a caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Page {
    pub limit: u32,
    pub offset: u32,
}
