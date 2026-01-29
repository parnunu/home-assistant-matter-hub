use serde_json::Value;

use crate::models::{BridgeFilter, EntityFilter, FilterKind};

#[derive(Debug, Clone)]
pub struct EntityDescriptor {
    pub entity_id: String,
    pub domain: String,
    pub platform: Option<String>,
    pub entity_category: Option<String>,
    pub area: Option<String>,
    pub labels: Vec<String>,
    pub device_id: Option<String>,
    pub attributes: Value,
}

pub fn matches_filter(filter: &BridgeFilter, entity: &EntityDescriptor) -> bool {
    let include_ok = if filter.include.is_empty() {
        true
    } else {
        filter.include.iter().any(|rule| matches_rule(rule, entity))
    };

    let exclude_hit = filter.exclude.iter().any(|rule| matches_rule(rule, entity));

    include_ok && !exclude_hit
}

fn matches_rule(rule: &EntityFilter, entity: &EntityDescriptor) -> bool {
    match rule.kind {
        FilterKind::Pattern => wildcard_match(&rule.value, &entity.entity_id),
        FilterKind::Domain => entity.domain == rule.value,
        FilterKind::Platform => entity
            .platform
            .as_ref()
            .map(|p| p == &rule.value)
            .unwrap_or(false),
        FilterKind::EntityCategory => entity
            .entity_category
            .as_ref()
            .map(|c| c == &rule.value)
            .unwrap_or(false),
        FilterKind::Area => entity.area.as_ref().map(|a| a == &rule.value).unwrap_or(false),
        FilterKind::Label => entity.labels.iter().any(|l| l == &rule.value),
        FilterKind::EntityId => entity.entity_id == rule.value,
        FilterKind::DeviceId => entity.device_id.as_ref().map(|d| d == &rule.value).unwrap_or(false),
    }
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let mut dp = vec![false; t.len() + 1];
    dp[0] = true;

    for ch in p {
        let mut next = vec![false; t.len() + 1];
        if ch == '*' {
            let mut any = false;
            for j in 0..=t.len() {
                any = any || dp[j];
                next[j] = any;
            }
        } else if ch == '?' {
            for j in 0..t.len() {
                if dp[j] {
                    next[j + 1] = true;
                }
            }
        } else {
            for j in 0..t.len() {
                if dp[j] && t[j] == ch {
                    next[j + 1] = true;
                }
            }
        }
        dp = next;
    }

    dp[t.len()]
}
