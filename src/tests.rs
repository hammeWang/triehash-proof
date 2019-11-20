use super::*;
use trie_root::*;
extern crate ethereum_types;
extern crate keccak_hasher;
pub use trie_root::{trie_root, shared_prefix_len, hex_prefix_encode};
pub use keccak_hasher::KeccakHasher;
pub use self::ethereum_types::H256;



#[test]
fn test_hex_prefix_encode() {
	let v = vec![0, 0, 1, 2, 3, 4, 5];
	let e = vec![0x10, 0x01, 0x23, 0x45];
	let h = hex_prefix_encode(&v, false).collect::<Vec<_>>();
	assert_eq!(h, e);

	let v = vec![0, 1, 2, 3, 4, 5];
	let e = vec![0x00, 0x01, 0x23, 0x45];
	let h = hex_prefix_encode(&v, false).collect::<Vec<_>>();
	assert_eq!(h, e);

	let v = vec![0, 1, 2, 3, 4, 5];
	let e = vec![0x20, 0x01, 0x23, 0x45];
	let h = hex_prefix_encode(&v, true).collect::<Vec<_>>();
	assert_eq!(h, e);

	let v = vec![1, 2, 3, 4, 5];
	let e = vec![0x31, 0x23, 0x45];
	let h = hex_prefix_encode(&v, true).collect::<Vec<_>>();
	assert_eq!(h, e);

	let v = vec![1, 2, 3, 4];
	let e = vec![0x00, 0x12, 0x34];
	let h = hex_prefix_encode(&v, false).collect::<Vec<_>>();
	assert_eq!(h, e);

	let v = vec![4, 1];
	let e = vec![0x20, 0x41];
	let h = hex_prefix_encode(&v, true).collect::<Vec<_>>();
	assert_eq!(h, e);
}

#[test]
fn simple_test() {
	assert_eq!(trie_root::<KeccakHasher, _, _, _>(vec![
		(b"A", b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" as &[u8])
	]), H256::from(hex!("d23786fb4a010da3ce639d66d5e904a11dbc02746d1ce25029e53290cabf28ab")).as_ref());
}

#[test]
fn test_triehash_out_of_order() {
	assert!(trie_root::<KeccakHasher, _, _, _>(vec![
		(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
		(vec![0x81u8, 0x23], vec![0x81u8, 0x23]),
		(vec![0xf1u8, 0x23], vec![0xf1u8, 0x23]),
	]) ==
		trie_root::<KeccakHasher, _, _, _>(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0xf1u8, 0x23], vec![0xf1u8, 0x23]), // last two tuples are swapped
			(vec![0x81u8, 0x23], vec![0x81u8, 0x23]),
		]));
}

#[test]
fn test_shared_prefix() {
	let a = vec![1, 2, 3, 4, 5, 6];
	let b = vec![4, 2, 3, 4, 5, 6];
	assert_eq!(shared_prefix_len(&a, &b), 0);
}

#[test]
fn test_shared_prefix2() {
	let a = vec![1, 2, 3, 3, 5];
	let b = vec![1, 2, 3];
	assert_eq!(shared_prefix_len(&a, &b), 3);
}

#[test]
fn test_shared_prefix3() {
	let a = vec![1, 2, 3, 4, 5, 6];
	let b = vec![1, 2, 3, 4, 5, 6];
	assert_eq!(shared_prefix_len(&a, &b), 6);
}

#[cfg(test)]
pub mod extra_tests {
	use alloc::vec::Vec;
	#[test]
	/// check nibbles in trie_root
	fn check_rlp_index() {
		// index = 1
		let ri = rlp::encode(&155_u32);
		assert_eq!(ri, vec![0x81, 0x9b]);
		let mut v: Vec<u8> = vec![];
		for b in ri {
			v.push(b >> 4);
			v. push(b & 0x0F);
		}
		assert_eq!(v, vec![8, 1, 9, 0xb]);
	}
}
