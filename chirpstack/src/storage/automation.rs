use super::{error::Error, get_async_db_conn};
use crate::storage::schema_postgres::automation_rules;
use base64::engine::general_purpose::STANDARD as base64_engine;
use base64::Engine;
use base64::{engine::general_purpose as base64_engine, Engine as _};
use chrono::NaiveDateTime;
use chrono::Utc;
use diesel::deserialize::QueryableByName;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Nullable;
use diesel::sql_types::*;
use diesel::sql_types::Uuid as DieselUuid;
use diesel::sql_types::{Bool, Integer, Text};
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[derive(Debug, Queryable, Insertable, AsChangeset, QueryableByName)]
#[diesel(table_name = automation_rules)]
pub struct Automation {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Nullable<Text>)]
    pub sender_sensor: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub receiver_sensor: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub condition: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub action: Option<String>,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub created_at: Option<NaiveDateTime>,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub updated_at: Option<NaiveDateTime>,
    #[diesel(sql_type = Nullable<Bool>)]
    pub is_active: Option<bool>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub sender_device_type: Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub receiver_device_type: Option<i32>,
    #[diesel(sql_type = Nullable<Text>)]
    pub sender_device_name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub receiver_device_name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub trigger_type: Option<String>,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub trigger_time: Option<NaiveDateTime>,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    pub tenant_id: Option<uuid::Uuid>,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    pub user_id: Option<Uuid>,
}

pub struct AutomationFilters {
    pub user_id: Uuid,
    pub sender_sensor: Option<String>,
    pub tenant_id: Uuid,
}

pub struct AutomationTempHum {
    pub temperature: f32,
    pub humidity: f32,
}

pub struct AutomationDoor {
    pub door_status: i64,
}

#[derive(Serialize, Deserialize)]
pub struct DeviceQueueItem1 {
    #[serde(rename = "fPort")]
    pub f_port: i32,
    pub confirmed: bool,
    pub data: String,
    #[serde(rename = "devEUI")]
    pub dev_eui: String,
}

#[derive(Serialize, Deserialize)]
pub struct RequestPayload1 {
    #[serde(rename = "deviceQueueItem")]
    pub device_queue_item: DeviceQueueItem1,
}

pub async fn create_automation_rule(
    mut automation: Automation,
) -> Result<Automation, diesel::result::Error> {
    let mut conn = get_async_db_conn().await.map_err(|e| {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    // Strip out auto-set fields before insert
    automation.id = 0; // dummy, will be ignored due to SERIAL
    automation.created_at = None; // optional default
    automation.updated_at = None;
    automation.is_active = Some(true); // optional default

    let inserted: Automation = diesel::insert_into(automation_rules::table)
        .values(&automation)
        .get_result(&mut conn)
        .await?;

    Ok(inserted)
}

pub async fn get_automation_rule(id: i32) -> Result<Automation, diesel::result::Error> {
    let mut conn = get_async_db_conn().await.map_err(|e| {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    let automation: Automation = automation_rules::table
        .filter(automation_rules::id.eq(id))
        .first(&mut conn)
        .await?;

    Ok(automation)
}

pub async fn list_automation_rules(
    filters: AutomationFilters,
) -> Result<Vec<Automation>, diesel::result::Error> {
    let mut conn = get_async_db_conn().await.map_err(|e| {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    let mut query = automation_rules::table.into_boxed();

    query = query.filter(automation_rules::user_id.eq(filters.user_id));

    if let Some(sender_sensor) = filters.sender_sensor {
        query = query.filter(automation_rules::sender_sensor.eq(sender_sensor));
    }

    query = query.filter(automation_rules::tenant_id.eq(filters.tenant_id));

    let automations: Vec<Automation> = query.load(&mut conn).await?;

    Ok(automations)
}

pub async fn update_automation_rule(
    id: i32,
    automation: Automation,
) -> Result<Automation, diesel::result::Error> {
    let mut conn = get_async_db_conn().await.map_err(|e| {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    let updated_automation: Automation = diesel::update(automation_rules::table.find(id))
        .set(&automation)
        .get_result(&mut conn)
        .await?;

    Ok(updated_automation)
}

pub async fn delete_automation_rule(id: i32) -> Result<usize, diesel::result::Error> {
    let mut conn = get_async_db_conn().await.map_err(|e| {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    let deleted_count = diesel::delete(automation_rules::table.find(id))
        .execute(&mut conn)
        .await?;

    Ok(deleted_count)
}

pub async fn check_automation_rules(device_dev_eui: &str) -> Result<Vec<Automation>, Error> {
    let mut conn = get_async_db_conn().await?;

    let rules: Vec<Automation> = automation_rules::table
        .filter(automation_rules::trigger_type.eq("device"))
        .filter(automation_rules::sender_sensor.eq(device_dev_eui))
        .filter(automation_rules::is_active.eq(true))
        .load(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, device_dev_eui.to_string()))?;

    Ok(rules)
}

pub async fn check_automation_alarm_rules(alarm_id: i64) -> Result<Vec<Automation>, Error> {
    let mut conn = get_async_db_conn().await?;

    let alarm_id_str = alarm_id.to_string();

    let rules: Vec<Automation> = automation_rules::table
        .filter(automation_rules::condition.eq(alarm_id_str.clone()))
        .filter(automation_rules::is_active.eq(true))
        .load(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, alarm_id_str.clone()))?;

    Ok(rules)
}

pub async fn check_time_triggered_rules() -> Result<Vec<Automation>, Error> {
    let mut conn = get_async_db_conn().await?;

    // The query logic is ported from the original SQL.
    let query = r#"
        SELECT * FROM automation_rules
        WHERE trigger_type = 'time'
        AND is_active = TRUE
        AND (
            SELECT array_agg(trim(day)::integer)
            FROM unnest(string_to_array(split_part(condition, ';', 1), ',')) AS day
        ) @> ARRAY[EXTRACT(DOW FROM NOW() + INTERVAL '3 hours')::integer]
        AND TO_CHAR(NOW() + INTERVAL '3 hours', 'HH24:MI') = trim(split_part(condition, ';', 2))
    "#;

    let rules: Vec<Automation> = sql_query(query)
        .load(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, "".to_string()))?;

    Ok(rules)
}

pub async fn check_automation_condition(
    object_json: &str,
    rule: &Automation,
) -> Result<bool, Error> {
    let values: Vec<&str> = match &rule.condition {
        Some(cond) => cond.split(',').collect(),
        None => return Err(Error::Validation("No condition specified".to_string())),
    };

    let sender_device_type = rule.sender_device_type.unwrap_or(0);

    match sender_device_type {
        1 => {
            // LSN50V2
            let value: Value = serde_json::from_str(object_json)
                .map_err(|e| Error::Validation(format!("Error parsing JSON: {}", e)))?;
            let f: f32 = values.get(2).and_then(|v| v.parse().ok()).unwrap_or(0.0);

            if let Some(param) = values.get(0) {
                match *param {
                    "temperature" => {
                        let temp = value
                            .get("Temperature")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;
                        match values.get(1).map(|s| *s) {
                            Some("over") => Ok(temp > f),
                            Some("below") => Ok(temp < f),
                            _ => Err(Error::Validation("Invalid comparison".to_string())),
                        }
                    }
                    "humadity" => {
                        let hum = value
                            .get("Humidity")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;
                        match values.get(1).map(|s| *s) {
                            Some("over") => Ok(hum > f),
                            Some("below") => Ok(hum < f),
                            _ => Err(Error::Validation("Invalid comparison".to_string())),
                        }
                    }
                    _ => Err(Error::Validation("Invalid parameter".to_string())),
                }
            } else {
                Err(Error::Validation("Missing parameter".to_string()))
            }
        }
        12 => {
            // EM300TH
            let value: Value = serde_json::from_str(object_json)
                .map_err(|e| Error::Validation(format!("Error parsing JSON: {}", e)))?;
            let f: f32 = values.get(2).and_then(|v| v.parse().ok()).unwrap_or(0.0);

            if let Some(param) = values.get(0) {
                match *param {
                    "temperature" => {
                        let temp = value
                            .get("Temperature")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;
                        match values.get(1).map(|s| *s) {
                            Some("over") => Ok(temp > f),
                            Some("below") => Ok(temp < f),
                            _ => Err(Error::Validation("Invalid comparison".to_string())),
                        }
                    }
                    "humadity" => {
                        let hum = value
                            .get("Humidity")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;
                        match values.get(1).map(|s| *s) {
                            Some("over") => Ok(hum > f),
                            Some("below") => Ok(hum < f),
                            _ => Err(Error::Validation("Invalid comparison".to_string())),
                        }
                    }
                    _ => Err(Error::Validation("Invalid parameter".to_string())),
                }
            } else {
                Err(Error::Validation("Missing parameter".to_string()))
            }
        }
        3 => {
            // LDS01 (door)
            let value: Value = serde_json::from_str(object_json)
                .map_err(|e| Error::Validation(format!("Error parsing JSON: {}", e)))?;
            if let Some(param) = values.get(0) {
                if *param == "status" {
                    let door_status = value
                        .get("DoorStatus")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(-1);
                    match values.get(1).map(|s| *s) {
                        Some("open") => Ok(door_status == 1),
                        Some("close") => Ok(door_status == 0),
                        _ => Err(Error::Validation("Invalid comparison".to_string())),
                    }
                } else {
                    Err(Error::Validation("Invalid parameter".to_string()))
                }
            } else {
                Err(Error::Validation("Missing parameter".to_string()))
            }
        }
        18 | 19 => {
            // EM300ZLD (water leak)
            let value: Value = serde_json::from_str(object_json)
                .map_err(|e| Error::Validation(format!("Error parsing JSON: {}", e)))?;
            if let Some(param) = values.get(0) {
                if *param == "status" {
                    let water_leek = value
                        .get("WaterLeek")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(-1);
                    match values.get(1).map(|s| *s) {
                        Some("leak") => Ok(water_leek == 1),
                        Some("noleak") => Ok(water_leek == 0),
                        _ => Err(Error::Validation("Invalid comparison".to_string())),
                    }
                } else {
                    Err(Error::Validation("Invalid parameter".to_string()))
                }
            } else {
                Err(Error::Validation("Missing parameter".to_string()))
            }
        }
        33 => {
            // EM400MUD (distance)
            let value: Value = serde_json::from_str(object_json)
                .map_err(|e| Error::Validation(format!("Error parsing JSON: {}", e)))?;
            let f: f32 = values.get(2).and_then(|v| v.parse().ok()).unwrap_or(0.0);

            if let Some(param) = values.get(0) {
                if *param == "distance" {
                    let distance = value
                        .get("Distance")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32
                        / 1000.0;
                    match values.get(1).map(|s| *s) {
                        Some("over") => Ok(distance > f),
                        Some("below") => Ok(distance < f),
                        _ => Err(Error::Validation("Invalid comparison".to_string())),
                    }
                } else {
                    Err(Error::Validation("Invalid parameter".to_string()))
                }
            } else {
                Err(Error::Validation("Missing parameter".to_string()))
            }
        }
        _ => Err(Error::Validation("Unsupported device type".to_string())),
    }
}

pub async fn check_state(rule: &Automation) -> Result<bool, Error> {
    let mut conn = get_async_db_conn().await?;

    match rule.receiver_device_type {
        Some(6) => {
            #[derive(QueryableByName, Debug)]
            struct CurrentState {
                #[sql_type = "Nullable<Text>"]
                gpio_out_1: Option<String>,
                #[sql_type = "Nullable<Text>"]
                gpio_out_2: Option<String>,
            }

            let receiver_sensor = match &rule.receiver_sensor {
                Some(s) => s,
                None => return Ok(false),
            };

            let query = r#"
                SELECT gpio_out_1, gpio_out_2
                FROM device_data_latest
                WHERE dev_eui = $1
            "#;

            let state: Option<CurrentState> = sql_query(query)
                .bind::<Text, _>(receiver_sensor)
                .get_result(&mut conn)
                .await
                .ok();

            if let Some(current_state) = state {
                let gpio_out_1 = current_state.gpio_out_1.unwrap_or_default();
                let gpio_out_2 = current_state.gpio_out_2.unwrap_or_default();

                match rule.action.as_deref() {
                    Some("AwAB") if gpio_out_1 == "0" || gpio_out_2 == "1" => return Ok(true),
                    Some("AwAA") if gpio_out_1 == "0" || gpio_out_2 == "0" => return Ok(true),
                    Some("AwEA") if gpio_out_1 == "1" || gpio_out_2 == "0" => return Ok(true),
                    Some("AwEB") if gpio_out_1 == "1" || gpio_out_2 == "1" => return Ok(true),
                    _ => return Ok(false),
                }
            }
            Ok(false)
        }
        Some(28) => {
            #[derive(QueryableByName, Debug)]
            struct LastRow {
                #[sql_type = "Nullable<Text>"]
                adc_1: Option<String>,
                #[sql_type = "Nullable<Text>"]
                adc_2: Option<String>,
                #[sql_type = "Nullable<Text>"]
                adv_1: Option<String>,
                #[sql_type = "Nullable<Text>"]
                gpio_in_1: Option<String>,
                #[sql_type = "Nullable<Text>"]
                gpio_in_2: Option<String>,
                #[sql_type = "Nullable<Text>"]
                gpio_in_3: Option<String>,
                #[sql_type = "Nullable<Text>"]
                gpio_in_4: Option<String>,
                #[sql_type = "Nullable<Text>"]
                gpio_out_1: Option<String>,
                #[sql_type = "Nullable<Text>"]
                gpio_out_2: Option<String>,
            }

            let receiver_sensor = match &rule.receiver_sensor {
                Some(s) => s,
                None => return Ok(false),
            };

            let query = r#"
                SELECT adc_1, adc_2, adv_1, gpio_in_1, gpio_in_2, gpio_in_3, gpio_in_4, gpio_out_1, gpio_out_2
                FROM uc300
                WHERE dev_eui = $1
                ORDER BY id DESC
                LIMIT 1
            "#;

            let last_row: Option<LastRow> = sql_query(query)
                .bind::<Text, _>(receiver_sensor)
                .get_result(&mut conn)
                .await
                .ok();

            if let Some(row) = last_row {
                let gpio_out_1 = row.gpio_out_1.unwrap_or_default();
                let gpio_out_2 = row.gpio_out_2.unwrap_or_default();

                match rule.action.as_deref() {
                    Some("BwAA/w==") if gpio_out_1 == "1" => return Ok(true),
                    Some("BwEA/w==") if gpio_out_1 == "0" => return Ok(true),
                    Some("CAAA/w==") if gpio_out_2 == "1" => return Ok(true),
                    Some("CAEA/w==") if gpio_out_2 == "0" => return Ok(true),
                    _ => return Ok(false),
                }
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

// pub async fn execute_automation_action(rule: &Automation) -> Result<(), Error> {
//     let receiver_sensor = rule
//         .receiver_sensor
//         .as_ref()
//         .ok_or_else(|| Error::from("Missing receiver_sensor"))?;

//     let port = match rule.receiver_device_type {
//         Some(6) => 8,
//         Some(27) | Some(28) => 85,
//         Some(t) => {
//             log::warn!("Unknown ReceiverDeviceType: {}", t);
//             return Ok(());
//         }
//         None => return Err(Error::from("Missing receiver_device_type")),
//     };

//     let action = rule
//         .action
//         .as_ref()
//         .ok_or_else(|| Error::from("Missing action"))?;

//     if rule.receiver_device_type == Some(28) {
//         let commands: Vec<&str> = action.split(';').collect();
//         if commands.len() != 2 {
//             log::warn!("Expected two commands for device type 28");
//             return Ok(());
//         }

//         for (i, cmd) in commands.iter().enumerate() {
//             let decoded = match base64_engine::STANDARD.decode(cmd) {
//                 Ok(d) => d,
//                 Err(e) => {
//                     log::warn!("Error decoding command {}: {}", i + 1, e);
//                     return Ok(());
//                 }
//             };

//             enqueue_device_queue_item(receiver_sensor, port, &decoded).await?;

//             if i == 0 {
//                 sleep(Duration::from_secs(10)).await;
//             }
//         }
//     } else {
//         let url = format!(
//             "https://cloud.vaps.com.tr:8443/api/devices/{}/queue",
//             receiver_sensor
//         );

//         let payload = RequestPayload1 {
//             device_queue_item: DeviceQueueItem1 {
//                 f_port: port as i32,
//                 confirmed: true,
//                 data: action.clone(),
//                 dev_eui: receiver_sensor.clone(),
//             },
//         };

//         let client = Client::new();
//         let token = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."; // shortened for brevity

//         let res = client
//             .post(url)
//             .header("Content-Type", "application/json")
//             .header("grpc-metadata-authorization", token)
//             .json(&payload)
//             .send()
//             .await
//             .map_err(|e| Error::from(format!("HTTP error: {}", e)))?;

//         log::info!("Response Status: {}", res.status());
//     }

//     Ok(())
// }

// pub async fn enqueue_device_queue_item(
//     dev_eui: &str,
//     port: u32,
//     data: &[u8],
// ) -> Result<(), Error> {
//     let url = format!("https://cloud.vaps.com.tr:8443/api/devices/{}/queue", dev_eui);
//     let payload = RequestPayload1 {
//         device_queue_item: DeviceQueueItem1 {
//             f_port: port as i32,
//             confirmed: true,
//             data: base64_engine::STANDARD.encode(data),
//             dev_eui: dev_eui.to_string(),
//         },
//     };

//     let client = Client::new();
//     let token = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."; // same token

//     let res = client
//         .post(url)
//         .header("Content-Type", "application/json")
//         .header("grpc-metadata-authorization", token)
//         .json(&payload)
//         .send()
//         .await
//         .map_err(|e| Error::from(format!("HTTP error: {}", e)))?;

//     log::info!("Enqueue Response Status: {}", res.status());
//     Ok(())
// }

// pub async fn enqueue_device_queue_item_internal(
//     dev_eui: &str,
//     port: u32,
//     data: &[u8],
//     json_object: Option<&str>,
// ) -> Result<(), Error> {
//     // 1. Lock device row (future implementation)
//     // 2. If JSON object is set, convert using codec
//     // 3. Insert into DB via Diesel

//     log::info!(
//         "EnqueueDeviceQueueItemInternal → dev_eui={}, port={}, data={:?}, json={:?}",
//         dev_eui,
//         port,
//         data,
//         json_object
//     );

//     // Placeholder for Diesel insert:
//     // diesel::insert_into(device_queue::table)
//     //     .values(...)
//     //     .execute(&mut conn).await?;

//     Ok(())
// }
