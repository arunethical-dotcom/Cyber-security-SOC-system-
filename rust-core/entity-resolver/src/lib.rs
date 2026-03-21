use ipnetwork::IpNetwork;
use shared::{Entity, EntityKind};
use std::collections::HashMap;
use std::net::IpAddr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("Invalid IP address: {0}")]
    InvalidIp(String),
    #[error("Config error: {0}")]
    ConfigError(String),
}

pub struct EntityResolver {
    aliases: HashMap<String, String>,
    cidr_groups: Vec<(IpNetwork, String)>,
}

impl EntityResolver {
    pub fn new(
        aliases: HashMap<String, String>,
        cidr_groups: Vec<(String, String)>,
    ) -> Result<Self, ResolverError> {
        let mut cidr_vec = Vec::new();
        for (cidr, group) in cidr_groups {
            let network: IpNetwork = cidr
                .parse()
                .map_err(|e| ResolverError::ConfigError(format!("Invalid CIDR {}: {}", cidr, e)))?;
            cidr_vec.push((network, group));
        }
        cidr_vec.sort_by(|a, b| b.0.prefix().cmp(&a.0.prefix()));
        Ok(Self {
            aliases,
            cidr_groups: cidr_vec,
        })
    }

    pub fn from_config(config: &Config) -> Result<Self, ResolverError> {
        let aliases: HashMap<String, String> = config
            .aliases
            .iter()
            .map(|(k, v)| (k.to_uppercase(), v.clone()))
            .collect();

        let cidr_groups: Vec<(String, String)> = config
            .cidr_groups
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Self::new(aliases, cidr_groups)
    }

    pub fn normalize_username(&self, username: &str) -> String {
        let upper = username.to_uppercase();
        if let Some(resolved) = self.aliases.get(&upper) {
            resolved.clone()
        } else {
            format!("user:{}", username.to_lowercase())
        }
    }

    pub fn normalize_ip(&self, ip_str: &str) -> Option<Entity> {
        let ip: IpAddr = ip_str.parse().ok()?;

        for (network, group) in &self.cidr_groups {
            if network.contains(ip) {
                return Some(Entity::new(EntityKind::IP, group.clone()));
            }
        }

        Some(Entity::new(EntityKind::IP, format!("ip:{}", ip_str)))
    }

    pub fn resolve(&self, kind: &EntityKind, key: &str) -> Entity {
        match kind {
            EntityKind::User => Entity::new(EntityKind::User, self.normalize_username(key)),
            EntityKind::IP => self
                .normalize_ip(key)
                .unwrap_or_else(|| Entity::new(EntityKind::IP, format!("ip:{}", key))),
            EntityKind::Device => Entity::new(EntityKind::Device, format!("device:{}", key)),
            EntityKind::Process => Entity::new(EntityKind::Process, format!("process:{}", key)),
            EntityKind::File => Entity::new(EntityKind::File, format!("file:{}", key)),
        }
    }
}

pub struct Config {
    pub aliases: HashMap<String, String>,
    pub cidr_groups: HashMap<String, String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, ResolverError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ResolverError::ConfigError(format!("Failed to read config: {}", e)))?;

        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, ResolverError> {
        let mut aliases = HashMap::new();
        let mut cidr_groups = HashMap::new();

        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].to_string();
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                match current_section.as_str() {
                    "aliases" => {
                        aliases.insert(key.to_string(), value.to_string());
                    }
                    "cidr_groups" => {
                        cidr_groups.insert(key.to_string(), value.to_string());
                    }
                    _ => {}
                }
            }
        }

        Ok(Self {
            aliases,
            cidr_groups,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_normalization() {
        let config = Config {
            aliases: {
                let mut m = HashMap::new();
                m.insert("ADMIN".to_string(), "user:admin".to_string());
                m.insert("DOMAIN\\ADMIN".to_string(), "user:admin".to_string());
                m
            },
            cidr_groups: HashMap::new(),
        };

        let resolver = EntityResolver::from_config(&config).unwrap();

        assert_eq!(resolver.normalize_username("admin"), "user:admin");
        assert_eq!(resolver.normalize_username("ADMIN"), "user:admin");
        assert_eq!(resolver.normalize_username("Domain\\admin"), "user:admin");
        assert_eq!(resolver.normalize_username("john"), "user:john");
    }

    #[test]
    fn test_ip_normalization() {
        let config = Config {
            aliases: HashMap::new(),
            cidr_groups: {
                let mut m = HashMap::new();
                m.insert("10.0.0.0/8".to_string(), "subnet:internal-a".to_string());
                m
            },
        };

        let resolver = EntityResolver::from_config(&config).unwrap();

        assert_eq!(
            resolver.normalize_ip("10.1.2.3").unwrap().key,
            "subnet:internal-a"
        );
        assert_eq!(
            resolver.normalize_ip("192.168.1.1").unwrap().key,
            "ip:192.168.1.1"
        );
    }
}
