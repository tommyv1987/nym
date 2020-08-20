use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct Fragment {
    data: Vec<u8>,
    index: usize,
}

impl Ord for Fragment {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.index).cmp(&(other.index))
    }
}

impl PartialOrd for Fragment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Fragment {
    fn eq(&self, other: &Self) -> bool {
        (self.index, &self.data) == (other.index, &other.data)
    }
}

impl Eq for Fragment {}

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

/// The `OrderedMessageBuffer` stores messages and emits them in order
pub struct OrderedMessageBuffer {
    fragments: Vec<Fragment>,
}

impl OrderedMessageBuffer {
    pub fn new() -> OrderedMessageBuffer {
        OrderedMessageBuffer {
            fragments: Vec::new(),
        }
    }

    /// Writes a fragment to the buffer. Fragments are sort on insertion, so
    /// that later on multiple reads for incomplete sequences don't result in
    /// useless sort work.
    pub fn write(&mut self, fragment: Fragment) {
        self.fragments.push(fragment.clone());
        OrderedMessageBuffer::insertion_sort(&mut self.fragments);
    }

    /// Reads an ordered sequence of bytes out of the buffer.
    pub fn read(&mut self) -> Option<Vec<u8>> {
        let data = self
            .fragments
            .iter()
            .flat_map(|frag| frag.data.clone())
            .collect();
        self.fragments = Vec::new();
        Some(data)
    }

    pub fn insertion_sort<T>(values: &mut [T])
    where
        T: Ord,
    {
        for i in 0..values.len() {
            for j in (0..i).rev() {
                if values[j] >= values[j + 1] {
                    values.swap(j, j + 1);
                } else {
                    break;
                }
            }
        }
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
                // check that indexes increase properly
                assert_eq!(0, output[0].index);
                assert_eq!(1, output[1].index);
            }
        }
    }

    #[cfg(test)]
    mod reading_from_the_buffer {
        use super::*;

        #[test]
        fn test_reads_returns_original_bytes_and_resets_buffer() {
            let mut buffer = OrderedMessageBuffer::new();

            let first_frag = Fragment {
                data: vec![1, 2, 3, 4],
                index: 0,
            };
            let second_frag = Fragment {
                data: vec![5, 6, 7, 8],
                index: 1,
            };

            buffer.write(first_frag);
            let first_read = buffer.read();
            assert_eq!(vec![1, 2, 3, 4], first_read.unwrap());

            buffer.write(second_frag);
            let second_read = buffer.read();
            assert_eq!(vec![5, 6, 7, 8], second_read.unwrap());
        }

        #[test]
        fn test_multiple_adds_stacks_up_bytes_in_the_buffer() {
            let mut buffer = OrderedMessageBuffer::new();

            let first_frag = Fragment {
                data: vec![1, 2, 3, 4],
                index: 0,
            };
            let second_frag = Fragment {
                data: vec![5, 6, 7, 8],
                index: 1,
            };

            buffer.write(first_frag);
            buffer.write(second_frag);
            let second_read = buffer.read();
            assert_eq!(vec![1, 2, 3, 4, 5, 6, 7, 8], second_read.unwrap());
        }

        #[test]
        fn test_out_of_order_adds_results_in_ordered_byte_vector() {
            let mut buffer = OrderedMessageBuffer::new();

            let first_frag = Fragment {
                data: vec![1, 2, 3, 4],
                index: 0,
            };
            let second_frag = Fragment {
                data: vec![5, 6, 7, 8],
                index: 1,
            };

            buffer.write(second_frag);
            buffer.write(first_frag);
            let read = buffer.read();
            assert_eq!(vec![1, 2, 3, 4, 5, 6, 7, 8], read.unwrap());
        }
    }
}
