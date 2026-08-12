use serde::Serialize;
use serde_json::Value;

use crate::config;

pub enum Source<'a> {
    Path(&'a str),
    WorkshopId(&'a str),
}

pub fn resolve(source: Source) -> Result<String, String> {
    match source {
        Source::Path(path) => Ok(path.to_string()),
        Source::WorkshopId(id) => resolve_workshop_id(id),
    }
}

fn resolve_workshop_id(id: &str) -> Result<String, String> {
    let project_file = format!(
        "{}/{id}/project.json",
        config::workshop_content_dir_result()?
    );
    if !std::path::Path::new(&project_file).is_file() {
        return Err(format!(
            "no workshop wallpaper found for id {id}: expected {project_file}"
        ));
    }
    Ok(project_file)
}

fn read_project(project_file: &str) -> Result<Value, String> {
    let data = std::fs::read_to_string(project_file)
        .map_err(|e| format!("failed to read {project_file}: {e}"))?;
    serde_json::from_str(&data).map_err(|e| format!("failed to parse {project_file}: {e}"))
}

#[derive(Serialize)]
pub struct WallpaperInfo {
    pub id: String,
    pub title: String,
    pub kind: String,
}

pub fn list_wallpapers() -> Result<Vec<WallpaperInfo>, String> {
    let dir = config::workshop_content_dir_result()?;
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("failed to read {dir}: {e}"))?;

    let mut wallpapers = Vec::new();
    for entry in entries.flatten() {
        let id = entry.file_name().to_string_lossy().into_owned();
        let project_file = entry.path().join("project.json");
        let Ok(data) = std::fs::read_to_string(&project_file) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&data) else {
            continue;
        };
        let title = value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("(untitled)")
            .to_string();
        let kind = value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        wallpapers.push(WallpaperInfo { id, title, kind });
    }
    wallpapers.sort_by_key(|a| a.title.to_lowercase());
    Ok(wallpapers)
}

#[derive(Serialize)]
pub struct PropertyInfo {
    pub key: String,
    pub kind: String,
    pub text: String,
    pub value: Value,
}

pub fn properties(id: &str) -> Result<(String, Vec<PropertyInfo>), String> {
    let project_file = resolve_workshop_id(id)?;
    let project = read_project(&project_file)?;

    let title = project
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("(untitled)")
        .to_string();

    let mut properties: Vec<PropertyInfo> = project
        .get("general")
        .and_then(|general| general.get("properties"))
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(key, prop)| PropertyInfo {
            key: key.clone(),
            kind: prop
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            text: prop
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            value: prop.get("value").cloned().unwrap_or(Value::Null),
        })
        .collect();
    properties.sort_by(|a, b| a.key.cmp(&b.key));

    Ok((title, properties))
}
