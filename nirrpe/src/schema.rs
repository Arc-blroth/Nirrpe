//! Core data structures that form the Nirrpe schema system.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION_1: u32 = 1;

/// Meta information about a schema.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct SchemaMeta<'a> {
    pub version: u32,
    pub name: &'a str,
    pub parent: Option<&'a str>,
}

/// The data type of an attribute of the trait a schema represents.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttributeType {
    /// The [`bool`](core::bool) type.
    Bool,
    /// The [`u32`](core::u32) type.
    Int,
    /// The [`u64`](core::u64) type.
    Long,
    /// The [`f64`](core::f64) type.
    Double,
    /// The [`String`](std::string::String) type.
    String,
    /// Discord's 64-bit snowflake type. This is
    /// automatically coerced to the right
    /// [`serenity::model::id`] type for scripts.
    Snowflake,
}

/// How an attribute is displayed in the Discord UI.
#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AttributeDisplay<'a> {
    #[default]
    Plain,
    Bar(#[serde(borrow)] AttributeDisplayBar<'a>),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AttributeDisplayBar<'a> {
    pub foreground: &'a str,
    pub background: &'a str,
    #[serde(default = "AttributeDisplayBar::default_scale")]
    pub scale: f64,
}

impl<'a> AttributeDisplayBar<'a> {
    pub fn default_scale() -> f64 {
        1.0
    }
}

/// A single attribute of the trait a schema represents.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Attribute<'a> {
    pub name: &'a str,
    #[serde(rename = "type")]
    pub ty: AttributeType,
    #[serde(borrow, default)]
    pub display: AttributeDisplay<'a>,
}

/// A schema that describes the layout of a Nirrpe trait.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Schema<'a> {
    #[serde(borrow, rename = "schema")]
    pub meta: SchemaMeta<'a>,
    #[serde(borrow)]
    pub attributes: HashMap<&'a str, Attribute<'a>>,
}
