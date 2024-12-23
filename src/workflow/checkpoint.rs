use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Checkpoint {
    pub offset: i64,
    pub partition: u32,
    pub metadata: HashMap<String, String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePartition {
    pub id: u32,
    pub host: String,
    pub offset: i64,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub id: String,
    pub version: i32,
    pub fields: Vec<SchemaField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub name: String,
    pub field_type: FieldType,
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Timestamp,
    Array(Box<FieldType>),
    Struct(Vec<SchemaField>),
} 