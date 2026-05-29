// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use lore_revision::interface::LoreString;

#[test]
fn lore_string_serde() {
    let serialize_deserialize =
        |value| serde_json::from_slice(&serde_json::to_vec(value).unwrap()).unwrap();

    let abc = LoreString::from("abc");
    let serde_abc = serialize_deserialize(&abc);
    assert_eq!(abc, serde_abc);

    let escaped_character = LoreString::from("ab\nc");
    let serde_escaped_character = serialize_deserialize(&escaped_character);
    assert_eq!(escaped_character, serde_escaped_character);

    let empty_string = LoreString::from("");
    let serde_empty_string = serialize_deserialize(&empty_string);
    assert_eq!(empty_string, serde_empty_string);

    let null_string = LoreString {
        string: std::ptr::null(),
        length: 0,
    };
    let serde_null_string = serialize_deserialize(&null_string);
    assert_eq!(null_string, serde_null_string);
}
