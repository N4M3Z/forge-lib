use serde_yaml::Value;
use std::path::Path;

pub struct ModelTiers {
    pub fast: String,
    pub strong: String,
}

impl Default for ModelTiers {
    fn default() -> Self {
        Self {
            fast: "sonnet".to_string(),
            strong: "opus".to_string(),
        }
    }
}

pub struct SidecarConfig {
    raw: Value,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self { raw: Value::Null }
    }
}

impl SidecarConfig {
    pub fn load(module_root: &Path) -> Self {
        let defaults = load_yaml_file(&module_root.join("defaults.yaml"))
            .or_else(|| load_yaml_file(&module_root.join("defaults.yml")))
            .unwrap_or(Value::Null);
        let config = load_yaml_file(&module_root.join("config.yaml"))
            .or_else(|| load_yaml_file(&module_root.join("config.yml")))
            .unwrap_or(Value::Null);

        let merged = merge_values(defaults, config);
        Self { raw: merged }
    }

    pub fn provider_tiers(&self, provider: &str) -> ModelTiers {
        let global = self.global_tiers();

        let provider_section = navigate(&self.raw, &["providers", provider, "models"])
            .filter(Value::is_mapping)
            .or_else(|| navigate(&self.raw, &["providers", provider]))
            .or_else(|| navigate(&self.raw, &[provider]));
        if let Some(section) = provider_section {
            ModelTiers {
                fast: yaml_string(&section, "fast").unwrap_or(global.fast),
                strong: yaml_string(&section, "strong").unwrap_or(global.strong),
            }
        } else {
            global
        }
    }

    pub fn is_model_whitelisted(&self, provider: &str, model: &str) -> bool {
        let whitelist = navigate(&self.raw, &["providers", provider, "whitelist"])
            .or_else(|| {
                navigate(&self.raw, &["providers", provider, "models"]).filter(Value::is_sequence)
            })
            .or_else(|| navigate(&self.raw, &[provider, "models"]));
        match whitelist {
            Some(Value::Sequence(ref seq)) if seq.is_empty() => false,
            Some(Value::Sequence(seq)) => seq.iter().any(|v| match v {
                Value::String(s) => s == model,
                _ => false,
            }),
            _ => true,
        }
    }

    pub fn agent_value(&self, agent: &str, key: &str) -> Option<String> {
        let val = navigate(&self.raw, &["agents", agent, key])
            .or_else(|| navigate(&self.raw, &[agent, key]))?;
        normalize_value(val)
    }

    pub fn skill_value(&self, skill_name: &str, key: &str) -> Option<String> {
        let val = navigate(&self.raw, &["skills", skill_name, key])
            .or_else(|| navigate(&self.raw, &[skill_name, key]))?;
        normalize_value(val)
    }

    pub fn provider_skills(&self, provider: &str) -> Vec<String> {
        let Some(section) = navigate(&self.raw, &["skills", provider]) else {
            return Vec::new();
        };
        let Some(mapping) = section.as_mapping() else {
            return Vec::new();
        };
        mapping
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect()
    }

    pub fn provider_skill_value(&self, provider: &str, skill: &str, key: &str) -> Option<String> {
        let val = navigate(&self.raw, &["skills", provider, skill, key])?;
        normalize_value(val)
    }

    pub fn provider_reasoning_effort(&self, provider: &str, model_tier: &str) -> Option<String> {
        let val = navigate(
            &self.raw,
            &["providers", provider, "reasoning_effort", model_tier],
        )
        .or_else(|| navigate(&self.raw, &[provider, "reasoning_effort", model_tier]))?;
        normalize_value(val)
    }

    pub fn global_tiers(&self) -> ModelTiers {
        let shared =
            navigate(&self.raw, &["shared", "models"]).or_else(|| navigate(&self.raw, &["models"]));
        match shared {
            Some(section) => ModelTiers {
                fast: yaml_string(&section, "fast").unwrap_or_else(|| "sonnet".to_string()),
                strong: yaml_string(&section, "strong").unwrap_or_else(|| "opus".to_string()),
            },
            None => ModelTiers::default(),
        }
    }
}

pub fn resolve_model(model: &str, global: &ModelTiers, provider: &ModelTiers) -> String {
    if model == "fast" || model == global.fast {
        provider.fast.clone()
    } else if model == "strong" || model == global.strong {
        provider.strong.clone()
    } else {
        model.to_string()
    }
}

fn load_yaml_file(path: &Path) -> Option<Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&content).ok()
}

fn merge_values(base: Value, overlay: Value) -> Value {
    match (base, overlay) {
        (Value::Mapping(mut base_map), Value::Mapping(overlay_map)) => {
            for (k, v) in overlay_map {
                let merged = if let Some(base_v) = base_map.remove(&k) {
                    merge_values(base_v, v)
                } else {
                    v
                };
                base_map.insert(k, merged);
            }
            Value::Mapping(base_map)
        }
        (_, overlay) if overlay != Value::Null => overlay,
        (base, _) => base,
    }
}

fn normalize_value(val: Value) -> Option<String> {
    match val {
        Value::String(s) => Some(s),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::Null => None,
        _ => Some(serde_yaml::to_string(&val).ok()?.trim().to_string()),
    }
}

fn navigate(value: &Value, keys: &[&str]) -> Option<Value> {
    let mut current = value;
    for key in keys {
        current = current
            .as_mapping()?
            .get(Value::String((*key).to_string()))?;
    }
    Some(current.clone())
}

fn yaml_string(value: &Value, key: &str) -> Option<String> {
    match value.as_mapping()?.get(Value::String(key.to_string()))? {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
