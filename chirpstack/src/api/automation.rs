use super::auth::validator::{self};
use super::helpers::{self};
use crate::storage::automation::{self, AutomationFilters};
use chirpstack_api::api;
use chirpstack_api::api::automation_service_server::AutomationService;
use chrono::{TimeZone, Utc};
use tonic::{Request, Response, Status};
use prost_types::Timestamp;
use diesel::sql_types::Uuid as DieselUuid;
pub struct Automation {
    validator: validator::RequestValidator,
}

impl Automation {
    pub fn new(validator: validator::RequestValidator) -> Self {
        Automation { validator }
    }
}

#[tonic::async_trait]
impl AutomationService for Automation {
    async fn create(
        &self,
        request: tonic::Request<api::CreateAutomationRequest>,
    ) -> Result<tonic::Response<api::CreateAutomationResponse>, Status> {
        // self.validator.validate_request(request).await
        let req = request.get_ref().automation.as_ref();
        if req.is_none() {
            return Err(Status::invalid_argument("automation is required"));
        }
        let automation = req.unwrap();

        let mut automation = automation::Automation {
            id: 0,
            user_id: Some(uuid::Uuid::parse_str(&automation.user_id)
                .map_err(|_| Status::invalid_argument("invalid user_id UUID"))?),
            sender_sensor: Some(automation.sender_sensor.clone()),
            receiver_sensor: Some(automation.receiver_sensor.clone()),
            condition: Some(automation.condition.clone()),
            action: Some(automation.action.clone()),
            sender_device_type: Some(automation.sender_device_type as i32),
            receiver_device_type: Some(automation.receiver_device_type as i32),
            sender_device_name: Some(automation.sender_device_name.clone()),
            receiver_device_name: Some(automation.receiver_device_name.clone()),
            trigger_type: Some(automation.trigger_type.clone()),
            trigger_time: automation
                .trigger_time
                .as_ref()
                .map(|ts| chrono::NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32))
                .flatten(),
            tenant_id: if !automation.tenant_id.is_empty() {
                Some(uuid::Uuid::parse_str(&automation.tenant_id)
                    .map_err(|_| Status::invalid_argument("invalid tenant_id UUID"))?)
            } else {
                None
            },
            created_at: None,
            updated_at: None,
            is_active: Some(true),
        };

        let return_automation = automation::create_automation_rule(automation)
            .await
            .map_err(|e| Status::internal(format!("db error: {}", e)))?;
        // Convert the storage struct to the API response struct
        let mut automation_response = api::Automation {
            id: return_automation.id as i64,
            user_id: return_automation.user_id.as_ref().and_then(|id| uuid::Uuid::from_slice(id.as_ref()).ok()).map(|uuid| uuid.to_string()).unwrap_or_default(),
            sender_sensor: return_automation.sender_sensor.unwrap_or_default(),
            receiver_sensor: return_automation.receiver_sensor.unwrap_or_default(),
            condition: return_automation.condition.unwrap_or_default(),
            action: return_automation.action.unwrap_or_default(),
            sender_device_type: return_automation.sender_device_type.unwrap_or_default() as i64,
            receiver_device_type: return_automation.receiver_device_type.unwrap_or_default() as i64,
            sender_device_name: return_automation.sender_device_name.unwrap_or_default(),
            receiver_device_name: return_automation.receiver_device_name.unwrap_or_default(),
            trigger_type: return_automation.trigger_type.unwrap_or_default(),

            trigger_time: return_automation.trigger_time.map(|t| {
                let dt = Utc.from_utc_datetime(&t);
                prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos: dt.timestamp_subsec_nanos() as i32,
                }
            }),

            tenant_id: return_automation.tenant_id.map(|id| id.to_string()).unwrap_or_default(),

            created_at: return_automation.created_at.map(|t| {
                let dt = Utc.from_utc_datetime(&t);
                prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos: dt.timestamp_subsec_nanos() as i32,
                }
            }),

            updated_at: return_automation.updated_at.map(|t| {
                let dt = Utc.from_utc_datetime(&t);
                prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos: dt.timestamp_subsec_nanos() as i32,
                }
            }),
        };

        let response = api::CreateAutomationResponse {
            automation: Some(automation_response),
            ..Default::default()
        };
        Ok(tonic::Response::new(response))
    }

    async fn get(
        &self,
        request: tonic::Request<api::GetAutomationRequest>,
    ) -> Result<tonic::Response<api::GetAutomationResponse>, Status> {
        // self.validator.validate_request(request).await

        let id = request.get_ref().id;
        if id == 0 {
            return Err(Status::invalid_argument("id must be non-zero"));
        }

        let automation = automation::get_automation_rule(id as i32)
            .await
            .map_err(|e| Status::internal(format!("db error: {}", e)))?;

        let automation_response = api::Automation {
            id: automation.id as i64,
            user_id: automation.user_id.as_ref().and_then(|id| uuid::Uuid::from_slice(id.as_bytes()).ok()).map(|uuid| uuid.to_string()).unwrap_or_default(),
            sender_sensor: automation.sender_sensor.unwrap_or_default(),
            receiver_sensor: automation.receiver_sensor.unwrap_or_default(),
            condition: automation.condition.unwrap_or_default(),
            action: automation.action.unwrap_or_default(),
            sender_device_type: automation.sender_device_type.unwrap_or_default() as i64,
            receiver_device_type: automation.receiver_device_type.unwrap_or_default() as i64,
            sender_device_name: automation.sender_device_name.unwrap_or_default(),
            receiver_device_name: automation.receiver_device_name.unwrap_or_default(),
            trigger_type: automation.trigger_type.unwrap_or_default(),

            trigger_time: automation.trigger_time.map(|t| {
                let dt = Utc.from_utc_datetime(&t);
                prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos: dt.timestamp_subsec_nanos() as i32,
                }
            }),

            tenant_id: match &automation.tenant_id {
                Some(tid) => tid.to_string(),
                None => String::new(),
            },

            created_at: automation.created_at.map(|t| {
                let dt = Utc.from_utc_datetime(&t);
                prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos: dt.timestamp_subsec_nanos() as i32,
                }
            }),

            updated_at: automation.updated_at.map(|t| {
                let dt = Utc.from_utc_datetime(&t);
                prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos: dt.timestamp_subsec_nanos() as i32,
                }
            }),
        };

        let response = api::GetAutomationResponse {
            automation: Some(automation_response),
            ..Default::default()
        };

        Ok(tonic::Response::new(response))
    }

    async fn list(
        &self,
        request: Request<api::ListAutomationRequest>,
    ) -> Result<Response<api::ListAutomationResponse>, Status> {
        let req = request.get_ref();

        // Validate input
        if uuid::Uuid::parse_str(&req.user_id).is_err() {
            return Err(Status::invalid_argument("invalid user_id UUID"));
        }

        let filters = AutomationFilters {
            user_id: match uuid::Uuid::parse_str(&req.user_id) {
                Ok(uuid) => uuid,
                Err(_) => return Err(Status::invalid_argument("invalid user_id UUID")),
            },
            sender_sensor: Some(req.dev_eui.clone()),
            tenant_id: match uuid::Uuid::parse_str(&req.tenant_id) {
                Ok(uuid) => uuid,
                Err(_) => return Err(Status::invalid_argument("invalid tenant_id UUID")),
            },
        };

        let automations = automation::list_automation_rules(filters)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;

        let mut response = api::ListAutomationResponse {
            automations: vec![],
        };

        for auto in automations {
            response.automations.push(api::Automation {
                id: auto.id as i64,
                user_id: auto.user_id.as_ref().and_then(|id| uuid::Uuid::from_slice(id.as_ref()).ok()).map(|uuid| uuid.to_string()).unwrap_or_default(),
                sender_sensor: auto.sender_sensor.unwrap_or_default(),
                receiver_sensor: auto.receiver_sensor.unwrap_or_default(),
                condition: auto.condition.unwrap_or_default(),
                action: auto.action.unwrap_or_default(),
                sender_device_type: auto.sender_device_type.unwrap_or_default() as i64,
                receiver_device_type: auto.receiver_device_type.unwrap_or_default() as i64,
                sender_device_name: auto.sender_device_name.unwrap_or_default(),
                receiver_device_name: auto.receiver_device_name.unwrap_or_default(),
                trigger_type: auto.trigger_type.unwrap_or_default(),
                trigger_time: auto.trigger_time.map(|t| {
                    let dt = Utc.from_utc_datetime(&t);
                    Timestamp {
                        seconds: dt.timestamp(),
                        nanos: dt.timestamp_subsec_nanos() as i32,
                    }
                }),
                tenant_id: auto.tenant_id.as_ref().map(|id| id.to_string()).unwrap_or_default(),
                created_at: auto.created_at.map(|t| {
                    let dt = Utc.from_utc_datetime(&t);
                    Timestamp {
                        seconds: dt.timestamp(),
                        nanos: dt.timestamp_subsec_nanos() as i32,
                    }
                }),
                updated_at: auto.updated_at.map(|t| {
                    let dt = Utc.from_utc_datetime(&t);
                    Timestamp {
                        seconds: dt.timestamp(),
                        nanos: dt.timestamp_subsec_nanos() as i32,
                    }
                }),
            });
        }

        Ok(Response::new(response))
    }

    async fn update(
        &self,
        request: tonic::Request<api::UpdateAutomationRequest>,
    ) -> Result<tonic::Response<api::GetAutomationResponse>, Status> {
        // self.validator.validate_request(request).await
        let req = &request.get_ref().automation;
        if req.is_none() {
            return Err(Status::invalid_argument("automation is required"));
        }
        let automation = req.as_ref().unwrap();

        // Validate user_id as UUID
        let user_id = uuid::Uuid::parse_str(&automation.user_id)
            .map_err(|_| Status::invalid_argument("invalid user_id UUID"))?;

        // Prepare the automation struct for update
        let mut automation_update = automation::Automation {
            id: automation.id as i32,
            user_id: Some(user_id),
            sender_sensor: Some(automation.sender_sensor.clone()),
            receiver_sensor: Some(automation.receiver_sensor.clone()),
            condition: Some(automation.condition.clone()),
            action: Some(automation.action.clone()),
            sender_device_type: Some(automation.sender_device_type as i32),
            receiver_device_type: Some(automation.receiver_device_type as i32),
            sender_device_name: Some(automation.sender_device_name.clone()),
            receiver_device_name: Some(automation.receiver_device_name.clone()),
            trigger_type: Some(automation.trigger_type.clone()),
            trigger_time: automation
                .trigger_time
                .as_ref()
                .and_then(|ts| chrono::NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32)),
            tenant_id: if !automation.tenant_id.is_empty() {
                Some(uuid::Uuid::parse_str(&automation.tenant_id)
                    .map_err(|_| Status::invalid_argument("invalid tenant_id UUID"))?)
            } else {
                None
            },
            created_at: None,
            updated_at: None,
            is_active: Some(true),
        };

        let updated_automation = automation::update_automation_rule(automation_update.id, automation_update)
            .await
            .map_err(|e| Status::internal(format!("db error: {}", e)))?;

        let automation_response = api::Automation {
            id: updated_automation.id as i64,
            user_id: updated_automation.user_id.as_ref().and_then(|id| uuid::Uuid::from_slice(id.as_ref()).ok()).map(|uuid| uuid.to_string()).unwrap_or_default(),
            sender_sensor: updated_automation.sender_sensor.unwrap_or_default(),
            receiver_sensor: updated_automation.receiver_sensor.unwrap_or_default(),
            condition: updated_automation.condition.unwrap_or_default(),
            action: updated_automation.action.unwrap_or_default(),
            sender_device_type: updated_automation.sender_device_type.unwrap_or_default() as i64,
            receiver_device_type: updated_automation.receiver_device_type.unwrap_or_default() as i64,
            sender_device_name: updated_automation.sender_device_name.unwrap_or_default(),
            receiver_device_name: updated_automation.receiver_device_name.unwrap_or_default(),
            trigger_type: updated_automation.trigger_type.unwrap_or_default(),
            trigger_time: updated_automation.trigger_time.map(|t| {
                let dt = Utc.from_utc_datetime(&t);
                prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos: dt.timestamp_subsec_nanos() as i32,
                }
            }),
            tenant_id: updated_automation.tenant_id.map(|id| id.to_string()).unwrap_or_default(),
            created_at: updated_automation.created_at.map(|t| {
                let dt = Utc.from_utc_datetime(&t);
                prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos: dt.timestamp_subsec_nanos() as i32,
                }
            }),
            updated_at: updated_automation.updated_at.map(|t| {
                let dt = Utc.from_utc_datetime(&t);
                prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos: dt.timestamp_subsec_nanos() as i32,
                }
            }),
        };

        let response = api::GetAutomationResponse {
            automation: Some(automation_response),
            ..Default::default()
        };

        Ok(tonic::Response::new(response))
    }

    async fn delete(
        &self,
        request: tonic::Request<api::DeleteAutomationRequest>,
    ) -> Result<Response<()>, Status> {
        // self.validator.validate_request(request).await
        let req = request.get_ref().id;
        if req == 0 {
            return Err(Status::invalid_argument("id is required"));
        }
        let id = req as i32;

        automation::delete_automation_rule(id)
            .await
            .map_err(|e| Status::internal(format!("db error: {}", e)))?;

        Ok(Response::new(()))
    }
}
