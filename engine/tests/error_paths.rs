//! Error path tests — verify the data layer handles invalid input gracefully.

use dj_engine::data::{load_project, load_scene, load_story_graph, DataError};
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

#[test]
fn load_project_missing_file_returns_not_found() {
    let result = load_project(Path::new("/nonexistent/project.json"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, DataError::NotFound(_)),
        "expected NotFound, got: {err}"
    );
}

#[test]
fn load_project_corrupted_json_returns_parse_error() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "{{not valid json!!!}}").unwrap();
    let result = load_project(f.path());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, DataError::Json(_)),
        "expected Json parse error, got: {err}"
    );
}

#[test]
fn load_project_empty_file_returns_parse_error() {
    let f = NamedTempFile::new().unwrap();
    let result = load_project(f.path());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, DataError::Json(_)),
        "expected Json parse error, got: {err}"
    );
}

#[test]
fn load_scene_missing_file_returns_not_found() {
    let result = load_scene(Path::new("/nonexistent/scene.json"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, DataError::NotFound(_)),
        "expected NotFound, got: {err}"
    );
}

#[test]
fn load_scene_corrupted_json_returns_parse_error() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "[broken").unwrap();
    let result = load_scene(f.path());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, DataError::Json(_)),
        "expected Json parse error, got: {err}"
    );
}

#[test]
fn load_story_graph_missing_file_returns_not_found() {
    let result = load_story_graph(Path::new("/nonexistent/story.json"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, DataError::NotFound(_)),
        "expected NotFound, got: {err}"
    );
}

#[test]
fn load_story_graph_corrupted_json_returns_parse_error() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "null").unwrap();
    let result = load_story_graph(f.path());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, DataError::Json(_)),
        "expected Json parse error, got: {err}"
    );
}

#[test]
fn load_project_wrong_schema_returns_parse_error() {
    let mut f = NamedTempFile::new().unwrap();
    // Valid JSON but wrong schema — an array instead of an object
    write!(f, "[1, 2, 3]").unwrap();
    let result = load_project(f.path());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, DataError::Json(_)),
        "expected Json parse error, got: {err}"
    );
}
