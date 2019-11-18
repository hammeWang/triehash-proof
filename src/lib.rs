// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Generetes trie root.
//!
//! This module should be used to generate trie root hash.

extern crate hash_db;
extern crate rlp;
#[cfg(test)]
extern crate keccak_hasher;
#[cfg(test)]
#[macro_use]
extern crate hex_literal;


use rstd::collections::btree_map::BTreeMap;
use rstd::cmp;
#[cfg(not(feature = "std"))]
use core::iter::once;
use hash_db::Hasher;
use rlp::RlpStream;

fn shared_prefix_len<T: Eq>(first: &[T], second: &[T]) -> usize {
    first.iter()
        .zip(second.iter())
        .position(|(f, s)| f != s)
        .unwrap_or_else(|| cmp::min(first.len(), second.len()))
}

/// Generates a trie root hash for a vector of values
pub fn ordered_trie_root<H, I>(input: I) -> H::Out
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
        H: Hasher,
        <H as hash_db::Hasher>::Out: cmp::Ord,
{
    trie_root::<H, _, _, _>(input.into_iter().enumerate().map(|(i, v)| (rlp::encode(&i), v)))
}


pub fn trie_root<H, I, A, B>(input: I) -> H::Out
    where
        I: IntoIterator<Item = (A, B)>,
        A: AsRef<[u8]> + Ord,
        B: AsRef<[u8]>,
        H: Hasher,
        <H as hash_db::Hasher>::Out: cmp::Ord,
{

    // first put elements into btree to sort them and to remove duplicates
    let input = input
        .into_iter()
        .collect::<BTreeMap<_, _>>();

    let mut nibbles = Vec::with_capacity(input.keys().map(|k| k.as_ref().len()).sum::<usize>() * 2);
    let mut lens = Vec::with_capacity(input.len() + 1);
    lens.push(0);
    for k in input.keys() {
        for &b in k.as_ref() {
            nibbles.push(b >> 4);
            nibbles.push(b & 0x0F);
        }
        lens.push(nibbles.len());
    }

    // then move them to a vector
    let input = input.into_iter().zip(lens.windows(2))
        .map(|((_, v), w)| (&nibbles[w[0]..w[1]], v))
        .collect::<Vec<_>>();

    let mut stream = RlpStream::new();
    hash256rlp::<H, _, _>(&input, 0, &mut stream);
    H::hash(&stream.out())
}

/// Generates a key-hashed (secure) trie root hash for a vector of key-value tuples.
pub fn sec_trie_root<H, I, A, B>(input: I) -> H::Out
    where
        I: IntoIterator<Item = (A, B)>,
        A: AsRef<[u8]>,
        B: AsRef<[u8]>,
        H: Hasher,
        <H as hash_db::Hasher>::Out: cmp::Ord,
{
    trie_root::<H, _, _, _>(input.into_iter().map(|(k, v)| (H::hash(k.as_ref()), v)))
}

/// Hex-prefix Notation. First nibble has flags: oddness = 2^0 & termination = 2^1.
///
/// The "termination marker" and "leaf-node" specifier are completely equivalent.
///
/// Input values are in range `[0, 0xf]`.
///
/// ```markdown
///  [0,0,1,2,3,4,5]   0x10012345 // 7 > 4
///  [0,1,2,3,4,5]     0x00012345 // 6 > 4
///  [1,2,3,4,5]       0x112345   // 5 > 3
///  [0,0,1,2,3,4]     0x00001234 // 6 > 3
///  [0,1,2,3,4]       0x101234   // 5 > 3
///  [1,2,3,4]         0x001234   // 4 > 3
///  [0,0,1,2,3,4,5,T] 0x30012345 // 7 > 4
///  [0,0,1,2,3,4,T]   0x20001234 // 6 > 4
///  [0,1,2,3,4,5,T]   0x20012345 // 6 > 4
///  [1,2,3,4,5,T]     0x312345   // 5 > 3
///  [1,2,3,4,T]       0x201234   // 4 > 3
/// ```
fn hex_prefix_encode<'a>(nibbles: &'a [u8], leaf: bool) -> impl Iterator<Item = u8> + 'a {
    let inlen = nibbles.len();
    let oddness_factor = inlen % 2;

    let first_byte = {
        let mut bits = ((inlen as u8 & 1) + (2 * leaf as u8)) << 4;
        if oddness_factor == 1 {
            bits += nibbles[0];
        }
        bits
    };
    once(first_byte).chain(nibbles[oddness_factor..].chunks(2).map(|ch| ch[0] << 4 | ch[1]))
}

fn hash256rlp<H, A, B>(input: &[(A, B)], pre_len: usize, stream: &mut RlpStream)
    where
        A: AsRef<[u8]>,
        B: AsRef<[u8]>,
        H: Hasher,
{
    let inlen = input.len();

    // in case of empty slice, just append empty data
    if inlen == 0 {
        stream.append_empty_data();
        return;
    }

    // take slices
    let key: &[u8] = &input[0].0.as_ref();
    let value: &[u8] = &input[0].1.as_ref();

    // if the slice contains just one item, append the suffix of the key
    // and then append value
    if inlen == 1 {
        stream.begin_list(2);
        stream.append_iter(hex_prefix_encode(&key[pre_len..], true));
        stream.append(&value);
        return;
    }

    // get length of the longest shared prefix in slice keys
    let shared_prefix = input.iter()
        // skip first tuple
        .skip(1)
        // get minimum number of shared nibbles between first and each successive
        .fold(key.len(), | acc, &(ref k, _) | {
            cmp::min(shared_prefix_len(key, k.as_ref()), acc)
        });

    // if shared prefix is higher than current prefix append its
    // new part of the key to the stream
    // then recursively append suffixes of all items who had this key
    if shared_prefix > pre_len {
        stream.begin_list(2);
        stream.append_iter(hex_prefix_encode(&key[pre_len..shared_prefix], false));
        hash256aux::<H, _, _>(input, shared_prefix, stream);
        return;
    }

    // an item for every possible nibble/suffix
    // + 1 for data
    stream.begin_list(17);

    // if first key len is equal to prefix_len, move to next element
    let mut begin = match pre_len == key.len() {
        true => 1,
        false => 0
    };

    // iterate over all possible nibbles
    for i in 0..16 {
        // count how many successive elements have same next nibble
        let len = match begin < input.len() {
            true => input[begin..].iter()
                .take_while(| pair | pair.0.as_ref()[pre_len] == i )
                .count(),
            false => 0
        };

        // if at least 1 successive element has the same nibble
        // append their suffixes
        match len {
            0 => { stream.append_empty_data(); },
            _ => hash256aux::<H, _, _>(&input[begin..(begin + len)], pre_len + 1, stream)
        }
        begin += len;
    }

    // if fist key len is equal prefix, append its value
    match pre_len == key.len() {
        true => { stream.append(&value); },
        false => { stream.append_empty_data(); }
    };
}

fn hash256aux<H, A, B>(input: &[(A, B)], pre_len: usize, stream: &mut RlpStream)
    where
        A: AsRef<[u8]>,
        B: AsRef<[u8]>,
        H: Hasher,
{
    let mut s = RlpStream::new();
    hash256rlp::<H, _, _>(input, pre_len, &mut s);
    let out = s.out();
    match out.len() {
        0..=31 => stream.append_raw(&out, 1),
        _ => stream.append(&H::hash(&out).as_ref())
    };
}

#[cfg(test)]
mod tests {
    extern crate ethereum_types;
    use super::{trie_root, shared_prefix_len, hex_prefix_encode};
    use keccak_hasher::KeccakHasher;
    use self::ethereum_types::H256;

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
        let a = vec![1,2,3,4,5,6];
        let b = vec![4,2,3,4,5,6];
        assert_eq!(shared_prefix_len(&a, &b), 0);
    }

    #[test]
    fn test_shared_prefix2() {
        let a = vec![1,2,3,3,5];
        let b = vec![1,2,3];
        assert_eq!(shared_prefix_len(&a, &b), 3);
    }

    #[test]
    fn test_shared_prefix3() {
        let a = vec![1,2,3,4,5,6];
        let b = vec![1,2,3,4,5,6];
        assert_eq!(shared_prefix_len(&a, &b), 6);
    }
}