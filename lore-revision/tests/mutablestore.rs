// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
#[cfg(test)]
mod tests {
    use lore_base::runtime::LORE_CONTEXT;
    use lore_base::types::Hash;
    use lore_base::types::KeyType;
    use lore_revision::lore::RepositoryId;

    include!("helper.rs");

    #[tokio::test]
    async fn compare_and_swap() {
        let (_, store, execution) = test_store_create().await.expect("Failed to create stores");

        LORE_CONTEXT.scope(execution.clone(), async move {
            let repository = RepositoryId::from([0; 16]);
            let zero_value = Hash::default();
            let base_key = Hash::from(rand::random::<[u8; 32]>());
            let base_value = Hash::from(rand::random::<[u8; 32]>());
            store.clone()
                .store(repository, base_key, base_value, KeyType::Untyped)
                .await
                .expect("Failed to store value");

            let updated_value = Hash::from(rand::random::<[u8; 32]>());
            assert_eq!(
                store.clone()
                    .compare_and_swap(repository, base_key, base_value, updated_value, KeyType::Untyped)
                    .await
                    .expect("Failed to compare-and-swap"),
                base_value,
                "Compare-and-swap did not return previous value as expected"
            );

            let fail_value = Hash::from(rand::random::<[u8; 32]>());
            assert_eq!(
                store.clone()
                    .compare_and_swap(repository, base_key, base_value, fail_value, KeyType::Untyped).await
                    .expect("Failed to compare-and-swap"),
                updated_value,
                "Compare-and-swap with outdated value did not return updated previous value as expected"
            );

            let new_key = Hash::from(rand::random::<[u8; 32]>());
            let new_value = Hash::from(rand::random::<[u8; 32]>());
            assert_eq!(
                store.clone()
                    .compare_and_swap(repository, new_key, fail_value, new_value, KeyType::Untyped).await
                    .expect("Failed to compare-and-swap"),
                zero_value,
                "Compare-and-swap with new key and non-zero expected value did not return zero value as expected"
            );
            assert_eq!(
                store.clone()
                    .compare_and_swap(repository, new_key, zero_value, new_value, KeyType::Untyped)
                    .await
                    .expect("Failed to compare-and-swap"),
                zero_value,
                "Compare-and-swap with new key and zero expected did not return zero value as expected"
            );
            assert_eq!(
                store.clone()
                    .compare_and_swap(repository, new_key, new_value, updated_value, KeyType::Untyped).await
                    .expect("Failed to compare-and-swap"),
                new_value,
                "Compare-and-swap with existing key and new value did not return expected previous value"
            );
            assert_eq!(
                store.clone()
                    .load(repository, new_key, KeyType::Untyped)
                    .await
                    .expect("Failed to load"),
                updated_value,
                "Load after compare-and-swap with new key did not return updated value as expected"
            );
        }).await;
    }
}
