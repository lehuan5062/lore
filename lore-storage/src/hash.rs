// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use crate::FragmentFlags;
use crate::compress::FRAGMENT_SIZE_THRESHOLD;
use crate::compress::FragmentError;
use crate::compress::decompress;
use crate::types::Fragment;
use crate::types::Hash;

/// Hash a function name with a domain salt prefix.
pub fn hash_function(salt: &[u8], function: &str) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(salt);
    hasher.update(function.as_bytes());
    hasher.finalize().as_bytes().into()
}

/// Hash a function name with a domain salt prefix and a single byte-slice argument.
pub fn hash_function_arg_slice(salt: &[u8], function: &str, arg: &[u8]) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(salt);
    hasher.update(function.as_bytes());
    hasher.update(arg);
    hasher.finalize().as_bytes().into()
}

/// Hash a function name with a domain salt prefix and a single string argument.
pub fn hash_function_arg(salt: &[u8], function: &str, arg: &str) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(salt);
    hasher.update(function.as_bytes());
    hasher.update(arg.as_bytes());
    hasher.finalize().as_bytes().into()
}

/// Hash a function name with a domain salt prefix and two string arguments.
pub fn hash_function_args(salt: &[u8], function: &str, first_arg: &str, second_arg: &str) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(salt);
    hasher.update(function.as_bytes());
    hasher.update(first_arg.as_bytes());
    hasher.update(second_arg.as_bytes());
    hasher.finalize().as_bytes().into()
}

/// Hash a function name with a domain salt prefix and two byte-slice arguments.
pub fn hash_function_args_slice(
    salt: &[u8],
    function: &str,
    first_arg: &[u8],
    second_arg: &[u8],
) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(salt);
    hasher.update(function.as_bytes());
    hasher.update(first_arg);
    hasher.update(second_arg);
    hasher.finalize().as_bytes().into()
}

/// Hash a function name with a domain salt prefix and a variable number of string arguments.
pub fn hash_function_strs_slice(salt: &[u8], function: &str, args: &[&str]) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(salt);
    hasher.update(function.as_bytes());
    for arg in args {
        hasher.update(arg.as_bytes());
    }
    hasher.finalize().as_bytes().into()
}

/// Hash a raw data slice using blake3.
pub fn hash_slice(data: &[u8]) -> Hash {
    blake3::hash(data).as_bytes().into()
}

/// Hash a fragment's content if it matches the payload metadata, decompressing first if needed
pub fn hash_fragment(fragment: Fragment, data: &[u8]) -> Result<Hash, FragmentError> {
    if fragment.size_payload as usize != data.len() {
        return Err(FragmentError::internal(
            "Invalid payload size for fragment hash",
        ));
    }

    if (fragment.flags & FragmentFlags::PayloadCompressed) == 0 {
        return Ok(hash_slice(data));
    }

    debug_assert!((fragment.flags & FragmentFlags::PayloadFragmented) == 0);
    debug_assert!(fragment.size_content as usize <= FRAGMENT_SIZE_THRESHOLD);

    let (_, decompressed) = decompress(fragment, data)?;

    if fragment.size_content as usize != decompressed.len() {
        return Err(FragmentError::internal(
            "Invalid content size for fragment hash after decompression",
        ));
    }

    Ok(hash_slice(decompressed.as_ref()))
}

/// 64-bit string hash type, used for node name lookups.
pub type StringHash = u64;

/// Compute the 64-bit xxh3 hash of the lowercase form of a string.
pub fn hash_string(string: &str) -> StringHash {
    let lowercase_string = string.to_lowercase();
    xxhash_rust::xxh3::xxh3_64(lowercase_string.as_bytes())
}

/// Zero-alloc xxh3 of raw string-like bytes (same digest family as [`hash_string`] without the lowercasing, distinct from the blake3 [`hash_slice`]).
pub fn hash_string_bytes(bytes: &[u8]) -> StringHash {
    xxhash_rust::xxh3::xxh3_64(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_fragment_uncompressed_ok() {
        let data = b"hello world";
        let fragment = Fragment {
            flags: 0,
            size_payload: data.len() as u32,
            size_content: data.len() as u64,
        };
        let result = hash_fragment(fragment, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), hash_slice(data));
    }

    #[test]
    fn hash_fragment_uncompressed_deterministic() {
        let data = b"deterministic content";
        let fragment = Fragment {
            flags: 0,
            size_payload: data.len() as u32,
            size_content: data.len() as u64,
        };
        let hash1 = hash_fragment(fragment, data).unwrap();
        let hash2 = hash_fragment(fragment, data).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn hash_fragment_different_data_different_hash() {
        let data_a = b"content a";
        let data_b = b"content b";
        let frag_a = Fragment {
            flags: 0,
            size_payload: data_a.len() as u32,
            size_content: data_a.len() as u64,
        };
        let frag_b = Fragment {
            flags: 0,
            size_payload: data_b.len() as u32,
            size_content: data_b.len() as u64,
        };
        assert_ne!(
            hash_fragment(frag_a, data_a).unwrap(),
            hash_fragment(frag_b, data_b).unwrap()
        );
    }

    #[test]
    fn hash_fragment_payload_size_mismatch() {
        let data = b"hello";
        let fragment = Fragment {
            flags: 0,
            size_payload: data.len() as u32 + 1,
            size_content: data.len() as u64,
        };
        assert!(hash_fragment(fragment, data).is_err());
    }

    #[test]
    fn hash_fragment_empty_payload() {
        let data: &[u8] = b"";
        let fragment = Fragment {
            flags: 0,
            size_payload: 0,
            size_content: 0,
        };
        let result = hash_fragment(fragment, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), hash_slice(data));
    }

    #[test]
    fn hash_fragment_compressed_payload_size_mismatch() {
        let data = b"short";
        let fragment = Fragment {
            flags: crate::FragmentFlags::PayloadCompressedLZ4.into(),
            size_payload: data.len() as u32 + 5,
            size_content: 100,
        };
        assert!(hash_fragment(fragment, data).is_err());
    }

    #[test]
    fn hash_fragment_compressed_invalid_data() {
        let data = b"this is not valid lz4 compressed data!!";
        let fragment = Fragment {
            flags: crate::FragmentFlags::PayloadCompressedLZ4.into(),
            size_payload: data.len() as u32,
            size_content: 100,
        };
        assert!(hash_fragment(fragment, data).is_err());
    }

    #[test]
    fn hash_function_different_salts_produce_different_keys() {
        let hash_urc = hash_function(b"urc", "test_function");
        let hash_lore = hash_function(b"lore", "test_function");
        assert_ne!(hash_urc, hash_lore);
    }

    #[test]
    fn hash_function_same_salt_is_deterministic() {
        let hash1 = hash_function(b"urc", "test_function");
        let hash2 = hash_function(b"urc", "test_function");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn hash_function_arg_with_salt() {
        let hash_urc = hash_function_arg(b"urc", "func", "arg");
        let hash_lore = hash_function_arg(b"lore", "func", "arg");
        assert_ne!(hash_urc, hash_lore);
    }

    #[test]
    fn hash_function_compressed_roundtrip() {
        let original = vec![0u8; 4096];
        let original = original.as_slice();
        let uncompressed_fragment = Fragment {
            flags: 0,
            size_payload: original.len() as u32,
            size_content: original.len() as u64,
        };
        let (compressed_fragment, compressed_data) =
            crate::compress::compress(uncompressed_fragment, original, crate::CompressionMode::Lz4)
                .unwrap();
        let hash = hash_fragment(compressed_fragment, compressed_data.as_ref()).unwrap();
        assert_eq!(hash, hash_slice(original));
    }
}
