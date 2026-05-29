// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use bytes::Bytes;
pub(crate) use lore_base::types::FragmentFlags;

use crate::lore::Fragment;

pub fn generate_random() -> (Fragment, crate::lore::Address, Bytes) {
    let payload = rand::random::<[u8; 32]>();
    let hash = crate::lore::Hash::hash_buffer(payload.as_slice());
    let context = rand::random::<crate::lore::Context>();

    (
        Fragment {
            flags: 0,
            size_payload: payload.len() as u32,
            size_content: payload.len() as u64,
        },
        crate::lore::Address { hash, context },
        Bytes::copy_from_slice(&payload),
    )
}
