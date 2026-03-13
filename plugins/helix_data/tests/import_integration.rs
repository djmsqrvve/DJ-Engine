use bevy::prelude::*;
use dj_engine::data::{load_custom_documents_from_project, load_project, CustomDocumentRegistry};
use dj_engine::project_mount::MountedProject;
use dj_engine_helix::{
    import_helix_project, HelixDataPlugin, HelixDocumentIndex, HELIX_ABILITY_KIND,
    HELIX_IMPORT_PREVIEW_ID, HELIX_ITEM_KIND, HELIX_MOB_KIND,
};
use serde_json::json;
use std::fs;

#[test]
fn imports_helix_fixture_and_builds_runtime_index() {
    let temp_dir = tempfile::tempdir().unwrap();
    let helix_dist = temp_dir.path().join("dist");
    fs::create_dir_all(helix_dist.join("ability/abilitys")).unwrap();
    fs::create_dir_all(helix_dist.join("weapon/items")).unwrap();
    fs::create_dir_all(helix_dist.join("GameData/demons")).unwrap();

    fs::write(
        helix_dist.join("ability/abilitys/fireball.json"),
        serde_json::to_string_pretty(&json!({
            "id": "fireball",
            "name": { "en": "Fireball" },
            "type": "spell",
            "description": { "en": "A fiery bolt." }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        helix_dist.join("weapon/items/common_dagger.json"),
        serde_json::to_string_pretty(&json!({
            "id": "common_dagger",
            "name": { "en": "Common Dagger" },
            "category": "weapon"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        helix_dist.join("GameData/demons/felguard.json"),
        serde_json::to_string_pretty(&json!({
            "id": "felguard",
            "name": { "en": "Felguard" },
            "type": "demon",
            "level": 10,
            "stats": { "health": 100, "damage": 12 },
            "model": { "rig_profile": "Felguard" },
            "abilities": ["fireball", "missing_spell"],
            "loot": [
                { "item": "common_dagger", "chance": 1.0 },
                { "item": "missing_item", "chance": 0.5 }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let project_path = temp_dir.path().join("mounted_project");
    let summary = import_helix_project(&helix_dist, &project_path).unwrap();
    assert_eq!(summary.abilities, 1);
    assert_eq!(summary.items, 1);
    assert_eq!(summary.mobs, 1);

    let manifest_path = project_path.join("project.json");
    let project = load_project(&manifest_path).unwrap();
    let mounted = MountedProject {
        root_path: Some(project_path.clone()),
        manifest_path: Some(manifest_path),
        project: Some(project),
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(HelixDataPlugin);

    let registry = app.world().resource::<CustomDocumentRegistry>().clone();
    let loaded_documents = load_custom_documents_from_project(&mounted, &registry);
    assert!(!loaded_documents.has_blocking_errors());
    assert!(loaded_documents
        .get(HELIX_ABILITY_KIND, "fireball")
        .is_some());
    assert!(loaded_documents
        .get(HELIX_ITEM_KIND, "common_dagger")
        .is_some());
    assert!(loaded_documents.get(HELIX_MOB_KIND, "felguard").is_some());
    assert!(loaded_documents
        .get("preview_profiles", HELIX_IMPORT_PREVIEW_ID)
        .is_some());

    let mob = loaded_documents.get(HELIX_MOB_KIND, "felguard").unwrap();
    let references = &mob.document.as_ref().unwrap().references;
    assert_eq!(references.len(), 2);
    assert_eq!(references[0].field_path, "payload.abilities[0]");
    assert_eq!(references[1].field_path, "payload.loot[0].item");

    app.insert_resource(loaded_documents);
    app.update();

    let index = app.world().resource::<HelixDocumentIndex>();
    assert!(index.ability("fireball").is_some());
    assert!(index.item("common_dagger").is_some());
    assert!(index.mob("felguard").is_some());
}
