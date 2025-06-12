use super::application::Application;
use super::device::Device;
use super::notification;
use super::{error::Error, get_async_db_conn};
use crate::storage::schema_postgres::alarm;
use crate::storage::schema_postgres::alarm_audit_log;
use crate::storage::schema_postgres::alarm_automation_rules;
use crate::storage::schema_postgres::alarm_date_time;
use anyhow::{Context, Result};
use bigdecimal::ToPrimitive;
use chirpstack_api::api;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::{Datelike, Local, Timelike};
use diesel::deserialize::QueryableByName;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Uuid as DieselUuid;
use diesel::sql_types::*;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(
    Queryable,
    QueryableByName,
    Insertable,
    AsChangeset,
    PartialEq,
    Debug,
    Clone,
    Serialize,
    Deserialize,
)]
#[diesel(table_name = alarm)]
pub struct Alarm {
    pub id: i32,
    pub dev_eui: String,
    pub min_treshold: Option<f64>,
    pub max_treshold: Option<f64>,
    pub sms: Option<bool>,
    pub email: Option<bool>,
    pub temperature: Option<bool>,
    pub humadity: Option<bool>,
    pub ec: Option<bool>,
    pub door: Option<bool>,
    pub w_leak: Option<bool>,
    pub is_time_limit_active: Option<bool>,
    pub alarm_start_time: Option<f64>,
    pub alarm_stop_time: Option<f64>,
    pub zone_category: Option<i32>,
    pub notification: Option<bool>,
    pub is_active: Option<bool>,
    pub pressure: Option<bool>,
    pub notification_sound: Option<String>,
    pub distance: Option<bool>,
    pub defrost_time: Option<i32>,
    pub user_id: Vec<Option<Uuid>>,
}

#[derive(Insertable)]
#[diesel(table_name = alarm)]
pub struct NewAlarm {
    pub dev_eui: String,
    pub min_treshold: Option<f64>,
    pub max_treshold: Option<f64>,
    pub sms: Option<bool>,
    pub email: Option<bool>,
    pub temperature: Option<bool>,
    pub humadity: Option<bool>,
    pub ec: Option<bool>,
    pub door: Option<bool>,
    pub w_leak: Option<bool>,
    pub is_time_limit_active: Option<bool>,
    pub alarm_start_time: Option<f64>,
    pub alarm_stop_time: Option<f64>,
    pub zone_category: Option<i32>,
    pub notification: Option<bool>,
    pub is_active: Option<bool>,
    pub pressure: Option<bool>,
    pub notification_sound: Option<String>,
    pub distance: Option<bool>,
    pub defrost_time: Option<i32>,
    pub user_id: Vec<Option<Uuid>>,
}
impl Default for Alarm {
    fn default() -> Self {
        Self {
            id: 0,
            dev_eui: String::new(),
            min_treshold: None,
            max_treshold: None,
            sms: None,
            email: None,
            temperature: None,
            humadity: None,
            ec: None,
            door: None,
            w_leak: None,
            is_time_limit_active: None,
            alarm_start_time: None,
            alarm_stop_time: None,
            zone_category: None,
            notification: None,
            is_active: Some(true),
            pressure: None,
            notification_sound: Some("default".to_string()),
            user_id: vec![None],
            distance: None,
            defrost_time: Some(0),
        }
    }
}

impl Default for NewAlarm {
    fn default() -> Self {
        Self {
            dev_eui: String::new(),
            min_treshold: None,
            max_treshold: None,
            sms: None,
            email: None,
            temperature: None,
            humadity: None,
            ec: None,
            door: None,
            w_leak: None,
            is_time_limit_active: None,
            alarm_start_time: None,
            alarm_stop_time: None,
            zone_category: None,
            notification: None,
            is_active: Some(true),
            pressure: None,
            notification_sound: Some("default".to_string()),
            user_id: vec![None],
            distance: None,
            defrost_time: Some(0),
        }
    }
}
#[derive(Debug, QueryableByName, Serialize, Deserialize)]
#[diesel(check_for_backend(Pg))]
pub struct OrganizationAlarmRaw {
    #[diesel(sql_type = Integer)]
    pub id: i32,

    #[diesel(sql_type = Text)]
    pub dev_eui: String,

    #[diesel(sql_type = Nullable<Double>)]
    pub min_treshold: Option<f64>,

    #[diesel(sql_type = Nullable<Double>)]
    pub max_treshold: Option<f64>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub sms: Option<bool>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub email: Option<bool>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub notification: Option<bool>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub temperature: Option<bool>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub humadity: Option<bool>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub ec: Option<bool>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub door: Option<bool>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub w_leak: Option<bool>,

    #[diesel(sql_type = Array<Nullable<DieselUuid>>)]
    pub user_id: Vec<Option<Uuid>>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub is_time_limit_active: Option<bool>,

    #[diesel(sql_type = Nullable<Double>)]
    pub alarm_start_time: Option<f64>,

    #[diesel(sql_type = Nullable<Double>)]
    pub alarm_stop_time: Option<f64>,

    #[diesel(sql_type = Nullable<Integer>)]
    pub zone_category: Option<i32>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub is_active: Option<bool>,

    #[diesel(sql_type = Nullable<Text>)]
    pub zone_name: Option<String>,

    #[diesel(sql_type = Nullable<Text>)]
    pub device_name: Option<String>,

    #[diesel(sql_type = Nullable<Text>)]
    pub username: Option<String>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub pressure: Option<bool>,

    #[diesel(sql_type = Nullable<Text>)]
    pub notification_sound: Option<String>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub distance: Option<bool>,

    #[diesel(sql_type = Nullable<Integer>)]
    pub time: Option<i32>,

    #[diesel(sql_type = Nullable<Integer>)]
    pub defrost_time: Option<i32>,
    // pub alarm_date_time: Option<AlarmDateTime>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationAlarm {
    pub id: i32,
    pub dev_eui: String,
    pub min_treshold: Option<f64>,
    pub max_treshold: Option<f64>,
    pub sms: Option<bool>,
    pub email: Option<bool>,
    pub notification: Option<bool>,
    pub temperature: Option<bool>,
    pub humadity: Option<bool>,
    pub ec: Option<bool>,
    pub door: Option<bool>,
    pub w_leak: Option<bool>,
    pub user_id: Vec<Option<Uuid>>,
    pub is_time_limit_active: Option<bool>,
    pub alarm_start_time: Option<f64>,
    pub alarm_stop_time: Option<f64>,
    pub zone_category: Option<i32>,
    pub is_active: Option<bool>,
    pub zone_name: Option<String>,
    pub device_name: Option<String>,
    pub username: Option<String>,
    pub pressure: Option<bool>,
    pub notification_sound: Option<String>,
    pub distance: Option<bool>,
    pub time: Option<i32>,
    pub defrost_time: Option<i32>,
    pub alarm_date_time: Option<Vec<AlarmDateTime>>,
}

#[derive(Queryable, Insertable, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = alarm_date_time)]
pub struct AlarmDateTime {
    pub alarm_id: i32,
    pub alarm_day: i32,
    pub start_time: f64,
    pub end_time: f64,
    pub id: i32,
}

#[derive(Queryable, QueryableByName, Debug, Clone, Serialize, Deserialize)]
#[diesel(check_for_backend(Pg))]

pub struct AlarmWithDates {
    #[diesel(sql_type = Int8)]
    pub id: i64,

    #[diesel(sql_type = Text)]
    pub dev_eui: String,

    #[diesel(sql_type = Float4)]
    pub min_treshold: f32,

    #[diesel(sql_type = Float4)]
    pub max_treshold: f32,

    #[diesel(sql_type = Bool)]
    pub sms: bool,

    #[diesel(sql_type = Bool)]
    pub notification: bool,

    #[diesel(sql_type = Bool)]
    pub email: bool,

    #[diesel(sql_type = Bool)]
    pub temperature: bool,

    #[diesel(sql_type = Bool)]
    pub humadity: bool,

    #[diesel(sql_type = Bool)]
    pub ec: bool,

    #[diesel(sql_type = Bool)]
    pub door: bool,

    #[diesel(sql_type = Bool)]
    pub w_leak: bool,

    #[diesel(sql_type = Array<Nullable<DieselUuid>>)]
    pub user_id: Vec<Option<Uuid>>,

    #[diesel(sql_type = Nullable<Text>)]
    pub ip_address: Option<String>,

    #[diesel(sql_type = Bool)]
    pub is_time_limit_active: bool,

    #[diesel(sql_type = BigInt)]
    pub zone_category_id: i64,

    #[diesel(sql_type = BigInt)]
    pub alarm_day: i64,

    #[diesel(sql_type = Float4)]
    pub alarm_start_time2: f32,

    #[diesel(sql_type = Float4)]
    pub alarm_stop_time2: f32,

    #[diesel(sql_type = Float4)]
    pub start_time: f32,

    #[diesel(sql_type = Float4)]
    pub end_time: f32,

    #[diesel(sql_type = Bool)]
    pub is_active: bool,

    #[diesel(sql_type = Bool)]
    pub pressure: bool,

    #[diesel(sql_type = Float4)]
    pub current: f32,

    #[diesel(sql_type = Float4)]
    pub factor: f32,

    #[diesel(sql_type = Float4)]
    pub power: f32,

    #[diesel(sql_type = Float4)]
    pub voltage: f32,

    #[diesel(sql_type = BigInt)]
    pub status: i64,

    #[diesel(sql_type = Float4)]
    pub power_sum: f32,

    #[diesel(sql_type = Bool)]
    pub distance: bool,

    #[diesel(sql_type = Bool)]
    pub co2: bool,

    #[diesel(sql_type = Nullable<Text>)]
    pub notification_sound: Option<String>,

    #[diesel(sql_type = BigInt)]
    pub defrost_time: i64,
}

impl AlarmWithDates {
    pub fn is_within_schedule(&self, current_time: NaiveTime) -> bool {
        if !self.is_time_limit_active {
            return true;
        }

        let time = current_time.hour() as f32 + current_time.minute() as f32 / 60.0 + 3.0;
        let adjusted_time = if time >= 24.0 { time - 24.0 } else { time };

        if self.end_time > self.start_time {
            self.start_time < adjusted_time && adjusted_time < self.end_time
        } else {
            (self.start_time < adjusted_time && adjusted_time < 24.0)
                || (0.0 < adjusted_time && adjusted_time < self.end_time)
        }
    }
}
#[derive(AsChangeset, Debug)]
#[diesel(table_name = alarm_automation_rules)]
pub struct UpdateAlarmAutomation {
    pub id: i32,
    pub alarm_id: i32,
    pub receiver_sensor: String,
    pub action: Option<String>,
    pub is_active: Option<bool>,
    pub receiver_device_type: Option<i32>,
    pub receiver_device_name: Option<String>,
    pub user_id: Option<Uuid>,
}

#[derive(Queryable, QueryableByName, Insertable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = alarm_audit_log)]
pub struct AlarmAuditLog {
    pub id: i32,
    pub alarm_id: i32,
    pub dev_eui: Option<String>,
    pub change_type: Option<String>,
    pub changed_at: Option<NaiveDateTime>,
    pub changed_by: Option<Uuid>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
}

impl Default for AlarmDateTime {
    fn default() -> Self {
        Self {
            alarm_id: 0,
            alarm_day: 0,
            start_time: 0.0,
            end_time: 0.0,
            id: 0,
        }
    }
}

#[derive(Debug, Queryable, QueryableByName, Serialize, Deserialize, Clone)]
pub struct DoorTimeAlarm {
    #[diesel(sql_type = Integer)]
    pub id: i32,

    #[diesel(sql_type = Nullable<Varchar>)]
    pub dev_eui: Option<String>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub sms: Option<bool>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub email: Option<bool>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub notification: Option<bool>,

    #[diesel(sql_type = Nullable<Timestamp>)]
    pub submission_time: Option<chrono::NaiveDateTime>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub is_active: Option<bool>,

    #[diesel(sql_type = Nullable<Int8>)]
    pub time: Option<i64>,

    #[diesel(sql_type = Nullable<Array<Nullable<DieselUuid>>>)]
    pub user_id: Option<Vec<Option<Uuid>>>,

    #[diesel(sql_type = Nullable<DieselUuid>)]
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AlarmFilters {
    pub limit: i32,
    pub dev_eui: String,
    pub user_id: Uuid,
}
#[derive(Debug, AsChangeset)]
#[diesel(table_name = alarm)]
pub struct UpdateAlarm {
    pub min_treshold: Option<f64>,
    pub max_treshold: Option<f64>,
    pub sms: Option<bool>,
    pub email: Option<bool>,
    pub notification: Option<bool>,
    pub is_time_limit_active: Option<bool>,
    pub notification_sound: Option<String>,
    pub user_id: Vec<uuid::Uuid>,
    pub is_active: Option<bool>,
    pub defrost_time: Option<i32>,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = alarm_automation_rules)]
pub struct AlarmAutomation {
    pub id: i32,
    pub alarm_id: i32,
    pub receiver_sensor: String,
    pub action: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub is_active: Option<bool>,
    pub receiver_device_type: Option<i32>,
    pub receiver_device_name: Option<String>,
    pub user_id: Option<Uuid>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = alarm_automation_rules)]
pub struct NewAlarmAutomation {
    pub alarm_id: i32,
    pub receiver_sensor: String,
    pub action: Option<String>,
    pub is_active: Option<bool>,
    pub receiver_device_type: Option<i32>,
    pub receiver_device_name: Option<String>,
    pub user_id: Option<Uuid>,
}

pub async fn create(
    alarm: NewAlarm,
    date_filters: Vec<AlarmDateTime>,
    sent_user_id: Uuid,
) -> Result<Alarm, Error> {
    let mut conn = get_async_db_conn().await?;

    let a: Alarm = diesel::insert_into(alarm::table)
        .values(&alarm)
        .get_result::<Alarm>(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, alarm.dev_eui.to_string()))?;

    log_audit(
        a.id as i64,
        &a.dev_eui,
        sent_user_id,
        "CREATE",
        None,
        Some(serde_json::to_value(&a).map_err(|e| Error::from(anyhow::Error::new(e)))?),
    )
    .await?;

    for df in &date_filters {
        let new_df = AlarmDateTime {
            id: df.id,
            alarm_id: a.id,
            alarm_day: df.alarm_day,
            start_time: df.start_time,
            end_time: df.end_time,
        };

        diesel::insert_into(alarm_date_time::table)
            .values(&new_df)
            .execute(&mut conn)
            .await
            .map_err(|e| Error::from_diesel(e, a.dev_eui.to_string()))?;
    }

    info!(id = %a.id, "Alarm created");
    Ok(a)
}
pub async fn get_alarm(alarm_id: i32) -> Result<Alarm, Error> {
    let mut conn = get_async_db_conn().await?;

    let alarm: Alarm = alarm::table
        .filter(alarm::id.eq(alarm_id))
        .first(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, alarm_id.to_string()))?;

    info!(alarm_id = %alarm_id, "Alarm fetched");
    Ok(alarm)
}

pub async fn get_alarm_dates(alarm_id: i32) -> Result<Vec<AlarmDateTime>, Error> {
    let mut conn = get_async_db_conn().await?;

    let dates: Vec<AlarmDateTime> = alarm_date_time::table
        .filter(alarm_date_time::alarm_id.eq(alarm_id))
        .load(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, alarm_id.to_string()))?;

    info!(alarm_id, "Alarm dates fetched");
    Ok(dates)
}

pub async fn get_organization_alarm_list(tenant_id: Uuid) -> Result<Vec<OrganizationAlarm>, Error> {
    let mut conn = get_async_db_conn().await?;
    info!("Alarm get_organization_alarm_list start");
    let raw_alarms = diesel::sql_query(
        r#"
        SELECT 
            a.id,
            a.dev_eui,
            a.min_treshold,
            a.max_treshold,
            a.sms,
            a.email,
            a.notification,
            a.temperature,
            a.humadity,
            a.ec,
            a.door,
            a.w_leak,
            a.user_id,
            a.is_time_limit_active,
            a.alarm_start_time,
            a.alarm_stop_time,
            a.zone_category,
            a.is_active,
            z.zone_name,
            d.name AS device_name,
            '' AS username,
            a.pressure,
            a.notification_sound,
            a.distance,
            0 AS time,
            a.defrost_time
        FROM alarm AS a
        INNER JOIN device AS d ON d.dev_eui::text = '\x' || a.dev_eui
        INNER JOIN zone AS z ON d.dev_eui::text = ANY(z.devices)
        WHERE d.tenant_id = $1
        "#,
    )
    .bind::<DieselUuid, _>(tenant_id)
    .load::<OrganizationAlarmRaw>(&mut conn)
    .await
    .map_err(|e| Error::from_diesel(e, tenant_id.to_string()))?;

    info!(tenant_id = %tenant_id, "Alarms fetched");

    if raw_alarms.is_empty() {
        return Ok(vec![]);
    }

    // Step 2: Extract alarm_ids
    let alarm_ids: Vec<i32> = raw_alarms.iter().map(|a| a.id).collect();
    info!("Collected alarm_ids: {:?}", alarm_ids);

    // Step 3: Load only related alarm_date_time rows
    let date_time_map = fetch_alarm_date_times_by_ids(&alarm_ids).await?;
    // Optionally dump a few entries
    for (id, dates) in date_time_map.iter().take(5) {
        info!("alarm_id = {}, dates = {:?}", id, dates);
    }
    // Step 4: Map raw alarms with corresponding alarm_date_time entries
    let enriched_alarms: Vec<OrganizationAlarm> = raw_alarms
        .into_iter()
        .map(|raw| {
            let dates = date_time_map.get(&raw.id).cloned().unwrap_or_default();

            if dates.is_empty() {
                warn!("No alarm_date_time found for alarm_id = {}", raw.id);
            } else {
                info!("Found {} date(s) for alarm_id = {}", dates.len(), raw.id);
            }

            map_alarm_with_dates(raw, dates)
        })
        .collect();

    Ok(enriched_alarms)
}

pub async fn fetch_alarm_date_times_by_ids(
    alarm_ids: &[i32],
) -> Result<HashMap<i32, Vec<AlarmDateTime>>> {
    let mut conn = get_async_db_conn().await?;

    let results: Vec<AlarmDateTime> = alarm_date_time::table
        .filter(alarm_date_time::alarm_id.eq_any(alarm_ids))
        .load::<AlarmDateTime>(&mut conn)
        .await?;

    let mut map = HashMap::new();
    for item in results {
        map.entry(item.alarm_id).or_insert_with(Vec::new).push(item);
    }

    Ok(map)
}

pub fn map_alarm_with_dates(
    raw: OrganizationAlarmRaw,
    dates: Vec<AlarmDateTime>,
) -> OrganizationAlarm {
    OrganizationAlarm {
        id: raw.id,
        dev_eui: raw.dev_eui,
        min_treshold: raw.min_treshold,
        max_treshold: raw.max_treshold,
        sms: raw.sms,
        email: raw.email,
        notification: raw.notification,
        temperature: raw.temperature,
        humadity: raw.humadity,
        ec: raw.ec,
        door: raw.door,
        w_leak: raw.w_leak,
        user_id: raw.user_id,
        is_time_limit_active: raw.is_time_limit_active,
        alarm_start_time: raw.alarm_start_time,
        alarm_stop_time: raw.alarm_stop_time,
        zone_category: raw.zone_category,
        is_active: raw.is_active,
        zone_name: raw.zone_name,
        device_name: raw.device_name,
        username: raw.username,
        pressure: raw.pressure,
        notification_sound: raw.notification_sound,
        distance: raw.distance,
        time: raw.time,
        defrost_time: raw.defrost_time,
        alarm_date_time: Some(dates),
    }
}

pub async fn update_alarm(
    alarm_id: i32,
    updated_fields: UpdateAlarm,
    date_filters: Vec<AlarmDateTime>,
    sent_user_id: Uuid,
) -> Result<(), Error> {
    let mut conn = get_async_db_conn().await?;

    let existing_alarm: Alarm = alarm::table
        .filter(alarm::id.eq(alarm_id))
        .first(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, alarm_id.to_string()))?;

    // This performs the update and fetches the updated row
    let updated_alarm: Alarm = diesel::update(alarm::table.filter(alarm::id.eq(alarm_id)))
        .set(&updated_fields)
        .get_result(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, alarm_id.to_string()))?;

    log_audit(
        updated_alarm.id as i64,
        &updated_alarm.dev_eui,
        sent_user_id,
        "UPDATE",
        Some(
            serde_json::to_value(&existing_alarm)
                .map_err(|e| Error::from(anyhow::Error::new(e)))?,
        ),
        Some(serde_json::to_value(&updated_alarm).map_err(|e| Error::from(anyhow::Error::new(e)))?),
    )
    .await?;

    diesel::delete(alarm_date_time::table.filter(alarm_date_time::alarm_id.eq(updated_alarm.id)))
        .execute(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, updated_alarm.dev_eui.to_string()))?;

    for df in &date_filters {
        let new_df = (
            alarm_date_time::alarm_id.eq(updated_alarm.id),
            alarm_date_time::alarm_day.eq(df.alarm_day),
            alarm_date_time::start_time.eq(df.start_time),
            alarm_date_time::end_time.eq(df.end_time),
        );

        diesel::insert_into(alarm_date_time::table)
            .values(&new_df)
            .execute(&mut conn)
            .await
            .map_err(|e| Error::from_diesel(e, updated_alarm.dev_eui.to_string()))?;
    }

    info!(dev_eui = %updated_alarm.dev_eui, "Alarm updated");
    Ok(())
}

pub async fn delete_alarm(alarm_id: i32, sent_user_id: Uuid) -> Result<(), Error> {
    let mut conn = get_async_db_conn().await?;

    let alarm: Alarm = alarm::table
        .filter(alarm::id.eq(alarm_id))
        .first(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, alarm_id.to_string()))?;

    log_audit(
        alarm.id as i64,
        &alarm.dev_eui,
        sent_user_id,
        "DELETE",
        Some(serde_json::to_value(&alarm).map_err(|e| Error::from(anyhow::Error::new(e)))?),
        None,
    )
    .await?;

    diesel::delete(alarm::table.filter(alarm::id.eq(alarm_id)))
        .execute(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, alarm_id.to_string()))?;

    info!(alarm_id = alarm_id, "Alarm deleted");
    Ok(())
}

pub async fn delete_user_alarm(user_id: Uuid, sent_user_id: Uuid) -> Result<(), Error> {
    let mut conn = get_async_db_conn().await?;

    let query = r#"
    WITH updated_rows AS (
        UPDATE alarm
        SET user_id = array_remove(user_id, $1::uuid)
        WHERE $1 = ANY(user_id)
        RETURNING *
    )
    SELECT * FROM updated_rows;
    "#;

    let alarms: Vec<Alarm> = sql_query(query)
        .bind::<DieselUuid, _>(user_id)
        .load(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, user_id.to_string()))?;

    for alarm in &alarms {
        if let user_ids = &alarm.user_id {
            if user_ids.is_empty() {
                log_audit(
                    alarm.id as i64,
                    &alarm.dev_eui,
                    sent_user_id,
                    "DELETE",
                    Some(
                        serde_json::to_value(alarm)
                            .map_err(|e| Error::from(anyhow::Error::new(e)))?,
                    ),
                    None,
                )
                .await?;

                diesel::delete(alarm::table.filter(alarm::id.eq(alarm.id)))
                    .execute(&mut conn)
                    .await
                    .map_err(|e| Error::from_diesel(e, alarm.id.to_string()))?;
            }
        }
    }

    info!(user_id = %user_id, "User alarm deleted");
    Ok(())
}

pub async fn delete_sensor_alarm(dev_eui: &str, sent_user_id: Uuid) -> Result<(), Error> {
    let mut conn = get_async_db_conn().await?;

    let alarm: Alarm = alarm::table
        .filter(alarm::dev_eui.eq(dev_eui))
        .first(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, dev_eui.to_string()))?;

    log_audit(
        alarm.id as i64,
        dev_eui,
        sent_user_id,
        "DELETE",
        Some(serde_json::to_value(&alarm).map_err(|e| Error::from(anyhow::Error::new(e)))?),
        None,
    )
    .await?;

    diesel::delete(alarm::table.filter(alarm::dev_eui.eq(dev_eui)))
        .execute(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, dev_eui.to_string()))?;

    info!(dev_eui = %dev_eui, "Sensor alarm deleted");
    Ok(())
}

pub async fn delete_zone_alarm(zone_id: i32, sent_user_id: Uuid) -> Result<(), Error> {
    let mut conn = get_async_db_conn().await?;

    let alarms: Vec<Alarm> = alarm::table
        .filter(alarm::zone_category.eq(zone_id))
        .load(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, zone_id.to_string()))?;

    if alarms.is_empty() {
        return Err(Error::NotFound("No alarms found for the given zone".into()));
    }

    for alarm in &alarms {
        let alarm_json = serde_json::to_value(alarm)
            .map_err(|e| Error::from(anyhow::anyhow!("Serialization error: {}", e)))?;

        log_audit(
            alarm.id as i64,
            &alarm.dev_eui,
            sent_user_id,
            "DELETE",
            Some(alarm_json),
            None,
        )
        .await?;
    }

    let affected = diesel::delete(alarm::table.filter(alarm::zone_category.eq(zone_id)))
        .execute(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, zone_id.to_string()))?;

    info!(zone_id, affected_rows = affected, "Zone alarms deleted");
    Ok(())
}

pub async fn log_audit(
    alarm_id: i64,
    dev_eui: &str,
    user_id: Uuid,
    change_type: &str,
    previous_value: Option<Value>,
    new_value: Option<Value>,
) -> Result<(), Error> {
    let mut conn = get_async_db_conn().await?;

    diesel::insert_into(alarm_audit_log::table)
        .values((
            alarm_audit_log::alarm_id.eq(alarm_id as i32),
            alarm_audit_log::dev_eui.eq(dev_eui),
            alarm_audit_log::change_type.eq(change_type),
            alarm_audit_log::changed_by.eq(Some(user_id as Uuid)),
            alarm_audit_log::old_values.eq(previous_value),
            alarm_audit_log::new_values.eq(new_value),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, alarm_id.to_string()))?;

    info!(alarm_id, "Audit log created");
    Ok(())
}

pub async fn create_door_time_alarm(
    door_time_alarm: DoorTimeAlarm,
    time_schedule: Vec<AlarmDateTime>,
    sent_user_id: Uuid,
) -> Result<DoorTimeAlarm, Error> {
    let mut conn = get_async_db_conn().await?;

    // Insert into door_time_alarm table and get the created alarm with id
    let insert_query = r#"
        INSERT INTO door_time_alarm (
            dev_eui, time, is_active, sms, notification, email, user_id, submission_time, tenant_id
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, NOW(), $8
        )
        RETURNING id, dev_eui, time, is_active, sms, notification, email, user_id, submission_time, tenant_id
    "#;

    let created_alarm: DoorTimeAlarm = sql_query(insert_query)
        .bind::<Nullable<Text>, _>(&door_time_alarm.dev_eui)
        .bind::<Nullable<Int8>, _>(door_time_alarm.time)
        .bind::<Nullable<Bool>, _>(door_time_alarm.is_active)
        .bind::<Nullable<Bool>, _>(door_time_alarm.sms)
        .bind::<Nullable<Bool>, _>(door_time_alarm.notification)
        .bind::<Nullable<Bool>, _>(door_time_alarm.email)
        .bind::<Nullable<Array<Nullable<DieselUuid>>>, _>(&door_time_alarm.user_id)
        .bind::<Nullable<DieselUuid>, _>(door_time_alarm.tenant_id)
        .get_result(&mut conn)
        .await
        .map_err(|e| {
            Error::from_diesel(
                e,
                door_time_alarm
                    .dev_eui
                    .as_deref()
                    .unwrap_or_default()
                    .to_string(),
            )
        })?;

    // Insert time schedule entries
    for alarm_date_time in &time_schedule {
        let insert_time_query = r#"
            INSERT INTO door_alarm_date_time (alarm_id, alarm_day, start_time, end_time)
            VALUES ($1, $2, $3, $4)
        "#;
        sql_query(insert_time_query)
            .bind::<Int4, _>(created_alarm.id)
            .bind::<Int4, _>(alarm_date_time.alarm_day)
            .bind::<Float8, _>(alarm_date_time.start_time)
            .bind::<Float8, _>(alarm_date_time.end_time)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                Error::from_diesel(e, created_alarm.dev_eui.clone().unwrap_or_default())
            })?;
    }

    log_audit(
        created_alarm.id as i64,
        created_alarm.dev_eui.as_deref().unwrap_or_default(),
        sent_user_id,
        "CREATE",
        None,
        Some(serde_json::to_value(&created_alarm).map_err(|e| Error::from(anyhow::Error::new(e)))?),
    )
    .await?;

    info!(id = %created_alarm.id, "Door time alarm created");
    Ok(created_alarm)
}

pub async fn delete_door_time_alarm(door_alarm_id: i32) -> Result<(), Error> {
    let mut conn = get_async_db_conn().await?;

    let select_sql = r#"
        SELECT id, dev_eui, time, is_active, sms, notification, email,
                submission_time, user_id, tenant_id
        FROM door_time_alarm WHERE id = $1
    "#;

    let _door_alarm: DoorTimeAlarm = sql_query(select_sql)
        .bind::<Int4, _>(door_alarm_id)
        .get_result(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, door_alarm_id.to_string()))?;

    let update_sql = r#"
        UPDATE door_time_alarm SET is_active = FALSE WHERE id = $1
    "#;
    sql_query(update_sql)
        .bind::<Int4, _>(door_alarm_id)
        .execute(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, door_alarm_id.to_string()))?;

    let update_automation_sql = r#"
        UPDATE automation_rules SET is_active = FALSE WHERE condition = $1
    "#;
    sql_query(update_automation_sql)
        .bind::<Int4, _>(door_alarm_id)
        .execute(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, door_alarm_id.to_string()))?;

    info!(door_alarm_id = door_alarm_id, "Door time alarm disabled");
    Ok(())
}

pub async fn list_door_time_alarms(
    dev_eui: String,
) -> Result<Vec<api::CreateDoorTimeResponse>, Error> {
    let mut conn = get_async_db_conn().await?;

    let query = r#"
    SELECT 
        id, 
        dev_eui, 
        sms, 
        email, 
        notification, 
        submission_time, 
        is_active, 
        time, 
        user_id, 
        tenant_id
    FROM door_time_alarm
    WHERE dev_eui = $1
"#;

    let rows: Vec<DoorTimeAlarm> = sql_query(query)
        .bind::<Text, _>(&dev_eui)
        .load(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, dev_eui.clone()))?;

    let result = rows
        .into_iter()
        .map(|alarm| api::CreateDoorTimeResponse {
            dev_eui: alarm.dev_eui.unwrap_or_default(),
            time: alarm.time.unwrap_or_default(),
            is_active: alarm.is_active.unwrap_or(false),
            sms: alarm.sms.unwrap_or(false),
            notification: alarm.notification.unwrap_or(false),
            email: alarm.email.unwrap_or(false),
            user_id: alarm
                .user_id
                .unwrap_or_default()
                .into_iter()
                .filter_map(|id| id.map(|uuid| uuid.to_string()))
                .collect(),
            ..Default::default()
        })
        .collect();

    Ok(result)
}

pub async fn get_alarm_audit_logs(dev_eui: &str) -> Result<Vec<AlarmAuditLog>, Error> {
    let mut conn = get_async_db_conn().await?;

    let logs = diesel::sql_query(
        r#"
        SELECT * 
        FROM alarm_audit_log
        WHERE dev_eui = $1
        "#,
    )
    .bind::<diesel::sql_types::Text, _>(dev_eui)
    .load::<AlarmAuditLog>(&mut conn)
    .await
    .map_err(|e| Error::from_diesel(e, "get audit logs error".to_string()))?;

    Ok(logs)
}

pub async fn check_alarm(
    db: &mut AsyncPgConnection,
    app: &Application,
    device: &Device,
    object_json: &Value,
) -> Result<()> {
    // Skip inactive devices
    if let Some(status) = device.tags.get("status") {
        if status != "active" {
            tracing::info!(dev_eui = %device.dev_eui, "Device status is not active, skipping alarm check");
            return Ok(());
        }
    }

    let current_time = Local::now().naive_local();
    let weekday = current_time.weekday().number_from_monday();

    let alarms: Vec<AlarmWithDates> =
        get_active_alarms_with_schedule(db, &device.dev_eui.to_string(), weekday as i32)
            .await
            .context("Fetching alarms")?;

    let _zone_name = get_zone_name_by_dev_eui(db, &device.dev_eui.to_string())
        .await
        .unwrap_or_else(|_| None)
        .unwrap_or_else(|| "Bilinmeyen Alan".to_string());

    for alarm in alarms {
        if !alarm.is_active || !alarm.is_within_schedule(current_time.time()) {
            continue;
        }

        match device.device_type {
            Some(1) => {
                if alarm.temperature {
                    if let Some(Value::String(temp)) = object_json.get("temperature") {
                        if let Ok(t) = temp.parse::<f32>() {
                            let calibration = device
                                .temperature_calibration
                                .as_ref()
                                .and_then(|v| v.to_f32())
                                .unwrap_or(0.0);
                            let value = t + calibration;
                            check_threshold(
                                &alarm,
                                value,
                                device,
                                "temperature",
                                &current_time.to_string(),
                                db,
                            )
                            .await?;
                        }
                    }
                } else if alarm.humadity {
                    if let Some(Value::String(hum)) = object_json.get("humidity") {
                        if let Ok(h) = hum.parse::<f32>() {
                            let calibration = device
                                .humadity_calibration
                                .as_ref()
                                .and_then(|v| v.to_f32())
                                .unwrap_or(0.0);
                            let value = h + calibration;
                            check_threshold(
                                &alarm,
                                value,
                                device,
                                "humidity",
                                &current_time.to_string(),
                                db,
                            )
                            .await?;
                        }
                    }
                }
            }
            Some(2) => {
                if let (Some(Value::String(temp)), Some(Value::String(water))) = (
                    object_json.get("temperature_soil"),
                    object_json.get("water_soil"),
                ) {
                    if temp != "0.00" && water != "0.00" {
                        if alarm.temperature {
                            if let Ok(v) = temp.parse::<f32>() {
                                check_threshold(
                                    &alarm,
                                    v,
                                    device,
                                    "temperature",
                                    &current_time.to_string(),
                                    db,
                                )
                                .await?;
                            }
                        } else if alarm.humadity {
                            if let Ok(v) = water.parse::<f32>() {
                                check_threshold(
                                    &alarm,
                                    v,
                                    device,
                                    "humidity",
                                    &current_time.to_string(),
                                    db,
                                )
                                .await?;
                            }
                        } else if alarm.ec {
                            if let Some(Value::Number(ec)) = object_json.get("conduct_soil") {
                                check_threshold(
                                    &alarm,
                                    ec.as_f64().unwrap_or(0.0) as f32,
                                    device,
                                    "ec",
                                    &current_time.to_string(),
                                    db,
                                )
                                .await?;
                            }
                        }
                    }
                }
            }
            Some(3) | Some(16) => {
                if alarm.door {
                    if let Some(Value::Number(status)) = object_json.get("door_status") {
                        if status.as_i64() == Some(1) {
                            execute_alarm(
                                &alarm,
                                0.0,
                                device,
                                "door",
                                &current_time.to_string(),
                                db,
                            )
                            .await?;
                        }
                    }
                }
            }
            Some(4) | Some(10) | Some(14) | Some(18) | Some(19) => {
                if alarm.w_leak {
                    if let Some(Value::Number(leak)) = object_json
                        .get("water_status")
                        .or_else(|| object_json.get("water_leek"))
                    {
                        if leak.as_i64() == Some(1) {
                            execute_alarm(
                                &alarm,
                                0.0,
                                device,
                                "water_leak",
                                &current_time.to_string(),
                                db,
                            )
                            .await?;
                        }
                    }
                }
            }
            Some(12) | Some(35) => {
                if alarm.temperature {
                    if let Some(Value::Number(temp)) = object_json.get("temperature") {
                        let calibration = device
                            .temperature_calibration
                            .as_ref()
                            .and_then(|v| v.to_f32())
                            .unwrap_or(0.0);
                        let value = temp.as_f64().unwrap_or(0.0) as f32 + calibration;
                        check_threshold(
                            &alarm,
                            value,
                            device,
                            "temperature",
                            &current_time.to_string(),
                            db,
                        )
                        .await?;
                    }
                }
                if alarm.humadity {
                    if let Some(Value::Number(hum)) = object_json.get("humidity") {
                        let base_value = hum.as_f64().unwrap_or(0.0) as f32;
                        let calibration = device
                            .humadity_calibration
                            .as_ref()
                            .and_then(|v| v.to_f32())
                            .unwrap_or(0.0);
                        let value = base_value + calibration;

                        check_threshold(
                            &alarm,
                            value,
                            device,
                            "humidity",
                            &current_time.to_string(),
                            db,
                        )
                        .await?;
                    }
                }

                if alarm.co2 {
                    if let Some(Value::Number(co2)) = object_json.get("co2") {
                        check_threshold(
                            &alarm,
                            co2.as_f64().unwrap_or(0.0) as f32,
                            device,
                            "co2",
                            &current_time.to_string(),
                            db,
                        )
                        .await?;
                    }
                }
            }
            Some(20) => {
                if alarm.temperature {
                    if let Some(Value::Number(temp)) = object_json.get("temperature") {
                        check_threshold(
                            &alarm,
                            temp.as_f64().unwrap_or(0.0) as f32,
                            device,
                            "temperature",
                            &current_time.to_string(),
                            db,
                        )
                        .await?;
                    }
                }
            }
            Some(21) => {
                if alarm.pressure {
                    if let Some(Value::Number(p)) = object_json.get("pressure") {
                        check_threshold(
                            &alarm,
                            (p.as_f64().unwrap_or(0.0) / 100.0) as f32,
                            device,
                            "pressure",
                            &current_time.to_string(),
                            db,
                        )
                        .await?;
                    }
                }
            }
            Some(33) => {
                if alarm.distance {
                    if let Some(Value::Number(d)) = object_json.get("distance") {
                        check_threshold(
                            &alarm,
                            (d.as_f64().unwrap_or(0.0) / 1000.0) as f32,
                            device,
                            "distance",
                            &current_time.to_string(),
                            db,
                        )
                        .await?;
                    }
                }
            }
            Some(36) => {
                if alarm.temperature {
                    let t = match object_json.get("temperature1") {
                        Some(Value::Number(t1)) if t1.as_f64().unwrap_or(-999.0) > -200.0 => {
                            t1.as_f64().unwrap_or(0.0)
                        }
                        _ => object_json
                            .get("temperature2")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0),
                    } as f32;
                    check_threshold(
                        &alarm,
                        t,
                        device,
                        "temperature",
                        &current_time.to_string(),
                        db,
                    )
                    .await?;
                }
            }
            Some(40) => {
                if alarm.temperature {
                    if let Some(Value::Number(t)) = object_json.get("temperature") {
                        check_threshold(
                            &alarm,
                            t.as_f64().unwrap_or(0.0) as f32,
                            device,
                            "temperature",
                            &current_time.to_string(),
                            db,
                        )
                        .await?;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub async fn get_active_alarms_with_schedule(
    conn: &mut AsyncPgConnection,
    dev_eui: &str,
    weekday: i32,
) -> anyhow::Result<Vec<AlarmWithDates>> {
    let query = r#"
        SELECT 
            alrm.*, 
            alrmDate.alarm_day AS alarm_day,
            alrmDate.start_time AS alarm_start_time,
            alrmDate.end_time AS alarm_end_time
        FROM alarm AS alrm 
        INNER JOIN alarm_date_time alrmDate ON alrm.id = alrmDate.alarm_id 
        WHERE dev_eui = $1 
          AND (alrmDate.alarm_day = 0 OR alrmDate.alarm_day = $2) 
          AND is_active = true
    "#;

    let results = diesel::sql_query(query)
        .bind::<Text, _>(dev_eui)
        .bind::<Int4, _>(weekday)
        .load::<AlarmWithDates>(conn)
        .await?;

    Ok(results)
}

pub async fn get_zone_name_by_dev_eui(
    conn: &mut AsyncPgConnection,
    dev_eui: &str,
) -> anyhow::Result<Option<String>> {
    let query = r#"
        SELECT zone_name 
        FROM zone 
        WHERE '\\x' || $1 = ANY(zone.devices)
    "#;
    #[derive(QueryableByName)]
    struct ZoneName {
        #[diesel(sql_type = Text)]
        zone_name: String,
    }
    let rows: Vec<ZoneName> = sql_query(query).bind::<Text, _>(dev_eui).load(conn).await?;

    Ok(rows.into_iter().map(|row| row.zone_name).next())
}

pub async fn check_threshold(
    alarm: &AlarmWithDates,
    value: f32,
    device: &Device,
    alarm_type: &str,
    date: &str,
    conn: &mut AsyncPgConnection,
) -> anyhow::Result<()> {
    if value < alarm.min_treshold || value > alarm.max_treshold {
        match alarm.zone_category_id {
            1 => {
                let ege = ege_method(value, alarm, conn).await;
                match ege {
                    Ok(true) => {
                        execute_alarm2(alarm, value, device, alarm_type, date, conn).await?;
                        return Ok(());
                    }
                    Ok(false) => {}
                    Err(e) => {
                        tracing::error!("ege_method error: {:?}", e);
                        return Err(anyhow::anyhow!("ege method error: {}", e));
                    }
                }
            }
            0 | 2 => {
                execute_alarm(alarm, value, device, alarm_type, date, conn).await?;
                return Ok(());
            }
            _ => {
                execute_alarm(alarm, value, device, alarm_type, date, conn).await?;
                return Ok(());
            }
        }
    }
    Ok(())
}

pub async fn execute_alarm(
    alarm: &AlarmWithDates,
    value: f32,
    device: &Device,
    alarm_type: &str,
    date: &str,
    conn: &mut AsyncPgConnection,
) -> anyhow::Result<()> {
    let zone_name = get_zone_name_by_dev_eui(conn, &device.dev_eui.to_string())
        .await
        .unwrap_or(Some("Bilinmeyen Alan".to_string()));

    let mut message = String::new();
    match alarm_type {
        "ısı" | "nem" | "basinc" | "co2" => {
            write!(
                &mut message,
                "{} tarihinde {} ortamındaki {} isimli sensör {} kritik alarm seviyesini gecti. Şu anki değeri: {:.2}",
                date,
                zone_name.as_deref().unwrap_or("Bilinmeyen Alan"),
                device.name,
                alarm_type,
                value
            )?;
        }
        "acil durum" => {
            write!(
                &mut message,
                "{} deki {} sensöründe acil durum var",
                zone_name.as_deref().unwrap_or("Bilinmeyen Alan"),
                device.name
            )?;
        }
        "kacak" => {
            write!(
                &mut message,
                "{} deki {} sensöründe su baskını alarmı var",
                zone_name.as_deref().unwrap_or("Bilinmeyen Alan"),
                device.name
            )?;
        }
        "button" => {
            write!(
                &mut message,
                "{} deki {} sensöründen çağrı var",
                zone_name.as_deref().unwrap_or("Bilinmeyen Alan"),
                device.name
            )?;
        }
        "door" => {
            write!(
                &mut message,
                "{} tarihinde {} ortamındaki {} isimli sensör açıldı",
                date,
                zone_name.as_deref().unwrap_or("Bilinmeyen Alan"),
                device.name
            )?;
        }
        "mesafe" => {
            write!(
                &mut message,
                "{} tarihinde {} ortamındaki {} isimli sensör mesafe limitini aştı: {:.2}",
                date,
                zone_name.as_deref().unwrap_or("Bilinmeyen Alan"),
                device.name,
                value
            )?;
        }
        _ => {}
    }

    let notification = notification::Notification {
        sender_id: alarm.id as i32,
        receiver_id: alarm.user_id.clone(),
        message: message.clone(),
        category_id: 1,
        is_read: Some(false),
        send_time: Some(Local::now().naive_local()),
        sender_ip: Some("System".to_string()),
        reader_ip: Some("".to_string()),
        is_deleted: Some(false),
        device_name: Some(device.name.clone()),
        dev_eui: Some(device.dev_eui.to_string()),
        deleted_time: None,
        id: 0,
        read_time: None,
    };

    notification::create_notification(notification).await?;
    Ok(())
}

pub async fn execute_alarm2(
    alarm: &AlarmWithDates,
    value: f32,
    device: &Device,
    alarm_type: &str,
    date: &str,
    conn: &mut AsyncPgConnection,
) -> anyhow::Result<()> {
    // Get zone name
    let zone_name = get_zone_name_by_dev_eui(conn, &device.dev_eui.to_string())
        .await
        .unwrap_or(Some("Bilinmeyen Alan".to_string()));

    // Get organization name
    #[derive(QueryableByName)]
    struct OrganizationRow {
        #[diesel(sql_type = Text)]
        name: String,
    }
    let organization_name: String = if let Some(org_id) = device.tenant_id {
        let org_query = r#"SELECT name FROM public.tenant WHERE id = $1"#;
        let org_rows: Vec<OrganizationRow> = sql_query(org_query)
            .bind::<Nullable<DieselUuid>, _>(device.tenant_id)
            .load(conn)
            .await
            .unwrap_or_default();

        org_rows
            .get(0)
            .map(|o| o.name.clone())
            .unwrap_or_else(|| "Bilinmeyen Organizasyon".to_string())
    } else {
        "Bilinmeyen Organizasyon".to_string()
    };
    // Compose message
    let message = match alarm_type {
        "ısı" | "nem" | "basinc" | "co2" => format!(
            "{} - {} tarihinde {} ortamındaki {} isimli sensör {} kritik alarm seviyesini gecti. şu an ki değeri: {:.2}",
            organization_name,
            date,
            zone_name.as_deref().unwrap_or("Bilinmeyen Alan"),
            device.name,
            alarm_type,
            value
        ),
        "acil durum" => format!(
            "{} - {} deki {} sensöründe acil durum var",
            organization_name,
            zone_name.as_deref().unwrap_or("Bilinmeyen Alan"),
            device.name
        ),
        "kacak" => format!(
            "{} - {} deki {} sensöründe su baskını alarmı var",
            organization_name,
            zone_name.as_deref().unwrap_or("Bilinmeyen Alan"),
            device.name
        ),
        "button" => format!(
            "{} - {} deki {} sensöründen çağrı var",
            organization_name,
            zone_name.as_deref().unwrap_or("Bilinmeyen Alan"),
            device.name
        ),
        "door" => format!(
            "{} - {} tarihinde {} ortamındaki {} isimli sensör açıldı",
            organization_name,
            date,
            zone_name.as_deref().unwrap_or("Bilinmeyen Alan"),
            device.name
        ),
        "mesafe" => format!(
            "{} - {} tarihinde {} ortamındaki {} isimli sensör mesafe limitini aştı: {:.2}",
            organization_name,
            date,
            zone_name.as_deref().unwrap_or("Bilinmeyen Alan"),
            device.name,
            value
        ),
        _ => String::new(),
    };

    let notification = notification::Notification {
        sender_id: alarm.id as i32,
        receiver_id: alarm.user_id.clone(),
        message,
        category_id: 1,
        is_read: Some(false),
        send_time: Some(Local::now().naive_local()),
        sender_ip: Some("system".to_string()),
        reader_ip: Some("".to_string()),
        is_deleted: Some(false),
        device_name: Some(device.name.clone()),
        dev_eui: Some(device.dev_eui.to_string()),
        deleted_time: None,
        id: 0,
        read_time: None,
    };

    notification::create_notification(notification).await?;
    Ok(())
}

#[derive(QueryableByName)]
struct TemperatureRow {
    #[diesel(sql_type = Float4)]
    air_temperature: f32,
}

pub async fn ege_method(
    value: f32,
    alarm: &AlarmWithDates,
    conn: &mut AsyncPgConnection,
) -> anyhow::Result<bool> {
    let interval = alarm.defrost_time;

    // First: get values from the last `interval` minutes
    let query = format!(
        "SELECT air_temperature FROM device_data_2025 \
         WHERE dev_eui = $1 AND submission_date > now() - interval '{} minute' \
         ORDER BY submission_date ASC",
        interval
    );

    let mut res_45_min: Vec<f32> = sql_query(&query)
        .bind::<diesel::sql_types::Text, _>(&alarm.dev_eui)
        .load::<TemperatureRow>(conn)
        .await?
        .into_iter()
        .map(|row| row.air_temperature)
        .collect();

    // If empty, fallback to most recent value
    if res_45_min.is_empty() {
        res_45_min = sql_query(
            "SELECT air_temperature FROM device_data_2025 \
             WHERE dev_eui = $1 ORDER BY submission_date DESC LIMIT 1",
        )
        .bind::<diesel::sql_types::Text, _>(&alarm.dev_eui)
        .load::<TemperatureRow>(conn)
        .await?
        .into_iter()
        .map(|row| row.air_temperature)
        .collect();
    }

    // No data found at all
    if res_45_min.is_empty() {
        tracing::warn!(dev_eui = %alarm.dev_eui, "No temperature data found for ege_method");
        return Ok(false);
    }

    // Threshold check
    if res_45_min.iter().any(|&t| t < alarm.max_treshold) {
        return Ok(false);
    }

    // Trend analysis
    if res_45_min.len() > 1 {
        let last = res_45_min[res_45_min.len() - 1];
        let previous = res_45_min[res_45_min.len() - 2];
        if last <= previous {
            return Ok(false);
        }
    }

    Ok(true)
}

pub async fn create_alarm_automation(alarm_automation: NewAlarmAutomation) -> Result<(), Error> {
    let mut conn = get_async_db_conn().await?;

    let new_alarm_automation: AlarmAutomation = diesel::insert_into(alarm_automation_rules::table)
        .values(&alarm_automation)
        .get_result(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, "Creating alarm automation".to_string()))?;

    info!(id = %new_alarm_automation.id, "Alarm automation created");

    Ok(())
}

pub async fn get_alarm_automation(id: i32) -> Result<AlarmAutomation, Error> {
    let mut conn = get_async_db_conn().await?;

    let alarm_automation: AlarmAutomation = alarm_automation_rules::table
        .find(id)
        .first(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, id.to_string()))?;

    Ok(alarm_automation)
}

pub async fn list_alarm_automation(alarm_id: i32) -> Result<Vec<AlarmAutomation>, Error> {
    let mut conn = get_async_db_conn().await?;

    let alarm_automations: Vec<AlarmAutomation> = alarm_automation_rules::table
        .filter(alarm_automation_rules::alarm_id.eq(alarm_id))
        .filter(alarm_automation_rules::is_active.eq(Some(true)))
        .load(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, alarm_id.to_string()))?;

    Ok(alarm_automations)
}

pub async fn delete_alarm_automation(id: i32) -> Result<(), Error> {
    let mut conn = get_async_db_conn().await?;

    let affected = diesel::delete(alarm_automation_rules::table.find(id))
        .execute(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, id.to_string()))?;

    if affected == 0 {
        return Err(Error::NotFound("Alarm automation not found".into()));
    }

    info!(id, affected_rows = affected, "Alarm automation deleted");
    Ok(())
}

pub async fn update_alarm_automation(
    updated_alarm_automation: UpdateAlarmAutomation,
) -> Result<AlarmAutomation, Error> {
    let mut conn = get_async_db_conn().await?;

    let updated_alarm_automation: AlarmAutomation =
        diesel::update(alarm_automation_rules::table.find(updated_alarm_automation.id))
            .set(&updated_alarm_automation)
            .get_result(&mut conn)
            .await
            .map_err(|e| Error::from_diesel(e, updated_alarm_automation.id.to_string()))?;

    info!(updated_alarm_automation.id, "Alarm automation updated");
    Ok(updated_alarm_automation)
}
