use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct OrderedMessage {
    pub data: Vec<u8>,
    pub index: u64,
}

impl Ord for OrderedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.index).cmp(&(other.index))
    }
}
