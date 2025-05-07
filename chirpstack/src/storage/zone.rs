use diesel::prelude::*;
use diesel::pg::Pg;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use crate::storage::schema_postgres::zone;
use super::{error, fields, get_async_db_conn};
use tracing::info;
use super::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Insertable, Queryable)]
#[diesel(table_name = zone)]
pub struct Zone {
    pub zone_id: i32,
    pub zone_name: Option<String>,
    pub tanent_id: Option<Uuid>,
    pub devices: Vec<String>,
    pub zone_order: Option<i64>,
    pub content_type: Option<i64>,
}



#[derive(Insertable, Debug)]
#[diesel(table_name = zone)]
pub struct NewZone<'a> {
    pub zone_name: Option<&'a str>,
    pub devices: Vec<String>,
    pub zone_order: Option<i64>,
    pub content_type: Option<i64>,
    pub tanent_id: Option<Uuid>,
}

impl Zone {
    fn validate(&self) -> Result<(), Error> {
        if let Some(name) = &self.zone_name {
            if name.trim().is_empty() {
                return Err(Error::Validation("Zone name cannot be empty".into()));
            }
        }
        Ok(())
    }
}

impl Default for Zone {
    fn default() -> Self {
        Zone {
            zone_id: 0, // or some sentinel like -1, depending on your logic
            zone_name: Some("".to_string()),
            tanent_id: Some(Uuid::new_v4()),
            devices: vec![],
            zone_order: Some(0),
            content_type: Some(0),
        }
    }
}
pub async fn create(a: Zone) -> Result<Zone, Error> {
    a.validate()?;

    let new = NewZone {
        zone_name: a.zone_name.as_deref(),
        devices: a.devices,
        zone_order: a.zone_order,
        content_type: a.content_type,
        tanent_id: a.tanent_id,
    };

    let inserted: Zone = diesel::insert_into(zone::table)
        .values(&new)
        .get_result(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, "insert zone".to_string()))?;

    info!(id = %inserted.zone_id, "Zone created");

    Ok(inserted)
}




pub async fn get(id: &i32) -> Result<Zone, Error> {
    let a = zone::dsl::zone
        .find(id)
        .first(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, id.to_string()))?;
    Ok(a)
}
