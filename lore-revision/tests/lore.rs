// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
#[cfg(test)]
mod tests {
    use lore_revision::lore::*;
    use lore_revision::lore_info;
    use tokio::task::JoinSet;

    include!("helper.rs");

    #[tokio::test]
    async fn test_execution_context() {
        let execution = setup_test_execution();

        assert!(
            !execution.globals().correlation_id.is_empty(),
            "Execution context has empty correlation id"
        );

        LORE_CONTEXT
            .scope(execution.clone(), async move {
                let execution = execution_context();
                lore_info!("Outer execution context is fine");

                async move {
                    let _execution = execution_context();
                    lore_info!("Inner async block execution context is fine");
                    Ok::<(), String>(())
                }
                .await?;

                let mut tasks: JoinSet<Result<(), String>> = JoinSet::new();
                #[allow(clippy::disallowed_methods)]
                tasks.spawn(LORE_CONTEXT.scope(execution.clone(), async move {
                    let _execution = execution_context();
                    lore_info!("Inner spawned task execution context is fine");
                    Ok(())
                }));

                while let Some(result) = tasks.join_next().await {
                    result.map_err(|err| err.to_string())??;
                }

                Ok::<(), String>(())
            })
            .await
            .expect("Execution context test task local key failed");
    }
}
