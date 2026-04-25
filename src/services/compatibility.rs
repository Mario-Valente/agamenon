use crate::error::CompatibilityError;
use crate::models::CompatibilityLevel;
use apache_avro::Schema;
use std::collections::HashMap;

pub struct CompatibilityChecker;

impl CompatibilityChecker {
    /// Check if new_schema is compatible with old_schema according to the level
    pub fn check(
        new_schema_str: &str,
        old_schema_str: &str,
        level: CompatibilityLevel,
    ) -> Result<bool, CompatibilityError> {
        if level == CompatibilityLevel::None {
            return Ok(true);
        }

        let new_schema = Schema::parse_str(new_schema_str)
            .map_err(|e| CompatibilityError::InvalidSchema(e.to_string()))?;
        let old_schema = Schema::parse_str(old_schema_str)
            .map_err(|e| CompatibilityError::InvalidSchema(e.to_string()))?;

        match level {
            CompatibilityLevel::None => Ok(true),
            CompatibilityLevel::Backward => Self::is_backward_compatible(&new_schema, &old_schema),
            CompatibilityLevel::Forward => Self::is_forward_compatible(&new_schema, &old_schema),
            CompatibilityLevel::Full => {
                let backward = Self::is_backward_compatible(&new_schema, &old_schema)?;
                let forward = Self::is_forward_compatible(&new_schema, &old_schema)?;
                Ok(backward && forward)
            }
        }
    }

    /// BACKWARD: new schema can read old data
    /// - New fields must have default values
    /// - Removed fields are OK
    /// - Field type changes: must be compatible (int -> long OK, int -> string NOT OK)
    fn is_backward_compatible(
        new_schema: &Schema,
        old_schema: &Schema,
    ) -> Result<bool, CompatibilityError> {
        match (new_schema, old_schema) {
            (Schema::Record(new_record), Schema::Record(old_record)) => {
                let new_map: HashMap<_, _> = new_record.fields.iter().map(|f| (&f.name, f)).collect();
                let old_map: HashMap<_, _> = old_record.fields.iter().map(|f| (&f.name, f)).collect();

                // All old fields must exist in new schema or have defaults
                for (old_name, old_field) in &old_map {
                    if let Some(new_field) = new_map.get(*old_name) {
                        // Field exists: check type compatibility
                        if !Self::is_type_compatible(&new_field.schema, &old_field.schema) {
                            return Err(CompatibilityError::Incompatible(
                                format!("Field '{}' type changed incompatibly", old_name),
                            ));
                        }
                    }
                    // If field doesn't exist in new schema, it's OK (backward compatible)
                }

                // New fields must have defaults
                for (new_name, new_field) in &new_map {
                    if !old_map.contains_key(*new_name) && new_field.default.is_none() {
                        return Err(CompatibilityError::Incompatible(
                            format!("New field '{}' missing default value", new_name),
                        ));
                    }
                }

                Ok(true)
            }
            // For simple types, check if types are compatible
            _ => Ok(Self::is_type_compatible(new_schema, old_schema)),
        }
    }

    /// FORWARD: old schema can read new data
    /// - New fields are ignored by old readers (OK)
    /// - Removed fields: old readers expect them (NOT OK unless they have defaults)
    /// - Field type changes: old reader expects original type
    fn is_forward_compatible(
        new_schema: &Schema,
        old_schema: &Schema,
    ) -> Result<bool, CompatibilityError> {
        match (new_schema, old_schema) {
            (Schema::Record(new_record), Schema::Record(old_record)) => {
                let new_map: HashMap<_, _> = new_record.fields.iter().map(|f| (&f.name, f)).collect();
                let old_map: HashMap<_, _> = old_record.fields.iter().map(|f| (&f.name, f)).collect();

                // All old fields must still exist or have defaults in old schema
                for (old_name, old_field) in &old_map {
                    if let Some(new_field) = new_map.get(*old_name) {
                        // Field exists: check type compatibility (from old perspective)
                        if !Self::is_type_compatible(&old_field.schema, &new_field.schema) {
                            return Err(CompatibilityError::Incompatible(
                                format!("Field '{}' type changed incompatibly", old_name),
                            ));
                        }
                    } else if old_field.default.is_none() {
                        // Field removed and old schema has no default
                        return Err(CompatibilityError::Incompatible(
                            format!("Required field '{}' was removed", old_name),
                        ));
                    }
                }

                Ok(true)
            }
            _ => Ok(Self::is_type_compatible(old_schema, new_schema)),
        }
    }

    /// Check if two types are compatible
    fn is_type_compatible(new: &Schema, old: &Schema) -> bool {
        use apache_avro::Schema::*;
        match (new, old) {
            // Exact same type
            (Null, Null) => true,
            (Boolean, Boolean) => true,
            (Int, Int) => true,
            (Long, Long) => true,
            (Float, Float) => true,
            (Double, Double) => true,
            (String, String) => true,
            (Bytes, Bytes) => true,

            // Numeric promotions (new can be larger type)
            (Long, Int) => true,  // new can read old int as long
            (Float, Int) => true,
            (Float, Long) => true,
            (Double, Int) => true,
            (Double, Long) => true,
            (Double, Float) => true,

            // Record compatibility (both must be records)
            (Record(_), Record(_)) => true, // Recursive check in function above

            // Union - if new is a union containing old type, compatible
            (Union(new_union), old_type) => {
                new_union.variants().iter().any(|s| Self::is_type_compatible(s, old_type))
            }

            (new_type, Union(old_union)) => {
                old_union.variants().iter().any(|s| Self::is_type_compatible(new_type, s))
            }

            _ => false,
        }
    }
}
