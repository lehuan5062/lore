// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
mod tests {
    use lore_base::runtime::LORE_CONTEXT;
    use lore_base::types::Fragment;
    use lore_base::types::FragmentFlags;
    use lore_storage::CompressionMode;
    use lore_storage::compress;
    use lore_storage::hash;
    use rand::Rng;

    include!("helper.rs");

    #[test]
    fn hash() {
        let mut data = [0u8; 100];
        rand::rng().fill(&mut data[..]);

        let hash = hash::hash_slice(&data);
        let ref_hash = blake3::hash(&data);
        assert_eq!(hash.data(), ref_hash.as_bytes());
    }

    #[tokio::test]
    async fn hash_uncompressed_fragment() {
        let mut data = [0u8; 100];
        rand::rng().fill(&mut data[..]);

        let execution = setup_test_execution();
        LORE_CONTEXT
            .scope(execution.clone(), async move {
                let fragment = Fragment {
                    flags: FragmentFlags::PayloadStoredLocal.bits(),
                    size_payload: data.len() as u32,
                    size_content: data.len() as u64,
                };

                let hash = hash::hash_fragment(fragment, &data).expect("Hash fragment failed");
                let ref_hash = hash::hash_slice(&data);
                assert_eq!(hash.data(), ref_hash.data());
            })
            .await;
    }

    #[tokio::test]
    async fn hash_compressed_fragment() {
        let mut data = [0u8; 1000];
        for (i, item) in data.iter_mut().enumerate() {
            *item = (i % 10) as u8;
        }

        let execution = setup_test_execution();
        LORE_CONTEXT
            .scope(execution.clone(), async move {
                let fragment = Fragment {
                    flags: FragmentFlags::PayloadStoredLocal.bits(),
                    size_payload: data.len() as u32,
                    size_content: data.len() as u64,
                };

                let (compressed_fragment, compressed_buffer) =
                    compress::compress(fragment, &data, CompressionMode::Oodle)
                        .expect("Compression failed");
                let compressed_data = compressed_buffer.as_ref();
                assert!((compressed_fragment.size_payload as usize) <= compressed_data.len());
                assert_eq!(compressed_fragment.size_content, fragment.size_content);
                assert!(
                    (compressed_fragment.size_payload as usize)
                        < (compressed_fragment.size_content as usize)
                );

                let hash = hash::hash_fragment(compressed_fragment, compressed_data)
                    .expect("Hash compressed fragment failed");
                let ref_hash = hash::hash_slice(&data);
                assert_eq!(hash, ref_hash);
            })
            .await;
    }
}
