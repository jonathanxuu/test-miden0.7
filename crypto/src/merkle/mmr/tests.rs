use super::{
    super::{InnerNodeInfo, Vec},
    bit::TrueBitPositionIterator,
    full::{high_bitmask, leaf_to_corresponding_tree, nodes_in_forest},
    Mmr, MmrPeaks, Rpo256,
};
use crate::{
    hash::rpo::RpoDigest,
    merkle::{int_to_node, MerklePath},
    Felt, Word,
};

#[test]
fn test_position_equal_or_higher_than_leafs_is_never_contained() {
    let empty_forest = 0;
    for pos in 1..1024 {
        // pos is index, 0 based
        // tree is a length counter, 1 based
        // so a valid pos is always smaller, not equal, to tree
        assert_eq!(leaf_to_corresponding_tree(pos, pos), None);
        assert_eq!(leaf_to_corresponding_tree(pos, pos - 1), None);
        // and empty forest has no trees, so no position is valid
        assert_eq!(leaf_to_corresponding_tree(pos, empty_forest), None);
    }
}

#[test]
fn test_position_zero_is_always_contained_by_the_highest_tree() {
    for leaves in 1..1024usize {
        let tree = leaves.ilog2();
        assert_eq!(leaf_to_corresponding_tree(0, leaves), Some(tree));
    }
}

#[test]
fn test_leaf_to_corresponding_tree() {
    assert_eq!(leaf_to_corresponding_tree(0, 0b0001), Some(0));
    assert_eq!(leaf_to_corresponding_tree(0, 0b0010), Some(1));
    assert_eq!(leaf_to_corresponding_tree(0, 0b0011), Some(1));
    assert_eq!(leaf_to_corresponding_tree(0, 0b1011), Some(3));

    // position one is always owned by the left-most tree
    assert_eq!(leaf_to_corresponding_tree(1, 0b0010), Some(1));
    assert_eq!(leaf_to_corresponding_tree(1, 0b0011), Some(1));
    assert_eq!(leaf_to_corresponding_tree(1, 0b1011), Some(3));

    // position two starts as its own root, and then it is merged with the left-most tree
    assert_eq!(leaf_to_corresponding_tree(2, 0b0011), Some(0));
    assert_eq!(leaf_to_corresponding_tree(2, 0b0100), Some(2));
    assert_eq!(leaf_to_corresponding_tree(2, 0b1011), Some(3));

    // position tree is merged on the left-most tree
    assert_eq!(leaf_to_corresponding_tree(3, 0b0011), None);
    assert_eq!(leaf_to_corresponding_tree(3, 0b0100), Some(2));
    assert_eq!(leaf_to_corresponding_tree(3, 0b1011), Some(3));

    assert_eq!(leaf_to_corresponding_tree(4, 0b0101), Some(0));
    assert_eq!(leaf_to_corresponding_tree(4, 0b0110), Some(1));
    assert_eq!(leaf_to_corresponding_tree(4, 0b0111), Some(1));
    assert_eq!(leaf_to_corresponding_tree(4, 0b1000), Some(3));

    assert_eq!(leaf_to_corresponding_tree(12, 0b01101), Some(0));
    assert_eq!(leaf_to_corresponding_tree(12, 0b01110), Some(1));
    assert_eq!(leaf_to_corresponding_tree(12, 0b01111), Some(1));
    assert_eq!(leaf_to_corresponding_tree(12, 0b10000), Some(4));
}

#[test]
fn test_high_bitmask() {
    assert_eq!(high_bitmask(0), usize::MAX);
    assert_eq!(high_bitmask(1), usize::MAX << 1);
    assert_eq!(high_bitmask(usize::BITS - 2), 0b11usize.rotate_right(2));
    assert_eq!(high_bitmask(usize::BITS - 1), 0b1usize.rotate_right(1));
    assert_eq!(high_bitmask(usize::BITS), 0, "overflow should be handled");
}

#[test]
fn test_nodes_in_forest() {
    assert_eq!(nodes_in_forest(0b0000), 0);
    assert_eq!(nodes_in_forest(0b0001), 1);
    assert_eq!(nodes_in_forest(0b0010), 3);
    assert_eq!(nodes_in_forest(0b0011), 4);
    assert_eq!(nodes_in_forest(0b0100), 7);
    assert_eq!(nodes_in_forest(0b0101), 8);
    assert_eq!(nodes_in_forest(0b0110), 10);
    assert_eq!(nodes_in_forest(0b0111), 11);
    assert_eq!(nodes_in_forest(0b1000), 15);
    assert_eq!(nodes_in_forest(0b1001), 16);
    assert_eq!(nodes_in_forest(0b1010), 18);
    assert_eq!(nodes_in_forest(0b1011), 19);
}

#[test]
fn test_nodes_in_forest_single_bit() {
    assert_eq!(nodes_in_forest(2usize.pow(0)), 2usize.pow(1) - 1);
    assert_eq!(nodes_in_forest(2usize.pow(1)), 2usize.pow(2) - 1);
    assert_eq!(nodes_in_forest(2usize.pow(2)), 2usize.pow(3) - 1);
    assert_eq!(nodes_in_forest(2usize.pow(3)), 2usize.pow(4) - 1);

    for bit in 0..(usize::BITS - 1) {
        let size = 2usize.pow(bit + 1) - 1;
        assert_eq!(nodes_in_forest(1usize << bit), size);
    }
}

const LEAVES: [RpoDigest; 7] = [
    int_to_node(0),
    int_to_node(1),
    int_to_node(2),
    int_to_node(3),
    int_to_node(4),
    int_to_node(5),
    int_to_node(6),
];

#[test]
fn test_mmr_simple() {
    let mut postorder = Vec::new();
    postorder.push(LEAVES[0]);
    postorder.push(LEAVES[1]);
    postorder.push(Rpo256::merge(&[LEAVES[0], LEAVES[1]]));
    postorder.push(LEAVES[2]);
    postorder.push(LEAVES[3]);
    postorder.push(Rpo256::merge(&[LEAVES[2], LEAVES[3]]));
    postorder.push(Rpo256::merge(&[postorder[2], postorder[5]]));
    postorder.push(LEAVES[4]);
    postorder.push(LEAVES[5]);
    postorder.push(Rpo256::merge(&[LEAVES[4], LEAVES[5]]));
    postorder.push(LEAVES[6]);

    let mut mmr = Mmr::new();
    assert_eq!(mmr.forest(), 0);
    assert_eq!(mmr.nodes.len(), 0);

    mmr.add(LEAVES[0]);
    assert_eq!(mmr.forest(), 1);
    assert_eq!(mmr.nodes.len(), 1);
    assert_eq!(mmr.nodes.as_slice(), &postorder[0..mmr.nodes.len()]);

    let acc = mmr.accumulator();
    assert_eq!(acc.num_leaves, 1);
    assert_eq!(acc.peaks, &[postorder[0]]);

    mmr.add(LEAVES[1]);
    assert_eq!(mmr.forest(), 2);
    assert_eq!(mmr.nodes.len(), 3);
    assert_eq!(mmr.nodes.as_slice(), &postorder[0..mmr.nodes.len()]);

    let acc = mmr.accumulator();
    assert_eq!(acc.num_leaves, 2);
    assert_eq!(acc.peaks, &[postorder[2]]);

    mmr.add(LEAVES[2]);
    assert_eq!(mmr.forest(), 3);
    assert_eq!(mmr.nodes.len(), 4);
    assert_eq!(mmr.nodes.as_slice(), &postorder[0..mmr.nodes.len()]);

    let acc = mmr.accumulator();
    assert_eq!(acc.num_leaves, 3);
    assert_eq!(acc.peaks, &[postorder[2], postorder[3]]);

    mmr.add(LEAVES[3]);
    assert_eq!(mmr.forest(), 4);
    assert_eq!(mmr.nodes.len(), 7);
    assert_eq!(mmr.nodes.as_slice(), &postorder[0..mmr.nodes.len()]);

    let acc = mmr.accumulator();
    assert_eq!(acc.num_leaves, 4);
    assert_eq!(acc.peaks, &[postorder[6]]);

    mmr.add(LEAVES[4]);
    assert_eq!(mmr.forest(), 5);
    assert_eq!(mmr.nodes.len(), 8);
    assert_eq!(mmr.nodes.as_slice(), &postorder[0..mmr.nodes.len()]);

    let acc = mmr.accumulator();
    assert_eq!(acc.num_leaves, 5);
    assert_eq!(acc.peaks, &[postorder[6], postorder[7]]);

    mmr.add(LEAVES[5]);
    assert_eq!(mmr.forest(), 6);
    assert_eq!(mmr.nodes.len(), 10);
    assert_eq!(mmr.nodes.as_slice(), &postorder[0..mmr.nodes.len()]);

    let acc = mmr.accumulator();
    assert_eq!(acc.num_leaves, 6);
    assert_eq!(acc.peaks, &[postorder[6], postorder[9]]);

    mmr.add(LEAVES[6]);
    assert_eq!(mmr.forest(), 7);
    assert_eq!(mmr.nodes.len(), 11);
    assert_eq!(mmr.nodes.as_slice(), &postorder[0..mmr.nodes.len()]);

    let acc = mmr.accumulator();
    assert_eq!(acc.num_leaves, 7);
    assert_eq!(acc.peaks, &[postorder[6], postorder[9], postorder[10]]);
}

#[test]
fn test_mmr_open() {
    let mmr: Mmr = LEAVES.into();
    let h01 = Rpo256::merge(&[LEAVES[0], LEAVES[1]]);
    let h23 = Rpo256::merge(&[LEAVES[2], LEAVES[3]]);

    // node at pos 7 is the root
    assert!(mmr.open(7).is_err(), "Element 7 is not in the tree, result should be None");

    // node at pos 6 is the root
    let empty: MerklePath = MerklePath::new(vec![]);
    let opening = mmr
        .open(6)
        .expect("Element 6 is contained in the tree, expected an opening result.");
    assert_eq!(opening.merkle_path, empty);
    assert_eq!(opening.forest, mmr.forest);
    assert_eq!(opening.position, 6);
    assert!(
        mmr.accumulator().verify(LEAVES[6], opening),
        "MmrProof should be valid for the current accumulator."
    );

    // nodes 4,5 are depth 1
    let root_to_path = MerklePath::new(vec![LEAVES[4]]);
    let opening = mmr
        .open(5)
        .expect("Element 5 is contained in the tree, expected an opening result.");
    assert_eq!(opening.merkle_path, root_to_path);
    assert_eq!(opening.forest, mmr.forest);
    assert_eq!(opening.position, 5);
    assert!(
        mmr.accumulator().verify(LEAVES[5], opening),
        "MmrProof should be valid for the current accumulator."
    );

    let root_to_path = MerklePath::new(vec![LEAVES[5]]);
    let opening = mmr
        .open(4)
        .expect("Element 4 is contained in the tree, expected an opening result.");
    assert_eq!(opening.merkle_path, root_to_path);
    assert_eq!(opening.forest, mmr.forest);
    assert_eq!(opening.position, 4);
    assert!(
        mmr.accumulator().verify(LEAVES[4], opening),
        "MmrProof should be valid for the current accumulator."
    );

    // nodes 0,1,2,3 are detph 2
    let root_to_path = MerklePath::new(vec![LEAVES[2], h01]);
    let opening = mmr
        .open(3)
        .expect("Element 3 is contained in the tree, expected an opening result.");
    assert_eq!(opening.merkle_path, root_to_path);
    assert_eq!(opening.forest, mmr.forest);
    assert_eq!(opening.position, 3);
    assert!(
        mmr.accumulator().verify(LEAVES[3], opening),
        "MmrProof should be valid for the current accumulator."
    );

    let root_to_path = MerklePath::new(vec![LEAVES[3], h01]);
    let opening = mmr
        .open(2)
        .expect("Element 2 is contained in the tree, expected an opening result.");
    assert_eq!(opening.merkle_path, root_to_path);
    assert_eq!(opening.forest, mmr.forest);
    assert_eq!(opening.position, 2);
    assert!(
        mmr.accumulator().verify(LEAVES[2], opening),
        "MmrProof should be valid for the current accumulator."
    );

    let root_to_path = MerklePath::new(vec![LEAVES[0], h23]);
    let opening = mmr
        .open(1)
        .expect("Element 1 is contained in the tree, expected an opening result.");
    assert_eq!(opening.merkle_path, root_to_path);
    assert_eq!(opening.forest, mmr.forest);
    assert_eq!(opening.position, 1);
    assert!(
        mmr.accumulator().verify(LEAVES[1], opening),
        "MmrProof should be valid for the current accumulator."
    );

    let root_to_path = MerklePath::new(vec![LEAVES[1], h23]);
    let opening = mmr
        .open(0)
        .expect("Element 0 is contained in the tree, expected an opening result.");
    assert_eq!(opening.merkle_path, root_to_path);
    assert_eq!(opening.forest, mmr.forest);
    assert_eq!(opening.position, 0);
    assert!(
        mmr.accumulator().verify(LEAVES[0], opening),
        "MmrProof should be valid for the current accumulator."
    );
}

#[test]
fn test_mmr_get() {
    let mmr: Mmr = LEAVES.into();
    assert_eq!(mmr.get(0).unwrap(), LEAVES[0], "value at pos 0 must correspond");
    assert_eq!(mmr.get(1).unwrap(), LEAVES[1], "value at pos 1 must correspond");
    assert_eq!(mmr.get(2).unwrap(), LEAVES[2], "value at pos 2 must correspond");
    assert_eq!(mmr.get(3).unwrap(), LEAVES[3], "value at pos 3 must correspond");
    assert_eq!(mmr.get(4).unwrap(), LEAVES[4], "value at pos 4 must correspond");
    assert_eq!(mmr.get(5).unwrap(), LEAVES[5], "value at pos 5 must correspond");
    assert_eq!(mmr.get(6).unwrap(), LEAVES[6], "value at pos 6 must correspond");
    assert!(mmr.get(7).is_err());
}

#[test]
fn test_mmr_invariants() {
    let mut mmr = Mmr::new();
    for v in 1..=1028 {
        mmr.add(int_to_node(v));
        let accumulator = mmr.accumulator();
        assert_eq!(v as usize, mmr.forest(), "MMR leaf count must increase by one on every add");
        assert_eq!(
            v as usize, accumulator.num_leaves,
            "MMR and its accumulator must match leaves count"
        );
        assert_eq!(
            accumulator.num_leaves.count_ones() as usize,
            accumulator.peaks.len(),
            "bits on leaves must match the number of peaks"
        );

        let expected_nodes: usize = TrueBitPositionIterator::new(mmr.forest())
            .map(|bit_pos| nodes_in_forest(1 << bit_pos))
            .sum();

        assert_eq!(
            expected_nodes,
            mmr.nodes.len(),
            "the sum of every tree size must be equal to the number of nodes in the MMR (forest: {:b})",
            mmr.forest(),
        );
    }
}

#[test]
fn test_bit_position_iterator() {
    assert_eq!(TrueBitPositionIterator::new(0).count(), 0);
    assert_eq!(TrueBitPositionIterator::new(0).rev().count(), 0);

    assert_eq!(TrueBitPositionIterator::new(1).collect::<Vec<u32>>(), vec![0]);
    assert_eq!(TrueBitPositionIterator::new(1).rev().collect::<Vec<u32>>(), vec![0],);

    assert_eq!(TrueBitPositionIterator::new(2).collect::<Vec<u32>>(), vec![1]);
    assert_eq!(TrueBitPositionIterator::new(2).rev().collect::<Vec<u32>>(), vec![1],);

    assert_eq!(TrueBitPositionIterator::new(3).collect::<Vec<u32>>(), vec![0, 1],);
    assert_eq!(TrueBitPositionIterator::new(3).rev().collect::<Vec<u32>>(), vec![1, 0],);

    assert_eq!(
        TrueBitPositionIterator::new(0b11010101).collect::<Vec<u32>>(),
        vec![0, 2, 4, 6, 7],
    );
    assert_eq!(
        TrueBitPositionIterator::new(0b11010101).rev().collect::<Vec<u32>>(),
        vec![7, 6, 4, 2, 0],
    );
}

#[test]
fn test_mmr_inner_nodes() {
    let mmr: Mmr = LEAVES.into();
    let nodes: Vec<InnerNodeInfo> = mmr.inner_nodes().collect();

    let h01 = Rpo256::merge(&[LEAVES[0], LEAVES[1]]);
    let h23 = Rpo256::merge(&[LEAVES[2], LEAVES[3]]);
    let h0123 = Rpo256::merge(&[h01, h23]);
    let h45 = Rpo256::merge(&[LEAVES[4], LEAVES[5]]);
    let postorder = vec![
        InnerNodeInfo {
            value: h01,
            left: LEAVES[0],
            right: LEAVES[1],
        },
        InnerNodeInfo {
            value: h23,
            left: LEAVES[2],
            right: LEAVES[3],
        },
        InnerNodeInfo { value: h0123, left: h01, right: h23 },
        InnerNodeInfo {
            value: h45,
            left: LEAVES[4],
            right: LEAVES[5],
        },
    ];

    assert_eq!(postorder, nodes);
}

#[test]
fn test_mmr_hash_peaks() {
    let mmr: Mmr = LEAVES.into();
    let peaks = mmr.accumulator();

    let first_peak = Rpo256::merge(&[
        Rpo256::merge(&[LEAVES[0], LEAVES[1]]),
        Rpo256::merge(&[LEAVES[2], LEAVES[3]]),
    ]);
    let second_peak = Rpo256::merge(&[LEAVES[4], LEAVES[5]]);
    let third_peak = LEAVES[6];

    // minimum length is 16
    let mut expected_peaks = [first_peak, second_peak, third_peak].to_vec();
    expected_peaks.resize(16, RpoDigest::default());
    assert_eq!(
        peaks.hash_peaks(),
        *Rpo256::hash_elements(&digests_to_elements(&expected_peaks))
    );
}

#[test]
fn test_mmr_peaks_hash_less_than_16() {
    let mut peaks = Vec::new();

    for i in 0..16 {
        peaks.push(int_to_node(i));
        let accumulator = MmrPeaks {
            num_leaves: (1 << peaks.len()) - 1,
            peaks: peaks.clone(),
        };

        // minimum length is 16
        let mut expected_peaks = peaks.clone();
        expected_peaks.resize(16, RpoDigest::default());
        assert_eq!(
            accumulator.hash_peaks(),
            *Rpo256::hash_elements(&digests_to_elements(&expected_peaks))
        );
    }
}

#[test]
fn test_mmr_peaks_hash_odd() {
    let peaks: Vec<_> = (0..=17).map(int_to_node).collect();

    let accumulator = MmrPeaks {
        num_leaves: (1 << peaks.len()) - 1,
        peaks: peaks.clone(),
    };

    // odd length bigger than 16 is padded to the next even number
    let mut expected_peaks = peaks;
    expected_peaks.resize(18, RpoDigest::default());
    assert_eq!(
        accumulator.hash_peaks(),
        *Rpo256::hash_elements(&digests_to_elements(&expected_peaks))
    );
}

mod property_tests {
    use super::leaf_to_corresponding_tree;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_last_position_is_always_contained_in_the_last_tree(leaves in any::<usize>().prop_filter("cant have an empty tree", |v| *v != 0)) {
            let last_pos = leaves - 1;
            let lowest_bit = leaves.trailing_zeros();

            assert_eq!(
                leaf_to_corresponding_tree(last_pos, leaves),
                Some(lowest_bit),
            );
        }
    }

    proptest! {
        #[test]
        fn test_contained_tree_is_always_power_of_two((leaves, pos) in any::<usize>().prop_flat_map(|v| (Just(v), 0..v))) {
            let tree = leaf_to_corresponding_tree(pos, leaves).expect("pos is smaller than leaves, there should always be a corresponding tree");
            let mask = 1usize << tree;

            assert!(tree < usize::BITS, "the result must be a bit in usize");
            assert!(mask & leaves != 0, "the result should be a tree in leaves");
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================

fn digests_to_elements(digests: &[RpoDigest]) -> Vec<Felt> {
    digests.iter().flat_map(Word::from).collect()
}
