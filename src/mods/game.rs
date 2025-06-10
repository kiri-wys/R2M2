//! 1:1 Rimworld's XML's
//!
//! Here are all the models that should map 1:1 to Rimworld's XML files
//! All XML should map to a valid model and viceversa, but unused unnecessary
//! fields may be skipped.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModMetaData {
    pub name: String,
    #[serde(
        default,
        serialize_with = "wrap_strings",
        deserialize_with = "unwrap_strings"
    )]
    pub supported_versions: Vec<String>,
    #[serde(default)]
    pub mod_dependencies_by_version: ModDependencies,
    #[serde(
        default,
        serialize_with = "wrap_strings",
        deserialize_with = "unwrap_strings"
    )]
    pub load_after: Vec<String>,
    pub description: String,
    pub package_id: String,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ModDependencies {
    #[serde(
        default,
        rename = "v1.0",
        serialize_with = "wrap_deps",
        deserialize_with = "unwrap_deps"
    )]
    pub v1_0: Vec<Dependency>,
    #[serde(
        default,
        rename = "v1.1",
        serialize_with = "wrap_deps",
        deserialize_with = "unwrap_deps"
    )]
    pub v1_1: Vec<Dependency>,
    #[serde(
        default,
        rename = "v1.2",
        serialize_with = "wrap_deps",
        deserialize_with = "unwrap_deps"
    )]
    pub v1_2: Vec<Dependency>,
    #[serde(
        default,
        rename = "v1.3",
        serialize_with = "wrap_deps",
        deserialize_with = "unwrap_deps"
    )]
    pub v1_3: Vec<Dependency>,
    #[serde(
        default,
        rename = "v1.4",
        serialize_with = "wrap_deps",
        deserialize_with = "unwrap_deps"
    )]
    pub v1_4: Vec<Dependency>,
    #[serde(
        default,
        rename = "v1.5",
        serialize_with = "wrap_deps",
        deserialize_with = "unwrap_deps"
    )]
    pub v1_5: Vec<Dependency>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub package_id: String,
    pub display_name: String,
}

fn wrap_deps<S>(deps: &[Dependency], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    wrap_list(deps, serializer)
}
fn wrap_strings<S>(deps: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    wrap_list(deps, serializer)
}
fn unwrap_deps<'de, D>(deserializer: D) -> Result<Vec<Dependency>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    unwrap_list(deserializer)
}
fn unwrap_strings<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    unwrap_list(deserializer)
}
#[inline]
fn wrap_list<S, R>(vec: &[R], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    R: Serialize,
{
    #[derive(Serialize)]
    struct List<'a, R> {
        li: &'a [R],
    }

    List { li: vec }.serialize(serializer)
}
#[inline]
fn unwrap_list<'de, D, R>(deserializer: D) -> Result<Vec<R>, D::Error>
where
    D: serde::Deserializer<'de>,
    R: Default + serde::Deserialize<'de>,
{
    #[derive(Deserialize)]
    struct List<R> {
        #[serde(default)]
        li: Vec<R>,
    }
    Ok(List::deserialize(deserializer)?.li)
}
