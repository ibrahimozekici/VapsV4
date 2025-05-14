use chirpstack_api::api::ListZoneResponse;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use crate::storage::schema_postgres::zone;
use crate::storage::schema_postgres::zone::dsl;
use diesel::sql_query;
use tonic::Status;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use diesel::sql_types::Nullable;
use diesel::sql_types::Uuid as SqlUuid;
use super::{get_async_db_conn};
use tracing::info;
use super::error::Error;
// use chirpstack_api::api::ListZoneResponse;
use serde_json;

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

#[derive(Debug, QueryableByName)]
pub struct ZoneListRow {
    #[sql_type = "diesel::sql_types::Text"]
    pub zones: String, // this is the raw JSON string returned from SQL
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListZoneResponseSerde {
    pub zones: Vec<GetZonesItemSerde>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetZonesItemSerde {
    #[serde(rename = "zoneId")]
    pub zone_id: i64,

    #[serde(rename = "zoneName")]
    pub zone_name: String,

    #[serde(rename = "orgID")]
    pub org_id: String,

    #[serde(rename = "order")]
    pub order: i64,

    #[serde(rename = "devices")]
    pub devices: Vec<ZoneDeviceSerde>,

    #[serde(rename = "contentType")]
    pub content_type: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ZoneDeviceSerde {
    pub device_dev_eui: String,
    pub device_name: String,
    pub device_description: String,
    pub device_last_seen_at: String,
    pub data: Vec<ZoneDataSerde>,
    pub device_profile_name: Vec<ZoneDeviceProfileSerde>,
    pub device_type: Option<i64>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[serde(rename = "temperatureCalibration")]
    pub temperature_calibration: f64,
    #[serde(rename = "humadityCalibration")]
    pub humadity_calibration: f64,
    pub variables: HashMap<String, String>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoneDeviceProfileSerde {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoneDataSerde {
    pub id: i64,
    pub dev_eui: String,
    pub device_type_id: i64,
    pub org_id: String,
    pub air_temperature: f32,
    pub air_humidity: f32,
    pub sol_temperature: f32,
    pub sol_water: f32,
    pub sol_conduct_soil: f32,
    pub submission_date: String,
    pub water_leak_status: i64,
    pub water_leak_times: i64,
    pub last_water_leak_duration: i64,
    pub door_open_status: i64,
    pub door_open_times: i64,
    pub last_door_open_duration: i64,
    pub batv: f32,
    pub ro1_status: i64,
    pub ro2_status: i64,
    pub ph_soil: f32,
    pub co2_ppm: f32,
    pub tvoc_ppm: f32,
    pub sensecap_light: f32,
    pub barometric_pressure: f32,
    pub power: f32,
    pub current: f32,
    pub voltage: f32,
    pub factor: f32,
    #[serde(rename = "powerSum")]
    pub power_sum: f32,
    pub status: i64,
    pub power_consumption: i64,
    pub switch1: i64,
    pub switch2: i64,
    pub switch3: i64,
    pub switch4: i64,
    pub switch5: i64,
    pub switch6: i64,
    pub switch7: i64,
    pub switch8: i64,
    pub adc_1: String,
    pub adc_2: String,
    pub adv_1: String,
    pub gpio_in_1: String,
    pub gpio_in_2: String,
    pub gpio_in_3: String,
    pub gpio_in_4: String,
    pub gpio_out_1: String,
    pub gpio_out_2: String,
    pub distance: i64,
    pub position: String,
    pub temperature1: f32,
    pub temperature2: f32,
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

pub async fn list(
    user_id: Option<Uuid>,
    tanent_id: Option<Uuid>,
) -> Result<ListZoneResponseSerde, Status> {
    let mut query = r#"
        WITH device_data_2025 AS (
            SELECT 
                dev.dev_eui,
                json_build_object(
                    'device_dev_eui', dev.dev_eui,
                    'device_name', dev.name,
                    'device_description', dev.description,
                    'tags', dev.tags,
                    'variables', dev.variables,
                    'temperature_calibration', dev.temperature_calibration,
                    'humadity_calibration', dev.humadity_calibration,
                    'device_type', dl.device_type_id,
                    'latitude', dev.latitude,
                    'longitude', dev.longitude,
                    'data', COALESCE(array_agg(dl) FILTER (WHERE dl.dev_eui IS NOT NULL), ARRAY[]::device_data_latest[])
                ) AS device_json
            FROM public.device AS dev
            LEFT JOIN device_data_latest dl ON dev.dev_eui::text = '\\x' || dl.dev_eui
            GROUP BY dev.dev_eui, dev.name, dev.tags, dev.variables, dev.temperature_calibration, dev.humadity_calibration, dl.device_type_id
        ),
        zone_data AS (
            SELECT 
                z.zone_id,
                z.zone_name,
                z.tanent_id,
                z.zone_order,
                z.content_type,
                json_build_object(
                    'zone_id', z.zone_id,
                    'zone_name', z.zone_name,
                    'org_id', z.tanent_id,
                    'order', z.zone_order,
                    'contentType', z.content_type,
                    'devices', COALESCE(array_agg(dd.device_json) FILTER (WHERE dd.device_json IS NOT NULL), ARRAY[]::json[])
                ) AS list
            FROM public.zone AS z
            LEFT JOIN public.device AS dev ON dev.dev_eui::text = ANY(z.devices)
            LEFT JOIN device_data_2025 dd ON dev.dev_eui = dd.dev_eui
            GROUP BY z.zone_id, z.zone_name, z.tanent_id, z.zone_order, z.content_type
        )
        SELECT json_build_object(
            'zones', COALESCE(array_agg(zl.list), ARRAY[]::json[])
        ) AS zones
        FROM public.user AS a
        INNER JOIN zone_data zl ON zl.zone_id = ANY(a.zone_id_list)
        WHERE a.id = $1
    "#.to_string();

    if tanent_id.is_some() {
        query.push_str(" AND zl.tanent_id = $2 GROUP BY a.id");
    } else {
        query.push_str(" GROUP BY a.id");
    }

    // DB connection
    let conn = &mut get_async_db_conn().await.map_err(|e| {
        Status::internal(format!("DB connection failed: {e}"))
    })?;

    let row: ZoneListRow = if let Some(tanent_id) = tanent_id {
        sql_query(&query)
            .bind::<Nullable<SqlUuid>, _>(user_id)
            .bind::<diesel::sql_types::Uuid, _>(tanent_id)
            .get_result(conn)
            .await
            .map_err(|e| Status::internal(format!("Query failed: {e}")))?
    } else {
        sql_query(&query)
            .bind::<Nullable<SqlUuid>, _>(user_id)
            .get_result(conn)
            .await
            .map_err(|e| Status::internal(format!("Query failed: {e}")))?
    };

    let parsed: ListZoneResponseSerde = serde_json::from_str(&row.zones)
        .map_err(|e| Status::internal(format!("Failed to deserialize: {e}")))?;


    // // Convert to protobuf
    // let zone_list: pb::ListZoneResponse = serde_json::from_value(row.zones)
    //     .map_err(|e| Status::internal(format!("Deserialization failed: {e}")))?;

    Ok(parsed)
}
