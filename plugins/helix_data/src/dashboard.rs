//! Contract validation dashboard for Helix data integrity.
//!
//! Validates that helix3d TOML files parse correctly, cross-references
//! are intact, and balance overlays reference valid fields.

use crate::api_health;
use crate::registries::HelixRegistries;
use crate::toml_loader;
use dj_engine::data::{ValidationIssue, ValidationSeverity};
use helix_data::Registry;
use std::path::Path;

/// Run all dashboard validation checks and collect issues.
pub fn validate_helix_registries(
    registries: &HelixRegistries,
    helix3d_dir: Option<&Path>,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_toml_coverage(helix3d_dir, issues);
    validate_cross_references(registries, issues);
    validate_localization(registries, issues);
    emit_entity_count_summary(registries, issues);
    validate_api_health(registries, issues);
    validate_data_freshness(helix3d_dir, issues);
    validate_remote_sample(registries, issues);
}

/// Check that all 22 expected TOML files exist.
fn validate_toml_coverage(helix3d_dir: Option<&Path>, issues: &mut Vec<ValidationIssue>) {
    let Some(dir) = helix3d_dir else {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            code: "helix_no_toml_dir".into(),
            source_kind: None,
            source_id: None,
            field_path: None,
            message: "No helix3d directory configured; TOML coverage not checked.".into(),
            related_refs: Vec::new(),
        });
        return;
    };

    let missing = toml_loader::missing_toml_files(dir);
    for file in &missing {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            code: "helix_missing_toml".into(),
            source_kind: None,
            source_id: None,
            field_path: Some(file.clone()),
            message: format!(
                "Expected TOML file '{}' not found in helix3d directory.",
                file
            ),
            related_refs: Vec::new(),
        });
    }

    // Try strict load to detect parse errors
    for file in toml_loader::discover_toml_files(dir) {
        let result: Result<Registry<toml::Value>, _> = toml_loader::load_registry(dir, &file);
        if let Err(toml_loader::HelixLoadError::Toml { source, .. }) = result {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "helix_toml_parse_error".into(),
                source_kind: None,
                source_id: None,
                field_path: Some(file.clone()),
                message: format!("Schema mismatch in '{}': {}", file, source.message()),
                related_refs: Vec::new(),
            });
        }
    }
}

/// Check cross-reference integrity: mob abilities exist, quest prerequisites exist, etc.
fn validate_cross_references(registries: &HelixRegistries, issues: &mut Vec<ValidationIssue>) {
    // Mob abilities → abilities registry
    for (mob_id, mob) in registries.mobs.iter() {
        for ability_id in &mob.abilities {
            if !registries.abilities.contains(ability_id) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("mobs".into()),
                    source_id: Some(mob_id.to_string()),
                    field_path: Some("abilities".into()),
                    message: format!(
                        "Mob '{}' references ability '{}' which does not exist.",
                        mob_id, ability_id
                    ),
                    related_refs: vec![ability_id.clone()],
                });
            }
        }

        // Mob zone_ids → zones registry
        for zone_id in &mob.zone_ids {
            if !registries.zones.contains(zone_id) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("mobs".into()),
                    source_id: Some(mob_id.to_string()),
                    field_path: Some("zone_ids".into()),
                    message: format!(
                        "Mob '{}' references zone '{}' which does not exist.",
                        mob_id, zone_id
                    ),
                    related_refs: vec![zone_id.clone()],
                });
            }
        }
    }

    // Quest prerequisite_quests → quests registry
    for (quest_id, quest) in registries.quests.iter() {
        for prereq in &quest.prerequisite_quests {
            if !registries.quests.contains(prereq) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("quests".into()),
                    source_id: Some(quest_id.to_string()),
                    field_path: Some("prerequisite_quests".into()),
                    message: format!(
                        "Quest '{}' requires prerequisite '{}' which does not exist.",
                        quest_id, prereq
                    ),
                    related_refs: vec![prereq.clone()],
                });
            }
        }
    }

    // Mob loot_ids → items registry
    for (mob_id, mob) in registries.mobs.iter() {
        for loot_id in &mob.loot_ids {
            if !registries.items.contains(loot_id) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("mobs".into()),
                    source_id: Some(mob_id.to_string()),
                    field_path: Some("loot_ids".into()),
                    message: format!(
                        "Mob '{}' references loot item '{}' which does not exist.",
                        mob_id, loot_id
                    ),
                    related_refs: vec![loot_id.clone()],
                });
            }
        }
    }

    // Class abilities → abilities registry
    for (class_id, class_data) in registries.class_data.iter() {
        for ability_id in &class_data.abilities {
            if !registries.abilities.contains(ability_id) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("class_data".into()),
                    source_id: Some(class_id.to_string()),
                    field_path: Some("abilities".into()),
                    message: format!(
                        "Class '{}' references ability '{}' which does not exist.",
                        class_id, ability_id
                    ),
                    related_refs: vec![ability_id.clone()],
                });
            }
        }
    }

    // Talent → class_data references
    for (talent_id, talent) in registries.talents.iter() {
        if !talent.class_id.is_empty() && !registries.class_data.contains(&talent.class_id) {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                code: "helix_broken_ref".into(),
                source_kind: Some("talents".into()),
                source_id: Some(talent_id.to_string()),
                field_path: Some("class_id".into()),
                message: format!(
                    "Talent '{}' references class '{}' which does not exist.",
                    talent_id, talent.class_id
                ),
                related_refs: vec![talent.class_id.clone()],
            });
        }
    }

    // PVP zone_id → zones registry
    for (pvp_id, pvp) in registries.pvp.iter() {
        if let Some(zone_id) = &pvp.zone_id {
            if !zone_id.is_empty() && !registries.zones.contains(zone_id) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("pvp".into()),
                    source_id: Some(pvp_id.to_string()),
                    field_path: Some("zone_id".into()),
                    message: format!(
                        "PvP '{}' references zone '{}' which does not exist.",
                        pvp_id, zone_id
                    ),
                    related_refs: vec![zone_id.clone()],
                });
            }
        }
    }

    // Loot table entries → items registry
    for (table_id, table) in registries.loot_tables.iter() {
        for entry in &table.entries {
            if !entry.item_id.is_empty() && !registries.items.contains(&entry.item_id) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("loot_tables".into()),
                    source_id: Some(table_id.to_string()),
                    field_path: Some("entries.item_id".into()),
                    message: format!(
                        "Loot table '{}' references item '{}' which does not exist.",
                        table_id, entry.item_id
                    ),
                    related_refs: vec![entry.item_id.clone()],
                });
            }
        }
    }

    // Quest objective → zones, items
    for (obj_id, obj) in registries.quest_objectives.iter() {
        if let Some(zone_id) = &obj.zone_id {
            if !zone_id.is_empty() && !registries.zones.contains(zone_id) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("quest_objectives".into()),
                    source_id: Some(obj_id.to_string()),
                    field_path: Some("zone_id".into()),
                    message: format!(
                        "Quest objective '{}' references zone '{}' which does not exist.",
                        obj_id, zone_id
                    ),
                    related_refs: vec![zone_id.clone()],
                });
            }
        }
        if let Some(item_id) = &obj.item_id {
            if !item_id.is_empty() && !registries.items.contains(item_id) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("quest_objectives".into()),
                    source_id: Some(obj_id.to_string()),
                    field_path: Some("item_id".into()),
                    message: format!(
                        "Quest objective '{}' references item '{}' which does not exist.",
                        obj_id, item_id
                    ),
                    related_refs: vec![item_id.clone()],
                });
            }
        }
    }

    // Faction allied/enemy → self-referential
    for (faction_id, faction) in registries.factions.iter() {
        for allied in &faction.allied_factions {
            if !allied.is_empty() && !registries.factions.contains(allied) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("factions".into()),
                    source_id: Some(faction_id.to_string()),
                    field_path: Some("allied_factions".into()),
                    message: format!(
                        "Faction '{}' references allied faction '{}' which does not exist.",
                        faction_id, allied
                    ),
                    related_refs: vec![allied.clone()],
                });
            }
        }
        for enemy in &faction.enemy_factions {
            if !enemy.is_empty() && !registries.factions.contains(enemy) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("factions".into()),
                    source_id: Some(faction_id.to_string()),
                    field_path: Some("enemy_factions".into()),
                    message: format!(
                        "Faction '{}' references enemy faction '{}' which does not exist.",
                        faction_id, enemy
                    ),
                    related_refs: vec![enemy.clone()],
                });
            }
        }
    }

    // Profession recipes → items registry
    for (prof_id, prof) in registries.professions.iter() {
        for recipe_item in &prof.recipes {
            if !recipe_item.is_empty() && !registries.items.contains(recipe_item) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("professions".into()),
                    source_id: Some(prof_id.to_string()),
                    field_path: Some("recipes".into()),
                    message: format!(
                        "Profession '{}' references recipe item '{}' which does not exist.",
                        prof_id, recipe_item
                    ),
                    related_refs: vec![recipe_item.clone()],
                });
            }
        }
    }

    // NPC quests → quests registry
    for (npc_id, npc) in registries.npcs.iter() {
        for quest_id in &npc.quests {
            if !registries.quests.contains(quest_id) {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "helix_broken_ref".into(),
                    source_kind: Some("npcs".into()),
                    source_id: Some(npc_id.to_string()),
                    field_path: Some("quests".into()),
                    message: format!(
                        "NPC '{}' references quest '{}' which does not exist.",
                        npc_id, quest_id
                    ),
                    related_refs: vec![quest_id.clone()],
                });
            }
        }
    }
}

/// Check that every entity has at least an English localized name.
fn validate_localization(registries: &HelixRegistries, issues: &mut Vec<ValidationIssue>) {
    macro_rules! check_locale {
        ($kind:expr, $registry:expr) => {
            for (id, entity) in $registry.iter() {
                if entity.base.name.en().is_empty() {
                    issues.push(ValidationIssue {
                        severity: ValidationSeverity::Warning,
                        code: "helix_missing_en_name".into(),
                        source_kind: Some($kind.into()),
                        source_id: Some(id.to_string()),
                        field_path: Some("name.en".into()),
                        message: format!("Entity '{}' in {} has no English name.", id, $kind),
                        related_refs: Vec::new(),
                    });
                }
            }
        };
    }

    check_locale!("abilities", registries.abilities);
    check_locale!("achievements", registries.achievements);
    check_locale!("auras", registries.auras);
    check_locale!("class_data", registries.class_data);
    check_locale!("consumables", registries.consumables);
    check_locale!("currencies", registries.currencies);
    check_locale!("equipment", registries.equipment);
    check_locale!("guilds", registries.guilds);
    check_locale!("inventory", registries.inventory);
    check_locale!("items", registries.items);
    check_locale!("mobs", registries.mobs);
    check_locale!("mounts", registries.mounts);
    check_locale!("npcs", registries.npcs);
    check_locale!("professions", registries.professions);
    check_locale!("pvp", registries.pvp);
    check_locale!("quests", registries.quests);
    check_locale!("raids", registries.raids);
    check_locale!("talents", registries.talents);
    check_locale!("titles", registries.titles);
    check_locale!("trade_goods", registries.trade_goods);
    check_locale!("weapon_skills", registries.weapon_skills);
    check_locale!("zones", registries.zones);
}

/// Emit entity count summary as Info-severity issues so the editor
/// validation view shows a quick data health overview.
fn emit_entity_count_summary(registries: &HelixRegistries, issues: &mut Vec<ValidationIssue>) {
    let total = registries.total_entities();
    if total == 0 {
        return;
    }

    issues.push(ValidationIssue {
        severity: ValidationSeverity::Info,
        code: "helix_entity_count".into(),
        source_kind: None,
        source_id: None,
        field_path: None,
        message: format!(
            "Helix registries: {} total entities across 22 kinds.",
            total
        ),
        related_refs: Vec::new(),
    });

    for (kind, count) in registries.summary() {
        if count > 0 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Info,
                code: "helix_entity_count".into(),
                source_kind: Some(kind.into()),
                source_id: None,
                field_path: None,
                message: format!("{}: {} entities", kind, count),
                related_refs: Vec::new(),
            });
        }
    }
}

/// Check standardization API health (opt-in, Info severity on failure).
fn validate_api_health(registries: &HelixRegistries, issues: &mut Vec<ValidationIssue>) {
    match api_health::check_api_health() {
        Some(mut report) => {
            report.entities_local = registries.total_entities();
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Info,
                code: "helix_api_health".into(),
                source_kind: None,
                source_id: None,
                field_path: None,
                message: format!(
                    "API Health: connected (remote: {} entities, local: {}, age: {} min, status: {})",
                    report.entities_remote,
                    report.entities_local,
                    report.data_age_minutes,
                    report.version,
                ),
                related_refs: Vec::new(),
            });
        }
        None => {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Info,
                code: "helix_api_health".into(),
                source_kind: None,
                source_id: None,
                field_path: None,
                message: "API Health: not running (port 6800 unreachable)".into(),
                related_refs: Vec::new(),
            });
        }
    }
}

/// Check data freshness by comparing local file age with remote API.
fn validate_data_freshness(helix3d_dir: Option<&Path>, issues: &mut Vec<ValidationIssue>) {
    let Some(dir) = helix3d_dir else {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Info,
            code: "helix_data_freshness".into(),
            source_kind: None,
            source_id: None,
            field_path: None,
            message: "Data Freshness: unknown (no helix3d directory)".into(),
            related_refs: Vec::new(),
        });
        return;
    };

    let report = api_health::check_data_freshness(dir);

    if report.needs_refresh {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            code: "helix_data_freshness".into(),
            source_kind: None,
            source_id: None,
            field_path: None,
            message: format!(
                "Data Freshness: local files are {} min old (consider re-import)",
                report.local_age_minutes,
            ),
            related_refs: Vec::new(),
        });
    } else {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Info,
            code: "helix_data_freshness".into(),
            source_kind: None,
            source_id: None,
            field_path: None,
            message: format!("Data Freshness: {} min old", report.local_age_minutes),
            related_refs: Vec::new(),
        });
    }
}

/// Validate a sample entity against the remote API.
fn validate_remote_sample(registries: &HelixRegistries, issues: &mut Vec<ValidationIssue>) {
    let remote_issues = api_health::validate_sample_against_api(registries);
    if remote_issues.is_empty() {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Info,
            code: "helix_remote_validation".into(),
            source_kind: None,
            source_id: None,
            field_path: None,
            message: "Remote Validation: sample valid (or API unavailable)".into(),
            related_refs: Vec::new(),
        });
    } else {
        for msg in &remote_issues {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                code: "helix_remote_validation".into(),
                source_kind: None,
                source_id: None,
                field_path: None,
                message: msg.clone(),
                related_refs: Vec::new(),
            });
        }
    }
}

/// Print a dashboard summary to stdout (for CLI use).
///
/// Renders a boxed dashboard with check-line summaries for TOML coverage,
/// cross-references, localization, API health, data freshness, and remote
/// validation. Detail lines follow the box for warnings/errors.
pub fn print_dashboard_summary(issues: &[ValidationIssue]) {
    let errors = issues
        .iter()
        .filter(|i| matches!(i.severity, ValidationSeverity::Error))
        .count();
    let warnings = issues
        .iter()
        .filter(|i| matches!(i.severity, ValidationSeverity::Warning))
        .count();

    // Derive check-line statuses from issues
    let toml_errors = issues
        .iter()
        .filter(|i| i.code == "helix_toml_parse_error" || i.code == "helix_missing_toml")
        .count();
    let toml_no_dir = issues.iter().any(|i| i.code == "helix_no_toml_dir");
    let broken_refs = issues
        .iter()
        .filter(|i| i.code == "helix_broken_ref")
        .count();
    let missing_en = issues
        .iter()
        .filter(|i| i.code == "helix_missing_en_name")
        .count();
    let entity_total = issues
        .iter()
        .find(|i| i.code == "helix_entity_count" && i.source_kind.is_none())
        .map(|i| i.message.clone());

    let api_issue = issues.iter().find(|i| i.code == "helix_api_health");
    let freshness_issue = issues.iter().find(|i| i.code == "helix_data_freshness");
    let remote_val_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.code == "helix_remote_validation")
        .collect();

    let w = 46;
    let bar = format!("+{}+", "-".repeat(w));

    println!("{}", bar);
    println!("| {:^width$} |", "HELIX DATA DASHBOARD", width = w);
    println!("{}", bar);

    // TOML Coverage
    if toml_no_dir {
        println!(
            "| {:width$} |",
            "  [--] TOML Coverage     no dir configured",
            width = w
        );
    } else if toml_errors == 0 {
        println!(
            "| {:width$} |",
            "  [OK] TOML Coverage     22/22 loaded",
            width = w
        );
    } else {
        let line = format!("  [!!] TOML Coverage     {} issue(s)", toml_errors);
        println!("| {:width$} |", line, width = w);
    }

    // Cross-References
    if broken_refs == 0 {
        println!(
            "| {:width$} |",
            "  [OK] Cross-References  0 broken",
            width = w
        );
    } else {
        let line = format!("  [!!] Cross-References  {} broken", broken_refs);
        println!("| {:width$} |", line, width = w);
    }

    // Localization
    if missing_en == 0 {
        let total_str = entity_total
            .as_deref()
            .and_then(|m| m.split_whitespace().nth(2))
            .unwrap_or("all");
        let line = format!("  [OK] Localization      {}/{} en", total_str, total_str);
        println!("| {:width$} |", line, width = w);
    } else {
        let line = format!("  [!!] Localization      {} missing en", missing_en);
        println!("| {:width$} |", line, width = w);
    }

    // API Health
    if let Some(issue) = api_issue {
        let api_connected = issue.message.contains("connected");
        let icon = if api_connected { "OK" } else { "--" };
        let detail = if api_connected {
            "connected"
        } else {
            "not running"
        };
        let line = format!("  [{}] API Health        {}", icon, detail);
        println!("| {:width$} |", line, width = w);
    }

    // Data Freshness
    if let Some(issue) = freshness_issue {
        let is_warning = matches!(issue.severity, ValidationSeverity::Warning);
        let icon = if is_warning {
            "!!"
        } else if issue.message.contains("unknown") {
            "--"
        } else {
            "OK"
        };
        let detail: String = if issue.message.contains("unknown") {
            "unknown".into()
        } else if let Some(age) = issue
            .message
            .split_whitespace()
            .find(|w| w.parse::<u64>().is_ok())
        {
            format!("{} min old", age)
        } else {
            "checked".into()
        };
        let line = format!("  [{}] Data Freshness    {}", icon, detail);
        println!("| {:width$} |", line, width = w);
    }

    // Remote Validation
    if !remote_val_issues.is_empty() {
        let has_warnings = remote_val_issues
            .iter()
            .any(|i| matches!(i.severity, ValidationSeverity::Warning));
        let icon = if has_warnings { "!!" } else { "OK" };
        let detail = if has_warnings {
            "issues found"
        } else {
            "sample valid"
        };
        let line = format!("  [{}] Remote Validation {}", icon, detail);
        println!("| {:width$} |", line, width = w);
    }

    println!("{}", bar);

    // Result line
    let result = if errors > 0 {
        format!("RESULT: {} ERROR(S), {} WARNING(S)", errors, warnings)
    } else if warnings > 0 {
        format!("RESULT: {} WARNING(S)", warnings)
    } else {
        "RESULT: ALL CHECKS PASSED".into()
    };
    println!("| {:^width$} |", result, width = w);
    println!("{}", bar);

    // Detail lines for non-info issues
    let detail_issues: Vec<_> = issues
        .iter()
        .filter(|i| !matches!(i.severity, ValidationSeverity::Info))
        .collect();

    if !detail_issues.is_empty() {
        println!();
        println!("Details:");
        for issue in detail_issues {
            let icon = match issue.severity {
                ValidationSeverity::Error => "ERR",
                ValidationSeverity::Warning => "WRN",
                ValidationSeverity::Info => "INF",
            };
            let loc = match (&issue.source_kind, &issue.source_id) {
                (Some(k), Some(id)) => format!(" [{}/{}]", k, id),
                _ => String::new(),
            };
            println!("  [{}]{} {}", icon, loc, issue.message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_empty_registries_has_no_issues() {
        let regs = HelixRegistries::default();
        let mut issues = Vec::new();
        validate_cross_references(&regs, &mut issues);
        validate_localization(&regs, &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn validate_toml_coverage_reports_missing_dir() {
        let mut issues = Vec::new();
        validate_toml_coverage(None, &mut issues);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "helix_no_toml_dir");
    }

    #[test]
    fn api_health_always_emits_info() {
        let regs = HelixRegistries::default();
        let mut issues = Vec::new();
        validate_api_health(&regs, &mut issues);
        // Whether API is online or offline, the issue is always Info severity
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "helix_api_health");
        assert!(matches!(issues[0].severity, ValidationSeverity::Info));
        assert!(
            issues[0].message.contains("not running")
                || issues[0].message.contains("connected")
        );
    }

    #[test]
    fn data_freshness_no_dir_emits_info() {
        let mut issues = Vec::new();
        validate_data_freshness(None, &mut issues);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "helix_data_freshness");
        assert!(matches!(issues[0].severity, ValidationSeverity::Info));
        assert!(issues[0].message.contains("unknown"));
    }

    #[test]
    fn remote_validation_offline_emits_info() {
        let regs = HelixRegistries::default();
        let mut issues = Vec::new();
        validate_remote_sample(&regs, &mut issues);
        // Empty registries + no API = single Info issue
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "helix_remote_validation");
        assert!(matches!(issues[0].severity, ValidationSeverity::Info));
    }

    #[test]
    fn validate_localization_reports_empty_name() {
        let mut regs = HelixRegistries::default();
        regs.mobs.insert(
            "nameless".to_string(),
            helix_data::mob::Mob {
                base: Default::default(),
                mob_type: helix_data::types::MobType::Normal,
                level_min: 0,
                level_max: 0,
                health: 0,
                mana: 0,
                damage_min: 0,
                damage_max: 0,
                armor: 0,
                abilities: Vec::new(),
                loot_table_id: None,
                zone_ids: Vec::new(),
                faction: None,
                respawn_time: 0.0,
                experience_value: 0,
                aggro_range: 0.0,
                leash_range: 0.0,
                attack_speed: 2.0,
                creature_type: None,
                move_speed: 0.0,
                ai_type: None,
                loot_ids: Vec::new(),
            },
        );

        let mut issues = Vec::new();
        validate_localization(&regs, &mut issues);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "helix_missing_en_name");
        assert_eq!(issues[0].source_id.as_deref(), Some("nameless"));
    }
}
