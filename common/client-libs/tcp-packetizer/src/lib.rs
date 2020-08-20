#[derive(Clone)]
pub struct Fragment {
    data: Vec<u8>,
    index: usize,
}

pub struct Packetizer {
    fragment_size: usize,
    next_index: usize,
}

impl Packetizer {
    pub fn new(fragment_size: usize) -> Packetizer {
        Packetizer {
            fragment_size,
            next_index: 0,
        }
    }

    pub fn packetize(&mut self, input: Vec<u8>) -> Vec<Fragment> {
        let data = input;
        let sequence_number = self.next_index;
        let frag = vec![Fragment {
            data,
            index: sequence_number,
        }];
        self.next_index = self.next_index + 1;
        frag
    }
}

/// The `MessageBuffer` stores messages and emits them in order
pub struct OrderedMessageBuffer {
    fragments: Vec<Fragment>,
}

impl OrderedMessageBuffer {
    pub fn new() -> OrderedMessageBuffer {
        OrderedMessageBuffer {
            fragments: Vec::new(),
        }
    }

    pub fn add(&mut self, fragment: Fragment) -> Option<Vec<u8>> {
        self.fragments.push(fragment.clone());
        Some(fragment.data.clone())
    }
}

#[cfg(test)]
mod test_chunking_and_reassembling {
    use super::*;

    mod when_input_bytes_are_empty {}

    #[test]
    fn test_chunk_bytes_with_fragment_size_at_byte_length_produces_a_fragment_vec_with_a_sequence_number(
    ) {
        let mut packetizer = Packetizer::new(4);
        let first_bytes: Vec<u8> = vec![1, 2, 3, 4];
        let second_bytes: Vec<u8> = vec![5, 6, 7, 8];
        let third_bytes: Vec<u8> = vec![9, 10, 11, 12];

        let first_frags = packetizer.packetize(first_bytes);
        assert_eq!(1, first_frags.len());
        let first_indexes: Vec<usize> = first_frags.iter().map(|frag| frag.index).collect();
        assert_eq!(first_indexes, vec![0]);

        let second_frags = packetizer.packetize(second_bytes);
        assert_eq!(1, second_frags.len());
        let second_indexes: Vec<usize> = second_frags.iter().map(|frag| frag.index).collect();
        assert_eq!(second_indexes, vec![1]);

        let third_frags = packetizer.packetize(third_bytes);
        assert_eq!(1, third_frags.len());
        let third_indexes: Vec<usize> = third_frags.iter().map(|frag| frag.index).collect();
        assert_eq!(third_indexes, vec![2]);
    }

    #[test]
    fn test_reassembling_fragments_produces_original_bytes() {
        let mut buffer = OrderedMessageBuffer::new();

        let first_frag = Fragment {
            data: vec![1, 2, 3, 4],
            index: 0,
        };
        let second_frag = Fragment {
            data: vec![5, 6, 7, 8],
            index: 1,
        };

        let first_add = buffer.add(first_frag);
        assert_eq!(vec![1, 2, 3, 4], first_add.unwrap());

        let second_add = buffer.add(second_frag);
        assert_eq!(vec![5, 6, 7, 8], second_add.unwrap());
    }
}
