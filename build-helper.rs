struct LoreVergen {
    pub revision_number: String,
    pub revision: String,
    pub warning: Vec<String>,
}

impl Default for LoreVergen {
    fn default() -> Self {
        // Invoke Lore CLI to get the revision number and signature
        let result = std::process::Command::new("lore")
            .arg("revision")
            .arg("info")
            .arg("--offline")
            .current_dir("..")
            .output();
        let Ok(output) = result else {
            let err = format!(
                "Failed to execute Lore to get revision information, unknown version generated: {}",
                result.unwrap_err()
            );
            return Self {
                revision_number: "0".to_string(),
                revision: "unknown".to_string(),
                warning: vec![err],
            };
        };

        let mut revision = String::default();
        let mut revision_number = String::default();

        let stdout = std::str::from_utf8(&output.stdout).unwrap_or_default();
        for line in stdout.lines() {
            if line.starts_with("Revision ") {
                if let Some((_, rev)) = line.split_once(':') {
                    // New CLI format 'Revision : $num'
                    revision_number = rev.trim().to_string();
                } else if let Some((_, rev)) = line.split_once(' ') {
                    // Old format 'Revision $num'
                    revision_number = rev.trim().to_string();
                }
            } else if line.starts_with("Signature: ") {
                // Old CLI format 'Signature: $sig'
                if let Some((_, rev)) = line.split_once(' ') {
                    revision = rev.trim()[..8].to_string();
                }
            } else if line.starts_with("Signature ") {
                // New CLI format 'Signature : $sig'
                if let Some((_, rev)) = line.split_once(':') {
                    revision = rev.trim()[..8].to_string();
                }
            }
        }

        if revision.is_empty() || revision_number.is_empty() {
            Self {
                revision_number: "0".to_string(),
                revision: "unknown".to_string(),
                warning: vec![
                    "Failed to execute Lore to get revision information, no data extracted"
                        .to_string(),
                ],
            }
        } else {
            Self {
                revision_number,
                revision,
                warning: vec![],
            }
        }
    }
}

impl vergen::AddCustomEntries<&str, String> for LoreVergen {
    fn add_calculated_entries(
        &self,
        _idempotent: bool,
        cargo_rustc_env_map: &mut std::collections::BTreeMap<&str, String>,
        _cargo_rerun_if_changed: &mut vergen::CargoRerunIfChanged,
        cargo_warning: &mut vergen::CargoWarning,
    ) -> Result<(), anyhow::Error> {
        if !self.warning.is_empty() {
            cargo_warning.extend_from_slice(&self.warning);
        }

        cargo_rustc_env_map.insert("VERGEN_LORE_REVISION_NUMBER", self.revision_number.clone());
        cargo_rustc_env_map.insert("VERGEN_LORE_REVISION", self.revision.clone());
        Ok(())
    }

    fn add_default_entries(
        &self,
        _config: &vergen::DefaultConfig,
        _cargo_rustc_env_map: &mut std::collections::BTreeMap<&str, String>,
        _cargo_rerun_if_changed: &mut vergen::CargoRerunIfChanged,
        _cargo_warning: &mut vergen::CargoWarning,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

#[allow(dead_code)]
fn profile_dir() -> String {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let path = std::path::PathBuf::from(out_dir);
    let mut path = path.as_path();
    while let Some(name) = path.file_name() {
        if name == "build" {
            return path
                .parent()
                .expect("No parent of build")
                .display()
                .to_string();
        }
        path = path.parent().expect("Reached root of filesystem");
    }
    panic!("OUT_DIR did not contain a build directory");
}

#[allow(dead_code)]
fn profile_name() -> String {
    let profile_dir = profile_dir();
    std::path::PathBuf::from(profile_dir)
        .file_name()
        .expect("Failed to get profile name")
        .display()
        .to_string()
}
