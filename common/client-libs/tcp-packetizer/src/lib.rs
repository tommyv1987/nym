#[derive(Clone, Debug)]
pub struct Fragment {
    data: Vec<u8>,
    index: usize,
}

pub struct Packetizer {
    fragment_max_size: usize,
    next_index: usize,
}

impl Packetizer {
    pub fn new(fragment_max_size: usize) -> Packetizer {
        Packetizer {
            fragment_max_size,
            next_index: 0,
        }
    }

    pub fn packetize(&mut self, input: Vec<u8>) -> Vec<Fragment> {
        input
            .chunks(self.fragment_max_size)
            .map(|frag| {
                let f = Fragment {
                    data: frag.to_vec(),
                    index: self.next_index,
                };
                self.next_index = self.next_index + 1;
                f
            })
            .collect()
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

    #[cfg(test)]
    mod sequence_numbers {
        use super::*;

        #[test]
        fn increase_when_packetizing() {
            let mut packetizer = Packetizer::new(4);
            let first_bytes = vec![1, 2, 3, 4];
            let second_bytes = vec![5, 6, 7, 8];

            let first_frags = packetizer.packetize(first_bytes);
            assert_eq!(1, first_frags.len());
            let first_indexes: Vec<usize> = first_frags.iter().map(|frag| frag.index).collect();
            assert_eq!(first_indexes, vec![0]);

            let second_frags = packetizer.packetize(second_bytes);
            assert_eq!(1, second_frags.len());
            let second_indexes: Vec<usize> = second_frags.iter().map(|frag| frag.index).collect();
            assert_eq!(second_indexes, vec![1]);
        }
    }

    #[cfg(test)]
    mod packet_chunking {
        use super::*;

        #[cfg(test)]
        mod when_max_fragment_size_equals_bytes_supplied {
            use super::*;

            #[test]
            fn produces_a_vec_with_a_single_fragment() {
                let mut packetizer = Packetizer::new(4);
                let bytes: Vec<u8> = vec![1, 2, 3, 4];
                let output = packetizer.packetize(bytes);
                assert_eq!(1, output.len());
                assert_eq!(0, output.first().unwrap().index);
            }
        }

        #[cfg(test)]
        mod when_max_size_is_greater_than_bytes_supplied {
            use super::*;

            #[test]
            fn produces_a_vec_with_a_single_fragment() {
                let mut packetizer = Packetizer::new(5);
                let bytes: Vec<u8> = vec![1, 2, 3, 4];
                let output = packetizer.packetize(bytes);
                assert_eq!(1, output.len());
                assert_eq!(0, output.first().unwrap().index);
            }
        }

        #[cfg(test)]
        mod when_max_size_is_less_than_bytes_supplied {
            use super::*;

            #[test]
            fn produces_a_vec_with_modulo_fragments() {
                let mut packetizer = Packetizer::new(3);
                let bytes: Vec<u8> = vec![1, 2, 3, 4];
                let output = packetizer.packetize(bytes);
                assert_eq!(2, output.len());
                // check that indexes are correct
                assert_eq!(0, output[0].index);
                assert_eq!(1, output[1].index);
            }
        }
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
