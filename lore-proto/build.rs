// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::env;
use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("No manifest dir set");
    let output_dir = PathBuf::from(crate_dir).join("src").join("grpc");

    let mut config = tonic_prost_build::Config::new();
    config.enable_type_names();
    // Use Bytes for buffers instead of Vec
    config.bytes(["."]);

    tonic_prost_build::configure()
        .out_dir(&output_dir)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_with_config(
            config,
            &[
                "./proto/model.proto",
                "./proto/admin.proto",
                "./proto/lock.proto",
                "./proto/epic_events.proto",
                "./proto/lore_notification.proto",
                "./proto/notification.proto",
                "./proto/replication.proto",
            ],
            &["./proto"],
        )?;

    // lore.model.v1 — shared base types
    let mut config = tonic_prost_build::Config::new();
    config.enable_type_names();
    config.bytes(["."]);

    tonic_prost_build::configure()
        .out_dir(&output_dir)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_with_config(config, &["./proto/lore/model/v1/model.proto"], &["./proto"])?;

    // lore.storage.v1 — storage service, references lore.model.v1 via extern_path
    let mut config = tonic_prost_build::Config::new();
    config.enable_type_names();
    config.bytes(["."]);
    config.extern_path(".lore.model.v1", "crate::lore::model::v1");

    tonic_prost_build::configure()
        .out_dir(&output_dir)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_with_config(
            config,
            &["./proto/lore/storage/v1/storage.proto"],
            &["./proto"],
        )?;

    // lore.revision.v1 — baseline revision-graph service, references lore.model.v1
    let mut config = tonic_prost_build::Config::new();
    config.enable_type_names();
    config.bytes(["."]);
    config.extern_path(".lore.model.v1", "crate::lore::model::v1");

    tonic_prost_build::configure()
        .out_dir(&output_dir)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_with_config(
            config,
            &["./proto/lore/revision/v1/revision.proto"],
            &["./proto"],
        )?;

    // lore.repository.v1 — repository-management service, references lore.model.v1
    let mut config = tonic_prost_build::Config::new();
    config.enable_type_names();
    config.bytes(["."]);
    config.extern_path(".lore.model.v1", "crate::lore::model::v1");

    tonic_prost_build::configure()
        .out_dir(&output_dir)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_with_config(
            config,
            &["./proto/lore/repository/v1/repository.proto"],
            &["./proto"],
        )?;

    // lore.thin_client.v1 — thin-client presentation helpers, references lore.model.v1.
    // model.proto and thin_client.proto share the same package and are compiled
    // together into a single generated module.
    let mut config = tonic_prost_build::Config::new();
    config.enable_type_names();
    config.bytes(["."]);
    config.extern_path(".lore.model.v1", "crate::lore::model::v1");

    tonic_prost_build::configure()
        .out_dir(&output_dir)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_with_config(
            config,
            &[
                "./proto/lore/thin_client/v1/model.proto",
                "./proto/lore/thin_client/v1/thin_client.proto",
            ],
            &["./proto"],
        )?;

    // lore.environment.v1 — server-side environment discovery service. Self-contained — declares its own messages
    let mut config = tonic_prost_build::Config::new();
    config.enable_type_names();
    config.bytes(["."]);

    tonic_prost_build::configure()
        .out_dir(&output_dir)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_with_config(
            config,
            &["./proto/lore/environment/v1/environment.proto"],
            &["./proto"],
        )?;

    let mut config = tonic_prost_build::Config::new();
    // Use Bytes for buffers instead of Vec
    config.bytes(["."]);

    tonic_prost_build::configure()
        .out_dir(&output_dir)
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_server(false)
        .compile_with_config(
            config,
            &["./proto/auth_api.proto", "./proto/rebac_api.proto"],
            &["./proto"],
        )?;

    Ok(())
}
