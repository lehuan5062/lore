// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
#[cfg(test)]
mod tests {
    use lore_base::types::Address;
    use lore_base::types::Context;
    use lore_base::types::Hash;
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct Foo {
        address: Address,
        context: Context,
        hash: Hash,
    }

    impl Default for Foo {
        fn default() -> Self {
            Foo {
                address: Address {
                    hash: Hash::from([5; 32]),
                    context: Context::from([6; 16]),
                },
                hash: Hash::from([7; 32]),
                context: Context::from([8; 16]),
            }
        }
    }

    #[test]
    fn test_serde_json() {
        let item = Foo::default();
        let out = serde_json::to_string(&item).unwrap();
        let expected = serde_json::json!({
            "address": "0505050505050505050505050505050505050505050505050505050505050505-06060606060606060606060606060606",
            "context": "08080808080808080808080808080808",
            "hash": "0707070707070707070707070707070707070707070707070707070707070707",
        });
        assert_eq!(out, expected.to_string());

        let deserialized: Foo = serde_json::from_str(&expected.to_string()).unwrap();
        assert_eq!(deserialized, item);
    }

    #[test]
    fn test_toml() {
        let item = Foo::default();
        let out = toml::to_string(&item).unwrap();
        let expected = "address = \"0505050505050505050505050505050505050505050505050505050505050505-06060606060606060606060606060606\"\
            \ncontext = \"08080808080808080808080808080808\"\
            \nhash = \"0707070707070707070707070707070707070707070707070707070707070707\"\n";
        assert_eq!(out, expected.to_string());
        let deserialized: Foo = toml::from_str(expected).unwrap();
        assert_eq!(deserialized, item);
    }
}
