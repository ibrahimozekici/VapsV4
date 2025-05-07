use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use crate::storage::schema_postgres::zone;
use crate::storage::schema_postgres::zone::dsl;

use super::{get_async_db_conn};
use tracing::info;
use super::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Insertable, Queryable)]
#[diesel(table_name = zone)]
pub struct Zone {
    pub zone_id: i32,
    pub zone_name: Option<String>,
    pub zone_order: Option<i64>,     // moved up
    pub content_type: Option<i64>,   // moved up
    pub tanent_id: Option<Uuid>,     // moved down
    pub devices: Vec<Option<String>>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = zone)]
pub struct NewZone {
    pub zone_name: Option<String>,
    pub zone_order: Option<i64>,
    pub content_type: Option<i64>,
    pub tanent_id: Option<Uuid>,
    devices: Vec<Option<String>>,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = zone)]
pub struct UpdateZone {
    pub zone_name: Option<String>,
    pub zone_order: Option<i64>,
    pub content_type: Option<i64>,
    // pub tanent_id: Option<Uuid>,
    // pub devices: Option<Vec<Option<String>>>,
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
            zone_order: Some(0),
            content_type: Some(0),
            devices: vec![],
        }
    }
}
impl Default for NewZone {
    fn default() -> Self {
        NewZone {
            zone_name: Some("".to_string()),
            tanent_id: Some(Uuid::new_v4()),
            zone_order: Some(0),
            content_type: Some(0),
            devices: vec![],
        }
    }
}
pub async fn create(a: Zone) -> Result<Zone, Error> {
    a.validate()?;

    let new = NewZone {
        zone_name: a.zone_name,
        zone_order: a.zone_order,
        content_type: a.content_type,
        tanent_id: a.tanent_id,
        devices: a.devices
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

pub async fn update(zone_id: i32, update_data: UpdateZone) -> Result<Zone, Error> {
    let updated: Zone = diesel::update(dsl::zone.filter(dsl::zone_id.eq(zone_id)))
        .set(update_data)
        .get_result(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, format!("update zone {zone_id}")))?;

    info!(id = %updated.zone_id, "Zone updated");

    Ok(updated)
}

pub async fn delete(zone_id: i32) -> Result<usize, Error> {
    let deleted_rows = diesel::delete(dsl::zone.filter(dsl::zone_id.eq(zone_id)))
        .execute(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, format!("delete zone {zone_id}")))?;

    info!(id = %zone_id, rows = %deleted_rows, "Zone deleted");

    Ok(deleted_rows)
}

pub async fn list_zones(
    organization_id: String,
) -> Result<Vec<Zone>, Error> {
    let conn = &mut get_async_db_conn().await?;

    let org_uuid = Uuid::parse_str(&organization_id)
        .map_err(|_| Error::Validation("Invalid organization_id".into()))?;

    let zones = dsl::zone
        .filter(dsl::tanent_id.eq(org_uuid))
        .load::<Zone>(conn)
        .await
        .map_err(|e| Error::from_diesel(e, "list zones".into()))?;

    Ok(zones)
}