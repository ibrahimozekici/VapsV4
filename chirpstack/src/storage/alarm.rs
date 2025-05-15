use super::schema::{
    alarm, alarm_audit_log, alarm_automation_rules, alarm_date_time, automation_rules,
    door_time_alarm,
};
use super::{db_transaction, error::Error, fields, get_async_db_conn};
use anyhow::Result;
use chirpstack_api::{api, common, internal};
use chrono::NaiveDateTime;
use diesel::dsl::now;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::*;
use diesel::Connection;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

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
    pub user_id: Option<Vec<Option<i64>>>,
    pub distance: Option<bool>,
    pub defrost_time: Option<i32>,
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
            user_id: Some(vec![None]),
            distance: None,
            defrost_time: Some(0),
        }
    }
}
#[derive(Debug, QueryableByName, Serialize, Deserialize)]
pub struct OrganizationAlarm {
    #[diesel(sql_type = Int8)]
    pub id: i64,

    #[diesel(sql_type = Text)]
    pub dev_eui: String,

    #[diesel(sql_type = Nullable<Float4>)]
    pub min_treshold: Option<f32>,

    #[diesel(sql_type = Nullable<Float4>)]
    pub max_treshold: Option<f32>,

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

    #[diesel(sql_type = Nullable<Array<Nullable<Int8>>>)]
    pub user_id: Option<Vec<Option<i64>>>,

    #[diesel(sql_type = Nullable<Text>)]
    pub ip_address: Option<String>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub is_time_limit_active: Option<bool>,

    #[diesel(sql_type = Nullable<Float4>)]
    pub alarm_start_time: Option<f32>,

    #[diesel(sql_type = Nullable<Float4>)]
    pub alarm_stop_time: Option<f32>,

    #[diesel(sql_type = Nullable<Int8>)]
    pub zone_category: Option<i64>,

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

    #[diesel(sql_type = Nullable<Float4>)]
    pub current: Option<f32>,

    #[diesel(sql_type = Nullable<Float4>)]
    pub factor: Option<f32>,

    #[diesel(sql_type = Nullable<Float4>)]
    pub power: Option<f32>,

    #[diesel(sql_type = Nullable<Float4>)]
    pub voltage: Option<f32>,

    #[diesel(sql_type = Nullable<Int8>)]
    pub status: Option<i64>,

    #[diesel(sql_type = Nullable<Float4>)]
    pub power_sum: Option<f32>,

    #[diesel(sql_type = Nullable<Text>)]
    pub notification_sound: Option<String>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub distance: Option<bool>,

    #[diesel(sql_type = Nullable<Int8>)]
    pub time: Option<i64>,

    #[diesel(sql_type = Nullable<Int8>)]
    pub defrost_time: Option<i64>,
}

#[derive(Queryable, QueryableByName, Insertable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = alarm_audit_log)]
pub struct AlarmAuditLog {
    pub id: i32,
    pub alarm_id: i32,
    pub dev_eui: Option<String>,
    pub change_type: Option<String>,
    pub changed_at: Option<NaiveDateTime>,
    pub changed_by: Option<i32>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
}

#[derive(Queryable, Insertable, PartialEq, Debug, Clone)]
#[diesel(table_name = alarm_date_time)]
pub struct AlarmDateTime {
    pub alarm_id: i32,
    pub alarm_day: i32,
    pub start_time: f64,
    pub end_time: f64,
    pub id: i32,
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

    #[diesel(sql_type = Nullable<Text>)]
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

    #[diesel(sql_type = Nullable<BigInt>)]
    pub time: Option<i64>,

    #[diesel(sql_type = Nullable<Array<Nullable<BigInt>>>) ]
    pub user_id: Option<Vec<Option<i64>>>,

    #[diesel(sql_type = Nullable<Integer>)]
    pub organization_id: Option<i32>,

    #[diesel(sql_type = Nullable<Bool>)]
    pub is_time_limit_active: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AlarmFilters {
    pub limit: i32,
    pub dev_eui: String,
    pub user_id: i64,
}

pub async fn create(
    alarm: Alarm,
    date_filters: Vec<AlarmDateTime>,
    sent_user_id: i64,
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

pub async fn get_organization_alarm_list(
    organization_id: i32,
) -> Result<Vec<OrganizationAlarm>, Error> {
    let mut conn = get_async_db_conn().await?;

    let alarms = diesel::sql_query(
        r#"
        SELECT z.zone_name, d.name AS device_name, 0 AS time, a.*
        FROM alarm AS a
        INNER JOIN device AS d ON d.dev_eui::text = '\x' || a.dev_eui
        INNER JOIN zone AS z ON d.dev_eui::text = ANY(z.devices)
        WHERE d.organization_id = $1
        "#,
    )
    .bind::<Int4, _>(organization_id)
    .load::<OrganizationAlarm>(&mut conn)
    .await
    .map_err(|e| Error::from_diesel(e, organization_id.to_string()))?;

    info!(organization_id = organization_id, "Alarms fetched");
    Ok(alarms)
}
pub async fn update_alarm(
    updated_alarm: Alarm,
    date_filters: Vec<AlarmDateTime>,
    sent_user_id: i64,
) -> Result<(), Error> {
    let mut conn = get_async_db_conn().await?;

    let existing_alarm: Alarm = alarm::table
        .filter(alarm::id.eq(updated_alarm.id))
        .first(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, updated_alarm.dev_eui.to_string()))?;

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

    diesel::update(alarm::table.filter(alarm::id.eq(updated_alarm.id)))
        .set(&updated_alarm)
        .execute(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, updated_alarm.dev_eui.to_string()))?;

    diesel::delete(alarm_date_time::table.filter(alarm_date_time::alarm_id.eq(updated_alarm.id)))
        .execute(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, updated_alarm.dev_eui.to_string()))?;

    for df in &date_filters {
        let new_df = AlarmDateTime {
            id: df.id,
            alarm_id: updated_alarm.id,
            alarm_day: df.alarm_day,
            start_time: df.start_time,
            end_time: df.end_time,
        };

        diesel::insert_into(alarm_date_time::table)
            .values(&new_df)
            .execute(&mut conn)
            .await
            .map_err(|e| Error::from_diesel(e, updated_alarm.dev_eui.to_string()))?;
    }

    info!(dev_eui = %updated_alarm.dev_eui, "Alarm updated");
    Ok(())
}

pub async fn delete_alarm(alarm_id: i32, sent_user_id: i64) -> Result<(), Error> {
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

pub async fn delete_user_alarm(user_id: i64, sent_user_id: i64) -> Result<(), Error> {
    let mut conn = get_async_db_conn().await?;

    let query = r#"
    WITH updated_rows AS (
        UPDATE alarm
        SET user_id = array_remove(user_id, $1::bigint)
        WHERE $1 = ANY(user_id)
        RETURNING *
    )
    SELECT * FROM updated_rows;
    "#;

    let alarms: Vec<Alarm> = sql_query(query)
        .bind::<BigInt, _>(user_id)
        .load(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, user_id.to_string()))?;

    for alarm in &alarms {
        if let Some(user_ids) = &alarm.user_id {
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

    info!(user_id = user_id, "User alarm deleted");
    Ok(())
}

pub async fn delete_sensor_alarm(dev_eui: &str, sent_user_id: i64) -> Result<(), Error> {
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

pub async fn delete_zone_alarm(zone_id: i32, sent_user_id: i64) -> Result<(), Error> {
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
    user_id: i64,
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
            alarm_audit_log::changed_by.eq(Some(user_id as i32)),
            alarm_audit_log::old_values.eq(previous_value),
            alarm_audit_log::new_values.eq(new_value),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, alarm_id.to_string()))?;

    info!(alarm_id, "Audit log created");
    Ok(())
}

pub async fn get_alarm_audit_logs(alarm_id: i64, limit: i64) -> Result<Vec<api::AuditLog>, Error> {
    let mut conn = get_async_db_conn().await?;

    let logs = diesel::sql_query(
        r#"
        SELECT * FROM alarm_audit_log
        WHERE alarm_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind::<Int8, _>(alarm_id)
    .bind::<Int8, _>(limit)
    .load::<AlarmAuditLog>(&mut conn)
    .await
    .map_err(|e| Error::from_diesel(e, alarm_id.to_string()))?
    .into_iter()
    .map(|log| api::AuditLog {
        log_id: log.id as i64,
        alarm_id: log.alarm_id as i64,
        dev_eui: log.dev_eui.unwrap_or_default(),
        change_type: log.change_type.unwrap_or_default(),
        changed_at: log.changed_at.map(|dt| ::prost_types::Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }),
        changed_by: log.changed_by.map(|v| v as i64).unwrap_or_default(),
        old_values: log
            .old_values
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string()),
        new_values: log
            .new_values
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string()),
    })
    .collect();

    info!(alarm_id, "Alarm audit logs fetched");
    Ok(logs)
}

pub async fn create_door_time_alarm(
    door_time_alarm: DoorTimeAlarm,
    time_schedule: Vec<AlarmDateTime>,
    sent_user_id: i64,
) -> Result<DoorTimeAlarm, Error> {
    let mut conn = get_async_db_conn().await?;

    // Insert into door_time_alarm table and get the created alarm with id
    let insert_query = r#"
        INSERT INTO door_time_alarm (
            dev_eui, time, is_active, sms, notification, email, user_id, submission_time, organization_id, is_time_limit_active
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, NOW(), $8, $9
        )
        RETURNING id, dev_eui, time, is_active, sms, notification, email, user_id, submission_time, organization_id, is_time_limit_active
    "#;

    let created_alarm: DoorTimeAlarm = sql_query(insert_query)
        .bind::<Nullable<Text>, _>(&door_time_alarm.dev_eui)
        .bind::<Nullable<Int8>, _>(door_time_alarm.time)
        .bind::<Nullable<Bool>, _>(door_time_alarm.is_active)
        .bind::<Nullable<Bool>, _>(door_time_alarm.sms)
        .bind::<Nullable<Bool>, _>(door_time_alarm.notification)
        .bind::<Nullable<Bool>, _>(door_time_alarm.email)
        .bind::<Nullable<Array<Nullable<Int8>>>, _>(&door_time_alarm.user_id)
        .bind::<Nullable<Int4>, _>(door_time_alarm.organization_id)
        .bind::<Nullable<Bool>, _>(door_time_alarm.is_time_limit_active)
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
               user_id, submission_time, organization_id, is_time_limit_active
        FROM door_time_alarm WHERE id = $1
    "#;

    let door_alarm: DoorTimeAlarm = sql_query(select_sql)
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

pub async fn list_door_time_alarms(dev_eui: String) -> Result<Vec<api::CreateDoorTimeResponse>, Error> {
    let mut conn = get_async_db_conn().await?;

    let query = r#"
        SELECT id, dev_eui, time, is_active, sms, notification, email, user_id
        FROM door_time_alarm
        WHERE $1 = '' OR dev_eui = $1
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
                .flatten()
                .collect(),
            ..Default::default()
        })
        .collect();

    Ok(result)
}

pub async fn get_audit_logs(dev_eui: &str) -> Result<Vec<AlarmAuditLog>, Error> {
    let mut conn = get_async_db_conn().await?;

    let logs = diesel::sql_query(
        r#"
        SELECT * FROM alarm_audit_log
        WHERE dev_eui = $1
        "#,
    )
    .bind::<Text, _>(dev_eui)
    .load::<AlarmAuditLog>(&mut conn)
    .await
    .map_err(|e| Error::from_diesel(e, dev_eui.to_string()))?;

    info!(dev_eui, "Alarm audit logs fetched by dev_eui");
    Ok(logs)
}
