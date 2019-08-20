use {
    chrono::{DateTime, Utc},
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Cache {
    #[serde(with = "domains_expire_serializer")]
    pub domains_expire: HashMap<String, DateTime<Utc>>,
}

mod domains_expire_serializer {
    use chrono::{DateTime, Utc};
    use serde::{ser::SerializeMap, Deserialize, Deserializer};
    use std::collections::HashMap;

    type ItemType = HashMap<String, DateTime<Utc>>;

    pub fn serialize<S>(item: &ItemType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut map = serializer.serialize_map(Some(item.len()))?;
        for (k, v) in item.iter() {
            map.serialize_entry(k, &v.to_rfc3339())?;
        }
        map.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ItemType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: HashMap<String, String> = HashMap::deserialize(deserializer)?;
        let mut res = ItemType::with_capacity(map.len());
        for (k, v) in map {
            let time_fixed = DateTime::parse_from_rfc3339(&v).map_err(serde::de::Error::custom)?;
            res.insert(k, time_fixed.into());
        }
        Ok(res)
    }
}

impl Cache {
    pub(crate) fn new() -> Self {
        Cache {
            domains_expire: HashMap::new(),
        }
    }

    pub(crate) fn clean(&mut self, now: &chrono::DateTime<Utc>, no_cache_days_before_expire: i64) {
        let mut for_delete = Vec::new();
        self.domains_expire.iter().for_each(|(k, expire)| {
            let days = (*expire - *now).num_days();
            if days <= no_cache_days_before_expire {
                for_delete.push(k.clone());
            }
        });
        for domain in &for_delete {
            self.domains_expire.remove(domain);
        }
    }
}
