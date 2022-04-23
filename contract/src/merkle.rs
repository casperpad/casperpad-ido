use merkle_tree::CBMT;

struct MergeI32 {}

impl Merge for MergeI32 {
    type Item = i32;
    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        right.wrapping_sub(*left)
    }
}

type CBMTI32 = CBMT<i32, MergeI32>;

let leaves = vec![2i32, 3, 5, 7, 11];
let indices = vec![0, 4];
let proof_leaves = vec![2i32, 11];
let root = CBMTI32::build_merkle_root(&leaves);
let proof = CBMTI32::build_merkle_proof(&leaves, &indices).unwrap();
let tree = CBMTI32::build_merkle_tree(leaves);
proof.verify(&proof_leaves, &root);