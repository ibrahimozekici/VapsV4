use crate::storage::schema_postgres::notifications::dsl as notif_dsl;
use crate::storage::schema_postgres::notifications;
use chrono::NaiveDateTime;
use diesel::{prelude::*};
use diesel::dsl::sql;
use diesel::sql_types::{Uuid as SqlUuid, Bool};

use diesel_async::RunQueryDsl;
use tracing::info;
use uuid::Uuid;
use super::{error::Error, get_async_db_conn};


#[derive(Debug, Clone, PartialEq, Eq, Insertable, Queryable)]
#[diesel(table_name = crate::storage::schema_postgres::notifications)]
pub struct Notification {
    pub id: i32,
    pub sender_id: i32,
    pub message: String,
    pub category_id: i32,
    pub is_read: Option<bool>,
    pub send_time: Option<NaiveDateTime>,
    pub read_time: Option<NaiveDateTime>,
    pub sender_ip: Option<String>,
    pub reader_ip: Option<String>,
    pub is_deleted: Option<bool>,
    pub deleted_time: Option<NaiveDateTime>,
    pub dev_eui: Option<String>,
    pub device_name: Option<String>,
    pub receiver_id: Vec<Option<Uuid>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::storage::schema_postgres::notifications)]
pub struct NewNotification {
    pub sender_id: i32,
    pub message: String,
    pub category_id: i32,
    pub is_read: Option<bool>,
    pub send_time: Option<NaiveDateTime>,
    pub read_time: Option<NaiveDateTime>,
    pub sender_ip: Option<String>,
    pub reader_ip: Option<String>,
    pub is_deleted: Option<bool>,
    pub deleted_time: Option<NaiveDateTime>,
    pub dev_eui: Option<String>,
    pub device_name: Option<String>,
    pub receiver_id: Vec<Option<Uuid>>,
}

impl Notification {
    pub fn validate(&self) -> Result<(), Error> {
        if self.message.trim().is_empty() {
            return Err(Error::Validation("Message cannot be empty".into()));
        }
        if self.receiver_id.is_empty() {
            return Err(Error::Validation("Receiver list cannot be empty".into()));
        }
        Ok(())
    }
}

impl NewNotification {
    pub fn validate(&self) -> Result<(), Error> {
        if self.message.trim().is_empty() {
            return Err(Error::Validation("Message cannot be empty".into()));
        }
        if self.receiver_id.is_empty() {
            return Err(Error::Validation("Receiver list cannot be empty".into()));
        }
        Ok(())
    }
}

impl Default for NewNotification {
    fn default() -> Self {
        NewNotification {
            sender_id: 0,
            message: "".to_string(),
            category_id: 0,
            is_read: Some(false),
            send_time: Some(chrono::Utc::now().naive_utc()),
            read_time: None,
            sender_ip: Some("".to_string()),
            reader_ip: Some("".to_string()),
            is_deleted: Some(false),
            deleted_time: None,
            dev_eui: Some("".to_string()),
            device_name: Some("".to_string()),
            receiver_id: vec![],
        }
    }
}

pub async fn create_notification(notification: Notification) -> Result<Notification, Error> {
    notification.validate()?;

    let new = NewNotification {
        sender_id: notification.sender_id,
        message: notification.message.clone(),
        category_id: notification.category_id,
        is_read: notification.is_read,
        send_time: notification.send_time,
        read_time: notification.read_time,
        sender_ip: notification.sender_ip.clone(),
        reader_ip: notification.reader_ip.clone(),
        is_deleted: notification.is_deleted,
        deleted_time: notification.deleted_time,
        dev_eui: notification.dev_eui.clone(),
        device_name: notification.device_name.clone(),
        receiver_id: notification.receiver_id.clone(),
    };

    let conn = &mut get_async_db_conn().await?;

    let generated_id: i32 = diesel::insert_into(notif_dsl::notifications)
        .values(&new)
        .returning(notif_dsl::id)
        .get_result(conn)
        .await
        .map_err(|e| Error::from_diesel(e, "insert notification".into()))?;

    Ok(Notification {
        id: generated_id,
        ..notification
    })
}

pub async fn get_notification(notification_id: i32) -> Result<Notification, Error> {
    let result = notif_dsl::notifications
        .find(notification_id)
        .first(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, "get notification".into()))?;

    Ok(result)
}

pub async fn list(user_id: Uuid) -> Result<Vec<Notification>, Error> {
    let mut conn = get_async_db_conn().await?;

    let results = notif_dsl::notifications
        .filter(notif_dsl::receiver_id.contains(vec![Some(user_id)]))
        .filter(notif_dsl::is_deleted.eq(Some(false)))
        .order(notif_dsl::id.desc())
        .limit(500)
        .load::<Notification>(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, "list notifications".into()))?;

    Ok(results)
}

pub async fn delete(notification_id: i32) -> Result<usize, Error> {
    let deleted_rows = diesel::delete(notif_dsl::notifications.filter(notif_dsl::id.eq(notification_id)))
        .execute(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, format!("delete notification {notification_id}")))?;

    info!(id = %notification_id, rows = %deleted_rows, "Notification deleted");

    Ok(deleted_rows)
}
