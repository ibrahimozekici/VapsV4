use diesel::{prelude::*, sql_query};
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use crate::storage::schema_postgres::notifications;
use diesel_async::RunQueryDsl;
use crate::storage::schema_postgres::notifications::dsl::*;

use super::{error::Error, get_async_db_conn};

#[derive(Debug, Clone, PartialEq, Eq, Insertable, Queryable)]
#[diesel(table_name = notifications)]
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
    pub receiver_id: Vec<Option<i32>>,
}


#[derive(Debug, Insertable)]
#[diesel(table_name = notifications)]
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
    pub receiver_id: Vec<Option<i32>>,
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
        message: notification.message,
        category_id: notification.category_id,
        is_read: notification.is_read,
        send_time: notification.send_time,
        read_time: notification.read_time,
        sender_ip: notification.sender_ip,
        reader_ip: notification.reader_ip,
        is_deleted: notification.is_deleted,
        deleted_time: notification.deleted_time,
        dev_eui: notification.dev_eui,
        device_name: notification.device_name,
        receiver_id: notification.receiver_id,
    };


   let conn = &mut get_async_db_conn().await?;

    // ✅ .await on the async version
    let generated_id: i32 = diesel::insert_into(notifications::table)
        .values(&new)
        .returning(notifications::id)
        .get_result(conn) // this works with diesel_async::RunQueryDsl
        .await
        .map_err(|e| Error::from_diesel(e, "insert notification".into()))?;

    let mut inserted = notification;
    inserted.id = generated_id;

    Ok(inserted)
}

pub async fn get_notification(notification_id: i32) -> Result<Notification, Error> {
    
    let result = notifications::dsl::notifications
         .find(notification_id)
        .first(&mut get_async_db_conn().await?)
       .await
        .map_err(|e| Error::from_diesel(e, "get notification".into()))?;

    Ok(result)
}

pub async fn list_notifications(user_id: i32) -> Result<Vec<Notification>, Error> {
    let mut conn = get_async_db_conn().await?;

    let result = notifications
        .filter(diesel::dsl::any(receiver_id).eq(user_id)) // OR: receiver_id.any(user_id)
        .filter(is_deleted.eq(false))
        .order(id.desc())
        .limit(500)
        .load::<Notification>(&mut conn)
        .await
        .map_err(|e| Error::from_diesel(e, "list notifications".into()))?;

    Ok(result)
}
