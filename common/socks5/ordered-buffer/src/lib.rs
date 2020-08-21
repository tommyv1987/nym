use std::cmp::Ordering;

pub struct Fragment {
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
    data: Vec<u8>,
    index: usize,
}

impl Ord for Fragment {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.index).cmp(&(other.index))
    }
}


pub struct OrderedFragmentSender {
    fragment_max_size: usize,
    next_index: usize,
}

impl OrderedFragmentSender {
    pub fn new(fragment_max_size: usize) -> OrderedFragmentSender {
        OrderedFragmentSender {
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

#[derive(Debug)]
/// The `OrderedFragmentBuffer` stores messages and emits them in order
pub struct OrderedFragmentBuffer {
    next_index: usize,
    fragments: Vec<Fragment>,
}

impl OrderedFragmentBuffer {
    pub fn new() -> OrderedFragmentBuffer {
        OrderedFragmentBuffer {
            next_index: 0,
            fragments: Vec::new(),
        }
    }

    /// Writes a fragment to the buffer. Fragments are sort on insertion, so
    /// that later on multiple reads for incomplete sequences don't result in
    /// useless sort work.
    pub fn write(&mut self, fragment: Fragment) {
        self.fragments.push(fragment.clone());
        OrderedFragmentBuffer::insertion_sort(&mut self.fragments);
    }

    /// Reads an ordered sequence of bytes out of the buffer. Only contiguous
    /// fragments with an index less than or equal to `next_index` will be
    /// returned - this avoids returning gaps while we wait for the buffer to fill
    /// up with the full sequence.
    pub fn read(&mut self) -> Option<Vec<u8>> {
        if self.fragments.is_empty() || self.fragments.first().unwrap().index > self.next_index {
            return None;
        } else {
            println!("initial self: {:?}", self);
            let index = self.next_index.clone() + 1;
            let contiguous_fragments: Vec<Fragment> = self
                .fragments
                .iter()
                .filter(|frag| frag.index <= index)
                .cloned()
                .collect();

            println!("contiguous_fragments is {:?}", contiguous_fragments);
            // get rid of all fragments we're about to send out of the buffer
            self.fragments.retain(|frag| frag.index > index);
            println!("retained: {:?}", self.fragments);

            // advance the index because we've read stuff up to a new high water mark
            let high_water = index + contiguous_fragments.len() - 1;
            self.next_index = high_water;
            println!("setting next_index to: {}", high_water);

            // dig out the bytes from inside the struct
            let data = contiguous_fragments
                .iter()
                .flat_map(|frag| frag.data.clone())
                .collect();

            Some(data)
        }
    }

    fn insertion_sort<T>(values: &mut [T])
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
    mod sequence_index_numbers {
        use super::*;

        #[test]
        fn increase_as_fragments_are_sent() {
            let mut packetizer = OrderedFragmentSender::new(4);
            let first_bytes = vec![1, 2, 3, 4];
            let second_bytes = vec![5, 6, 7, 8];

            let first_fragments = packetizer.packetize(first_bytes);
            assert_eq!(1, first_fragments.len());
            let first_indexes: Vec<usize> = first_fragments.iter().map(|frag| frag.index).collect();
            assert_eq!(first_indexes, vec![0]);

            let second_fragments = packetizer.packetize(second_bytes);
            assert_eq!(1, second_fragments.len());
            let second_indexes: Vec<usize> =
                second_fragments.iter().map(|frag| frag.index).collect();
            assert_eq!(second_indexes, vec![1]);
        }
    }

    #[cfg(test)]
    mod sending_ordered_fragments_for_received_bytes {
        use super::*;

        #[cfg(test)]
        mod when_max_fragment_size_equals_bytes_supplied {
            use super::*;

            #[test]
            fn produces_a_vec_with_a_single_fragment() {
                let mut packetizer = OrderedFragmentSender::new(4);
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
                let mut packetizer = OrderedFragmentSender::new(5);
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
                let mut packetizer = OrderedFragmentSender::new(3);
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
    mod reading_from_and_writing_to_the_buffer {
        use super::*;

        #[cfg(test)]
        mod when_full_ordered_sequence_exists {
            use super::*;
            #[test]
            fn read_returns_ordered_bytes_and_resets_buffer() {
                let mut buffer = OrderedFragmentBuffer::new();

                let first_frag = Fragment {
                    data: vec![1, 2, 3, 4],
                    index: 0,
                };
                let second_frag = Fragment {
                    data: vec![5, 6, 7, 8],
                    index: 1,
                };

                buffer.write(first_frag);
                let first_read = buffer.read().unwrap();
                assert_eq!(vec![1, 2, 3, 4], first_read);

                buffer.write(second_frag);
                let second_read = buffer.read().unwrap();
                assert_eq!(vec![5, 6, 7, 8], second_read);

                assert_eq!(None, buffer.read()); // second read on fully ordered result set is empty
            }

            #[test]
            fn test_multiple_adds_stacks_up_bytes_in_the_buffer() {
                let mut buffer = OrderedFragmentBuffer::new();

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
                assert_eq!(None, buffer.read()); // second read on fully ordered result set is empty
            }

            #[test]
            fn out_of_order_adds_results_in_ordered_byte_vector() {
                let mut buffer = OrderedFragmentBuffer::new();

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
                assert_eq!(None, buffer.read()); // second read on fully ordered result set is empty
            }
        }

        mod when_there_are_gaps_in_the_sequence {

            #[cfg(test)]
            use super::*;
            fn setup() -> OrderedFragmentBuffer {
                let mut buffer = OrderedFragmentBuffer::new();

                let zero_frag = Fragment {
                    data: vec![0, 0, 0, 0],
                    index: 0,
                };
                let one_frag = Fragment {
                    data: vec![1, 1, 1, 1],
                    index: 1,
                };

                let three_frag = Fragment {
                    data: vec![3, 3, 3, 3],
                    index: 3,
                };

                buffer.write(zero_frag);
                buffer.write(one_frag);
                buffer.write(three_frag);
                buffer
            }
            #[test]
            fn everything_up_to_the_indexing_gap_is_returned_xxx() {
                let mut buffer = setup();
                let ordered_bytes = buffer.read().unwrap();
                assert_eq!([0, 0, 0, 0, 1, 1, 1, 1].to_vec(), ordered_bytes);

                // we shouldn't get any more from a second attempt if nothing is added
                assert_eq!(None, buffer.read());

                // let's add another fragment, leaving a gap in place at index 2
                let five_frag = Fragment {
                    data: vec![5, 5, 5, 5],
                    index: 5,
                };
                buffer.write(five_frag);
                assert_eq!(None, buffer.read());
            }

            #[test]
            fn filling_the_gap_allows_us_to_get_everything() {
                let mut buffer = setup();
                buffer.read(); // that burns the first two. We still have a gap before the 3s.

                let two_frag = Fragment {
                    data: vec![2, 2, 2, 2],
                    index: 2,
                };
                buffer.write(two_frag);

                let more_ordered_bytes = buffer.read().unwrap();
                assert_eq!([2, 2, 2, 2, 3, 3, 3, 3].to_vec(), more_ordered_bytes);

                // let's add another fragment
                let five_frag = Fragment {
                    data: vec![5, 5, 5, 5],
                    index: 5,
                };
                buffer.write(five_frag);

                assert_eq!(None, buffer.read());

                // let's fill in the gap of 4s now and read again
                let four_frag = Fragment {
                    data: vec![4, 4, 4, 4],
                    index: 4,
                };
                buffer.write(four_frag);

                assert_eq!([4, 4, 4, 4, 5, 5, 5, 5].to_vec(), buffer.read().unwrap());

                // at this point we should again get back nothing if we try a read
                assert_eq!(None, buffer.read());
            }
        }
    }
}
