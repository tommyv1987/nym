use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct OrderedMessage {
    pub data: Vec<u8>,
    pub index: u64,
}

impl OrderedMessage {
    pub fn into_bytes(self) -> Vec<u8> {
        self.index
            .to_be_bytes()
            .iter()
            .cloned()
            .chain(self.data.into_iter())
            .collect()
    }

    pub fn from_be_bytes(data: Vec<u8>) -> OrderedMessage {
        let index = u64::from_be_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        OrderedMessage {
            data: data.to_vec(),
            index,
        }
    }
}

impl Ord for OrderedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.index).cmp(&(other.index))
    }
}

#[cfg(test)]
mod serializing_to_bytes {
    use super::*;

    #[test]
    fn works() {
        let message = OrderedMessage {
            data: vec![123],
            index: 1,
        };
        let bytes = message.into_bytes();

        let expected = vec![0, 0, 0, 0, 0, 0, 0, 1, 123];
        assert_eq!(expected, bytes);
    }
}
