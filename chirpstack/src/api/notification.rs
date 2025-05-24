use std::str::FromStr;

use super::{
    auth::{validator, AuthID},
    error::ToStatus,
};
use crate::storage::notification::{self};
use chirpstack_api::api::{self, notification_service_server::NotificationService};
use tonic::{Request, Response, Status};
use uuid::Uuid;
pub struct NotificationServiceImpl {
    validator: validator::RequestValidator,
}

impl NotificationServiceImpl {
    pub fn new(validator: validator::RequestValidator) -> Self {
        NotificationServiceImpl { validator }
    }
}

#[tonic::async_trait]
impl NotificationService for NotificationServiceImpl {
    async fn list(
        &self,
        request: Request<api::ListNotificationsRequest>,
    ) -> Result<Response<api::ListNotificationsResponse>, Status> {
        let req = request.get_ref();
        println!("🌀 Starting List Notification for org: {}", req.user_id);

        let user_id = Uuid::from_str(&req.user_id).map_err(|e| e.status())?;
        let notifications = notification::list(user_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to list notifications: {}", e)))?;

        let notifications_proto: Vec<api::Notification> = notifications
            .into_iter()
            .map(|n: crate::storage::notification::Notification| api::Notification::from(n))
            .collect();

        let resp = api::ListNotificationsResponse {
            notifications: notifications_proto,
        };

        Ok(Response::new(resp))
    }

    async fn update(
        &self,
        _request: Request<api::UpdateNotficationRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("update not implemented yet"))
    }

    async fn delete(
        &self,
        request: Request<api::DeleteNotficationRequest>,
    ) -> Result<Response<()>, Status> {
        let notification_id = request.get_ref().id;

        // Convert i64 -> i32 (assuming i32 DB type)
        let zone_id_i32 = i32::try_from(notification_id).map_err(|_| {
            Status::invalid_argument(format!(
                "zone_id {} is out of range for i32",
                notification_id
            ))
        })?;

        let deleted = notification::delete(zone_id_i32)
            .await
            .map_err(|e| e.status())?;

        if deleted == 0 {
            return Err(Status::not_found("Zone not found"));
        }
        let mut resp = Response::new(());
        // resp.metadata_mut()
        //     .insert("x-log-dev_eui", zone_id);

        Ok(resp)
    }
}

impl From<crate::storage::notification::Notification> for api::Notification {
    fn from(n: crate::storage::notification::Notification) -> Self {
        api::Notification {
            id: n.id as i64,
            message: n.message,
            is_read: n.is_read.unwrap_or(false),
            ..Default::default()
        }
    }
}
