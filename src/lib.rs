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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[macro_use]
extern crate hex_literal;

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

extern crate hash_db;

#[cfg(feature = "std")]
pub use std::collections::BTreeMap;
#[cfg(feature = "std")]
pub use std::cmp;
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
pub use core::cmp;
pub use hash_db::Hasher;

#[cfg(test)]
mod tests;

//mod proof;
mod trie_root;
use trie_root::trie_root;

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


///// Generates a trie root hash for a vector of values
///// and return a list of nodes as proof
//pub fn build_trie_proof<H, I>(input: I, index: u8) -> (H::Out, Vec<u8>)
//    where
//        I: IntoIterator,
//        I::Item: AsRef<[u8]>,
//        H: Hasher,
//        <H as hash_db::Hasher>::Out: cmp::Ord,
//{
//    let root = trie_root::<H, _, _, _>(input.into_iter().enumerate().map(|(i, v)| (rlp::encode(&i), v)));
//    (root, vec![])
//}


