use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Message {
    pub data: Vec<u8>,
    pub index: u64,
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.index).cmp(&(other.index))
    }
}
