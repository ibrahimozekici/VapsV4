use super::auth::validator::{self, Validator};
use crate::storage::{fields, get_async_db_conn};
use std::str::FromStr;
use tracing::info;

use super::auth::AuthID;
use super::error::ToStatus;
use super::helpers::{self, FromProto, ToProto};
use chirpstack_api::api::alarm_service_server::AlarmService;
use chirpstack_api::api::{AlarmDateTime, CreateDoorTimeResponse}; // Import the correct AlarmDateTime type
use chirpstack_api::{api};
use lrwn::{EUI64};

use crate::storage::{
    alarm::{self},
    error::Error as StorageError,
    metrics,
};
use tonic::{Code, Request, Response, Status};

pub struct Alarm {
    validator: validator::RequestValidator,
}

impl Alarm {
    pub fn new(validator: validator::RequestValidator) -> Self {
        Alarm { validator }
    }
}

#[tonic::async_trait]
impl AlarmService for Alarm {
    async fn create(
        &self,
        request: Request<api::CreateAlarmRequest>,
    ) -> Result<Response<api::CreateAlarmResponse>, Status> {
        let req = &request.get_ref().create_alarm;
        if req.is_empty() {
            return Err(Status::invalid_argument("No alarms provided"));
        }
        let auth_id = request
            .extensions()
            .get::<AuthID>()
            .ok_or_else(|| Status::unauthenticated("no auth_id found in request extensions"))?;
        let user_id = match auth_id {
            AuthID::User(id) => id,
            _ => {
                return Err(Status::unauthenticated("no user id"));
            }
        };

        let mut response_alarms: Vec<api::Alarm> = Vec::new();

        for proto_alarm in req {
            let dev_eui = EUI64::from_str(&proto_alarm.dev_eui).map_err(|_| {
                Status::invalid_argument("Invalid dev_eui, must be a valid EUI64 string")
            })?;

            if proto_alarm.min_treshold > proto_alarm.max_treshold {
                return Err(Status::invalid_argument(
                    "Maksimum değer minimum değerden küçük olamaz",
                ));
            }

            let alarm = alarm::Alarm {
                dev_eui: dev_eui.to_string(),
                min_treshold: Some(proto_alarm.min_treshold as f64),
                max_treshold: Some(proto_alarm.max_treshold as f64),
                sms: Some(proto_alarm.sms),
                email: Some(proto_alarm.email),
                notification: Some(proto_alarm.notification),
                temperature: Some(proto_alarm.temperature),
                humadity: Some(proto_alarm.humadity),
                ec: Some(proto_alarm.ec),
                door: Some(proto_alarm.door),
                w_leak: Some(proto_alarm.w_leak),
                is_active: Some(true),
                zone_category: Some(proto_alarm.zone_category as i32),
                notification_sound: Some(proto_alarm.notification_sound.clone()),
                pressure: Some(proto_alarm.pressure),
                distance: Some(proto_alarm.distance),
                defrost_time: Some(proto_alarm.defrost_time as i32),
                is_time_limit_active: Some(proto_alarm.is_time_scheduled),
                alarm_start_time: Some(proto_alarm.start_time as f64),
                alarm_stop_time: Some(proto_alarm.end_time as f64),
                user_id: Some(proto_alarm.user_ids.iter().map(|&id| Some(id)).collect()),
                ..Default::default()
            };

            let alarm_dates: Vec<crate::storage::alarm::AlarmDateTime> = proto_alarm
                .alarm_date_time
                .iter()
                .map(|dt| crate::storage::alarm::AlarmDateTime {
                    alarm_id: proto_alarm.id as i32,
                    alarm_day: dt.alarm_day as i32,
                    start_time: dt.alarm_start_time as f64,
                    end_time: dt.alarm_end_time as f64,
                    ..Default::default()
                })
                .collect();

            let stored_alarm = alarm::create(alarm, alarm_dates.clone(), user_id.as_u128() as i64)
                .await
                .map_err(|e| Status::internal(format!("Failed to create alarm: {}", e)))?;

            let api_alarm = api::Alarm {
                id: stored_alarm.id as i64,
                dev_eui: stored_alarm.dev_eui.clone(),
                min_treshold: stored_alarm.min_treshold.unwrap_or_default() as f32,
                max_treshold: stored_alarm.max_treshold.unwrap_or_default() as f32,
                sms: stored_alarm.sms.unwrap_or(false),
                email: stored_alarm.email.unwrap_or(false),
                notification: stored_alarm.notification.unwrap_or(false),
                temperature: stored_alarm.temperature.unwrap_or(false),
                humadity: stored_alarm.humadity.unwrap_or(false),
                ec: stored_alarm.ec.unwrap_or(false),
                door: stored_alarm.door.unwrap_or(false),
                w_leak: stored_alarm.w_leak.unwrap_or(false),
                is_time_scheduled: stored_alarm.is_time_limit_active.unwrap_or(false),
                start_time: stored_alarm.alarm_start_time.unwrap_or(0.0) as f32,
                end_time: stored_alarm.alarm_stop_time.unwrap_or(0.0) as f32,
                zone_category: stored_alarm.zone_category.unwrap_or(0) as i64,
                pressure: stored_alarm.pressure.unwrap_or(false),
                notification_sound: stored_alarm.notification_sound.clone().unwrap_or_default(),
                distance: stored_alarm.distance.unwrap_or(false),
                defrost_time: stored_alarm.defrost_time.unwrap_or(0) as i64,
                alarm_date_time: alarm_dates
                    .iter()
                    .map(|dt| api::AlarmDateTime {
                        id: dt.id as i64,
                        alarm_id: dt.alarm_id as i64,
                        alarm_day: dt.alarm_day as i64,
                        alarm_start_time: dt.start_time as f32,
                        alarm_end_time: dt.end_time as f32,
                    })
                    .collect(),
                submission_date: Some(helpers::datetime_to_prost_timestamp(
                    (&chrono::Utc::now()).into(),
                )),
                ip_address: "0.0.0.0".to_string(),
                user_ids: proto_alarm.user_ids.clone(),
                time: chrono::Utc::now().timestamp() as i64,
                is_active: stored_alarm.is_active.unwrap_or(false),
            };

            response_alarms.push(api_alarm);
        }

        Ok(Response::new(api::CreateAlarmResponse {
            alarm: response_alarms,
        }))
    }

    async fn get(
        &self,
        request: Request<api::GetAlarmRequest>,
    ) -> Result<Response<api::GetAlarmResponse>, Status> {
        let alarm_id = &request.get_ref().alarm_id;
        let alarm_id = alarm_id
            .parse::<i32>()
            .map_err(|_| Status::invalid_argument("Invalid alarm_id, must be an integer"))?;

        // Get alarm and user_ids tuple
        let stored_alarm = alarm::get_alarm(alarm_id)
            .await
            .map_err(|e| Status::not_found(format!("Alarm not found: {}", e)))?;

        // Get alarm date time filters
        let alarm_dates = alarm::get_alarm_dates(alarm_id)
            .await
            .map_err(|e| Status::internal(format!("Error fetching alarm dates: {}", e)))?;

        let api_alarm = api::Alarm {
            id: stored_alarm.id as i64,
            dev_eui: stored_alarm.dev_eui.clone(),
            min_treshold: stored_alarm.min_treshold.unwrap_or_default() as f32,
            max_treshold: stored_alarm.max_treshold.unwrap_or_default() as f32,
            sms: stored_alarm.sms.unwrap_or(false),
            email: stored_alarm.email.unwrap_or(false),
            temperature: stored_alarm.temperature.unwrap_or(false),
            humadity: stored_alarm.humadity.unwrap_or(false),
            ec: stored_alarm.ec.unwrap_or(false),
            door: stored_alarm.door.unwrap_or(false),
            w_leak: stored_alarm.w_leak.unwrap_or(false),
            zone_category: stored_alarm.zone_category.unwrap_or_default() as i64,
            notification: stored_alarm.notification.unwrap_or(false),
            is_active: stored_alarm.is_active.unwrap_or(false),
            pressure: stored_alarm.pressure.unwrap_or(false),
            notification_sound: stored_alarm.notification_sound.unwrap_or_default(),
            distance: stored_alarm.distance.unwrap_or(false),
            defrost_time: stored_alarm.defrost_time.unwrap_or_default() as i64,
            alarm_date_time: alarm_dates
                .iter()
                .map(|dt| api::AlarmDateTime {
                    id: dt.id as i64,
                    alarm_id: dt.alarm_id as i64,
                    alarm_day: dt.alarm_day as i64,
                    alarm_start_time: dt.start_time as f32,
                    alarm_end_time: dt.end_time as f32,
                })
                .collect(),
            start_time: stored_alarm.alarm_start_time.unwrap_or_default() as f32,
            end_time: stored_alarm.alarm_stop_time.unwrap_or_default() as f32,
            submission_date: Some(helpers::datetime_to_prost_timestamp(
                (&chrono::Utc::now()).into(),
            )),
            ip_address: "0.0.0.0".to_string(),
            is_time_scheduled: stored_alarm.is_time_limit_active.unwrap_or(false),
            user_ids: stored_alarm
                .user_id
                .unwrap_or_default()
                .into_iter()
                .filter_map(|x| x)
                .collect(),
            time: chrono::Utc::now().timestamp() as i64,
        };

        Ok(Response::new(api::GetAlarmResponse {
            alarm: Some(api_alarm),
        }))
    }
    async fn list_all_organization_alarms(
        &self,
        _request: Request<api::ListOrganizationAlarmRequest2>,
    ) -> Result<Response<api::ListOrganizationAlarmResponse2>, Status> {
        let req = _request.get_ref();

        let organization_id = req.organization_id.clone();
        if organization_id == 0 {
            return Err(Status::invalid_argument("Organization ID is required"));
        }

        let alarms = alarm::get_organization_alarm_list(organization_id as i32)
            .await
            .map_err(|e| Status::internal(format!("Failed to list alarms: {}", e)))?;

        let api_alarms: Vec<api::ListOrganizationAlarmResponse> = alarms
            .into_iter()
            .map(|alarm| api::ListOrganizationAlarmResponse {
                id: alarm.id as i64,
                dev_eui: alarm.dev_eui.clone(),
                min_treshold: alarm.min_treshold.unwrap_or_default() as f32,
                max_treshold: alarm.max_treshold.unwrap_or_default() as f32,
                sms: alarm.sms.unwrap_or(false),
                email: alarm.email.unwrap_or(false),
                notification: alarm.notification.unwrap_or(false),
                temperature: alarm.temperature.unwrap_or(false),
                humadity: alarm.humadity.unwrap_or(false),
                ec: alarm.ec.unwrap_or(false),
                door: alarm.door.unwrap_or(false),
                w_leak: alarm.w_leak.unwrap_or(false),
                zone_category: alarm.zone_category.unwrap_or_default() as i64,
                is_active: alarm.is_active.unwrap_or(false),
                pressure: alarm.pressure.unwrap_or(false),
                notification_sound: alarm.notification_sound.unwrap_or_default(),
                distance: alarm.distance.unwrap_or(false),
                defrost_time: alarm.defrost_time.unwrap_or_default() as i64,
                is_time_scheduled: alarm.is_time_limit_active.unwrap_or(false),
                submission_date: Some(helpers::datetime_to_prost_timestamp(
                    (&chrono::Utc::now()).into(),
                )),
                alarm_date_time: vec![],
                user_ids: alarm
                    .user_id
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|x| x)
                    .collect(),
                time: chrono::Utc::now().timestamp() as i64,
                device_name: alarm.device_name.clone().unwrap_or_default(),
                username: alarm.username.clone().unwrap_or_default(),
                zone_name: alarm.zone_name.clone().unwrap_or_default(),
                ..Default::default()
            })
            .collect();
        let total_count = api_alarms.len() as i64;
        Ok(Response::new(api::ListOrganizationAlarmResponse2 {
            result: api_alarms.clone(),
            total_count,
        }))
    }

    async fn update(
        &self,
        request: Request<api::UpdateAlarmRequest>,
    ) -> Result<Response<()>, Status> {
        let req = &request.get_ref().alarm;
        if req.is_none() {
            return Err(Status::invalid_argument("No alarms provided"));
        }

        let mut response_alarms: Vec<api::Alarm> = Vec::new();

        for proto_alarm in req {
            let dev_eui = EUI64::from_str(&proto_alarm.dev_eui).map_err(|_| {
                Status::invalid_argument("Invalid dev_eui, must be a valid EUI64 string")
            })?;

            if proto_alarm.min_treshold > proto_alarm.max_treshold {
                return Err(Status::invalid_argument(
                    "Maksimum değer minimum değerden küçük olamaz",
                ));
            }
            let auth_id = request
                .extensions()
                .get::<AuthID>()
                .ok_or_else(|| Status::unauthenticated("no auth_id found in request extensions"))?;
            let user_id = match auth_id {
                AuthID::User(id) => id,
                _ => {
                    return Err(Status::unauthenticated("no user id"));
                }
            };
            let alarm = alarm::Alarm {
                id: proto_alarm.id as i32,
                dev_eui: dev_eui.to_string(),
                min_treshold: Some(proto_alarm.min_treshold as f64),
                max_treshold: Some(proto_alarm.max_treshold as f64),
                sms: Some(proto_alarm.sms),
                email: Some(proto_alarm.email),
                notification: Some(proto_alarm.notification),
                temperature: Some(proto_alarm.temperature),
                humadity: Some(proto_alarm.humadity),
                ec: Some(proto_alarm.ec),
                door: Some(proto_alarm.door),
                w_leak: Some(proto_alarm.w_leak),
                is_active: Some(true),
                zone_category: Some(proto_alarm.zone_category as i32),
                notification_sound: Some(proto_alarm.notification_sound.clone()),
                pressure: Some(proto_alarm.pressure),
                distance: Some(proto_alarm.distance),
                defrost_time: Some(proto_alarm.defrost_time as i32),
                is_time_limit_active: Some(proto_alarm.is_time_scheduled),
                alarm_start_time: Some(proto_alarm.start_time as f64),
                alarm_stop_time: Some(proto_alarm.end_time as f64),
                user_id: Some(proto_alarm.user_ids.iter().map(|&id| Some(id)).collect()),
                ..Default::default()
            };

            let alarm_dates: Vec<alarm::AlarmDateTime> = proto_alarm
                .alarm_date_time
                .iter()
                .map(|dt| alarm::AlarmDateTime {
                    id: dt.id as i32,
                    alarm_id: dt.alarm_id as i32,
                    alarm_day: dt.alarm_day as i32,
                    start_time: dt.alarm_start_time as f64,
                    end_time: dt.alarm_end_time as f64,
                    ..Default::default()
                })
                .collect();

            let previous_alarm = alarm::get_alarm(proto_alarm.id as i32).await.map_err(|e| {
                Status::internal(format!("Failed to fetch alarm for audit log: {}", e))
            })?;

            let result = alarm::update_alarm(alarm, alarm_dates.clone(), user_id.as_u128() as i64)
                .await
                .map_err(|e| Status::internal(format!("Failed to update alarm: {}", e)))?;
        }
        Ok(Response::new(()))
    }

    async fn delete(
        &self,
        _request: Request<api::DeleteAlarmRequest>,
    ) -> Result<Response<()>, Status> {
        let req = _request.get_ref();
        if req.alarm_id.is_empty() {
            return Err(Status::invalid_argument(
                "Alarm ID list is required and cannot be empty",
            ));
        }
        let auth_id = _request
            .extensions()
            .get::<AuthID>()
            .ok_or_else(|| Status::unauthenticated("no auth_id found in request extensions"))?;

        let user_id = match auth_id {
            AuthID::User(id) => id,
            _ => {
                return Err(Status::unauthenticated("no user id"));
            }
        };
        for alarm_id_str in &req.alarm_id {
            let alarm_id = alarm_id_str
                .parse::<i32>()
                .map_err(|_| Status::invalid_argument("Alarm ID must be a valid integer"))?;

            let previous_alarm = alarm::get_alarm(alarm_id).await.map_err(|e| {
                Status::internal(format!("Failed to fetch alarm for audit log: {}", e))
            })?;

            alarm::delete_alarm(alarm_id, user_id.as_u128() as i64)
                .await
                .map_err(|e| Status::internal(format!("Failed to delete alarm: {}", e)))?;
        }

        Ok(Response::new(()))
    }

    async fn create_multiple(
        &self,
        _request: Request<api::CreateMultipleAlarmRequest>,
    ) -> Result<Response<()>, Status> {
        let req = &_request.get_ref().create_alarm;
        if req.is_empty() {
            return Err(Status::invalid_argument("No alarms provided"));
        }
        let auth_id = _request
            .extensions()
            .get::<AuthID>()
            .ok_or_else(|| Status::unauthenticated("no auth_id found in request extensions"))?;

        let user_id = match auth_id {
            AuthID::User(id) => id,
            _ => {
                return Err(Status::unauthenticated("no user id"));
            }
        };
        let mut response_alarms: Vec<api::Alarm> = Vec::new();

        for proto_alarm in req {
            let dev_eui = EUI64::from_str(&proto_alarm.dev_eui).map_err(|_| {
                Status::invalid_argument("Invalid dev_eui, must be a valid EUI64 string")
            })?;

            if proto_alarm.min_treshold > proto_alarm.max_treshold {
                return Err(Status::invalid_argument(
                    "Maksimum değer minimum değerden küçük olamaz",
                ));
            }

            let alarm = alarm::Alarm {
                dev_eui: dev_eui.to_string(),
                min_treshold: Some(proto_alarm.min_treshold as f64),
                max_treshold: Some(proto_alarm.max_treshold as f64),
                sms: Some(proto_alarm.sms),
                email: Some(proto_alarm.email),
                notification: Some(proto_alarm.notification),
                temperature: Some(proto_alarm.temperature),
                humadity: Some(proto_alarm.humadity),
                ec: Some(proto_alarm.ec),
                door: Some(proto_alarm.door),
                w_leak: Some(proto_alarm.w_leak),
                is_active: Some(true),
                zone_category: Some(proto_alarm.zone_category as i32),
                notification_sound: Some(proto_alarm.notification_sound.clone()),
                pressure: Some(proto_alarm.pressure),
                distance: Some(proto_alarm.distance),
                defrost_time: Some(proto_alarm.defrost_time as i32),
                is_time_limit_active: Some(proto_alarm.is_time_scheduled),
                alarm_start_time: Some(proto_alarm.start_time as f64),
                alarm_stop_time: Some(proto_alarm.end_time as f64),
                user_id: Some(proto_alarm.user_ids.iter().cloned().map(Some).collect()),
                ..Default::default()
            };

            let alarm_dates: Vec<alarm::AlarmDateTime> = proto_alarm
                .alarm_date_time
                .iter()
                .map(|dt| alarm::AlarmDateTime {
                    alarm_id: proto_alarm.id as i32,
                    alarm_day: dt.alarm_day as i32,
                    start_time: dt.alarm_start_time as f64,
                    end_time: dt.alarm_end_time as f64,
                    ..Default::default()
                })
                .collect();

            let stored_alarm = alarm::create(alarm, alarm_dates.clone(), user_id.as_u128() as i64)
                .await
                .map_err(|e| Status::internal(format!("Failed to create alarm: {}", e)))?;

            let api_alarm = api::Alarm {
                id: stored_alarm.id as i64,
                dev_eui: stored_alarm.dev_eui.clone(),
                min_treshold: stored_alarm.min_treshold.unwrap_or_default() as f32,
                max_treshold: stored_alarm.max_treshold.unwrap_or_default() as f32,
                sms: stored_alarm.sms.unwrap_or(false),
                email: stored_alarm.email.unwrap_or(false),
                notification: stored_alarm.notification.unwrap_or(false),
                temperature: stored_alarm.temperature.unwrap_or(false),
                humadity: stored_alarm.humadity.unwrap_or(false),
                ec: stored_alarm.ec.unwrap_or(false),
                door: stored_alarm.door.unwrap_or(false),
                w_leak: stored_alarm.w_leak.unwrap_or(false),
                is_time_scheduled: stored_alarm.is_time_limit_active.unwrap_or(false),
                start_time: stored_alarm.alarm_start_time.unwrap_or_default() as f32,
                end_time: stored_alarm.alarm_stop_time.unwrap_or_default() as f32,
                zone_category: stored_alarm.zone_category.unwrap_or(0) as i64,
                pressure: stored_alarm.pressure.unwrap_or(false),
                notification_sound: stored_alarm.notification_sound.clone().unwrap_or_default(),
                distance: stored_alarm.distance.unwrap_or(false),
                defrost_time: stored_alarm.defrost_time.unwrap_or(0) as i64,
                alarm_date_time: alarm_dates
                    .iter()
                    .map(|dt| api::AlarmDateTime {
                        id: dt.id as i64,
                        alarm_id: dt.alarm_id as i64,
                        alarm_day: dt.alarm_day as i64,
                        alarm_start_time: dt.start_time as f32,
                        alarm_end_time: dt.end_time as f32,
                    })
                    .collect(),
                submission_date: Some(helpers::datetime_to_prost_timestamp(
                    (&chrono::Utc::now()).into(),
                )),
                ip_address: "".to_string(),
                user_ids: proto_alarm.user_ids.iter().cloned().collect(),
                time: chrono::Utc::now().timestamp() as i64,
                is_active: stored_alarm.is_active.unwrap_or(false),
            };
            response_alarms.push(api_alarm);
        }
        Ok(Response::new(()))
    }

    async fn delete_multiple(
        &self,
        _request: Request<api::DeleteMultipleAlarmRequest>,
    ) -> Result<Response<()>, Status> {
        let req = &_request.get_ref().user_id;
        if req.is_empty() {
            return Err(Status::invalid_argument("No users provided"));
        }
        let auth_id = _request
            .extensions()
            .get::<AuthID>()
            .ok_or_else(|| Status::unauthenticated("no auth_id found in request extensions"))?;

        let sent_user_id = match auth_id {
            AuthID::User(id) => id,
            _ => {
                return Err(Status::unauthenticated("no user id"));
            }
        };

        for &user_id in req {
            alarm::delete_user_alarm(user_id, sent_user_id.as_u128() as i64)
                .await
                .map_err(|e| Status::internal(format!("Failed to delete alarm: {}", e)))?;
        }
        Ok(Response::new(()))
    }

    async fn delete_sensor_alarms(
        &self,
        _request: Request<api::DeleteMultipleDevAlarmRequest>,
    ) -> Result<Response<()>, Status> {
        let req = &_request.get_ref().dev_euis;
        if req.is_empty() {
            return Err(Status::invalid_argument("No dev_eui provided"));
        }
        let auth_id = _request
            .extensions()
            .get::<AuthID>()
            .ok_or_else(|| Status::unauthenticated("no auth_id found in request extensions"))?;

        let sent_user_id = match auth_id {
            AuthID::User(id) => id,
            _ => {
                return Err(Status::unauthenticated("no user id"));
            }
        };

        for dev_eui in req {
            let dev_eui = EUI64::from_str(dev_eui).map_err(|_| {
                Status::invalid_argument("Invalid dev_eui, must be a valid EUI64 string")
            })?;
            alarm::delete_sensor_alarm(&dev_eui.to_string(), sent_user_id.as_u128() as i64)
                .await
                .map_err(|e| Status::internal(format!("Failed to delete alarm: {}", e)))?;
        }
        Ok(Response::new(()))
    }

    async fn delete_zone_alarms(
        &self,
        _request: Request<api::DeleteMultipleZoneAlarmRequest>,
    ) -> Result<Response<()>, Status> {
        let req = &_request.get_ref().zone_id;
        if req.is_empty() {
            return Err(Status::invalid_argument("No zone_id provided"));
        }
        let auth_id = _request
            .extensions()
            .get::<AuthID>()
            .ok_or_else(|| Status::unauthenticated("no auth_id found in request extensions"))?;

        let sent_user_id = match auth_id {
            AuthID::User(id) => id,
            _ => {
                return Err(Status::unauthenticated("no user id"));
            }
        };

        for zone_id in req {
            alarm::delete_zone_alarm(*zone_id as i32, sent_user_id.as_u128() as i64)
                .await
                .map_err(|e| Status::internal(format!("Failed to delete alarm: {}", e)))?;
        }
        Ok(Response::new(()))
    }

    async fn create_door_time(
        &self,
        _request: Request<api::CreateDoorTimeRequest>,
    ) -> Result<Response<api::CreateDoorTimeResponse>, Status> {
        let req = _request.get_ref();
        let auth_id = _request
            .extensions()
            .get::<AuthID>()
            .ok_or_else(|| Status::unauthenticated("no auth_id found in request extensions"))?;

        let user_id = match auth_id {
            AuthID::User(id) => id,
            _ => {
                return Err(Status::unauthenticated("no user id"));
            }
        };
        let alarm_dates: Vec<alarm::AlarmDateTime> = req
            .alarm_date_time
            .iter()
            .map(|dt| alarm::AlarmDateTime {
                alarm_id: req.id as i32,
                alarm_day: dt.alarm_day as i32,
                start_time: dt.alarm_start_time as f64,
                end_time: dt.alarm_end_time as f64,
                ..Default::default()
            })
            .collect();
        let create_alarm_req = alarm::DoorTimeAlarm {
            id: req.id as i32, // fix: expected i32
            dev_eui: Some(req.dev_eui.clone()),
            time: Some(req.time as i64),
            sms: Some(req.sms),
            email: Some(req.email),
            notification: Some(req.notification),
            is_active: Some(req.is_active),
            user_id: Some(req.user_id.iter().map(|&id| Some(id)).collect()), // fix: Vec<i64> → Option<Vec<Option<i64>>>
            organization_id: Some(req.organization_id as i32), // fix: expected Option<i32>
            submission_time: None,
            is_time_limit_active: Some(req.is_time_scheduled),
        };

        let stored_alarm = alarm::create_door_time_alarm(
            create_alarm_req,
            alarm_dates.clone(),
            user_id.as_u128() as i64,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to create alarm: {}", e)))?;

        let api_alarm = api::CreateDoorTimeResponse {
            id: stored_alarm.id as i64,
            dev_eui: stored_alarm.dev_eui.clone().unwrap_or_default(),
            time: stored_alarm.time.unwrap_or_default(),
            sms: stored_alarm.sms.unwrap_or(false),
            email: stored_alarm.email.unwrap_or(false),
            notification: stored_alarm.notification.unwrap_or(false),
            is_active: stored_alarm.is_active.unwrap_or(false),
            user_id: stored_alarm
                .user_id
                .unwrap_or_default()
                .into_iter()
                .filter_map(|id| id)
                .collect(),
            submission_date: Some(helpers::datetime_to_prost_timestamp(&chrono::Utc::now())),
        };
        Ok(Response::new(api_alarm))
    }

    async fn list_door_alarm2(
        &self,
        request: Request<api::ListDoorAlarmRequest2>,
    ) -> Result<Response<api::ListDoorAlarmResponse>, Status> {
        // Parse dev_eui
        let dev_eui = EUI64::from_str(&request.get_ref().dev_eui).map_err(|_| {
            Status::invalid_argument("Invalid dev_eui, must be a valid EUI64 string")
        })?;

        // Optionally: Validate node access here if you have a validator (not shown in Rust code)
        // Example:
        // self.validator.validate_node_access(&dev_eui, validator::Access::Read).await?;

        // Query storage for door time alarms
        let resp: Vec<CreateDoorTimeResponse> = alarm::list_door_time_alarms(dev_eui.to_string())
            .await
            .map_err(|e| Status::internal(format!("Failed to list door alarms: {}", e)))?;


        let total_count = resp.len() as i64;
        Ok(Response::new(api::ListDoorAlarmResponse {
            result: resp,
            total_count,
        }))
    }

    async fn delete_door_alarm(
        &self,
        _request: Request<api::DeleteDoorAlarmRequest>,
    ) -> Result<Response<()>, Status> {
        let req = _request.get_ref();
        if req.alarm_id.is_empty() {
            return Err(Status::invalid_argument(
                "Alarm ID list is required and cannot be empty",
            ));
        }

        for alarm_id_str in &req.alarm_id {
            let alarm_id = alarm_id_str
                .parse::<i32>()
                .map_err(|_| Status::invalid_argument("Alarm ID must be a valid integer"))?;

            alarm::delete_door_time_alarm(alarm_id)
                .await
                .map_err(|e| Status::internal(format!("Failed to delete alarm: {}", e)))?;
        }

        Ok(Response::new(()))
    }

    async fn create_multiple_door_alarm(
        &self,
        _request: Request<api::CreateMultipleDoorAlarmRequest>,
    ) -> Result<Response<()>, Status> {
        let req = &_request.get_ref().create_alarm;
        if req.is_empty() {
            return Err(Status::invalid_argument("No alarms provided"));
        }
        let auth_id = _request
            .extensions()
            .get::<AuthID>()
            .ok_or_else(|| Status::unauthenticated("no auth_id found in request extensions"))?;

        let user_id = match auth_id {
            AuthID::User(id) => id,
            _ => {
                return Err(Status::unauthenticated("no user id"));
            }
        };

        for create_req in req {
            let alarm_dates: Vec<alarm::AlarmDateTime> = create_req
                .alarm_date_time
                .iter()
                .map(|dt| alarm::AlarmDateTime {
                    alarm_id: create_req.id as i32,
                    alarm_day: dt.alarm_day as i32,
                    start_time: dt.alarm_start_time as f64,
                    end_time: dt.alarm_end_time as f64,
                    ..Default::default()
                })
                .collect();

            let create_alarm_req = alarm::DoorTimeAlarm {
                dev_eui: Some(create_req.dev_eui.clone()),
                time: Some(create_req.time as i64),
                sms: Some(create_req.sms),
                email: Some(create_req.email),
                notification: Some(create_req.notification),
                is_active: Some(true),
                user_id: Some(create_req.user_id.iter().cloned().map(Some).collect()),
                organization_id: Some(create_req.organization_id as i32),
                submission_time: None,
                is_time_limit_active: Some(create_req.is_time_scheduled),
                id: 0, // or create_req.id as i64 if needed
            };

            alarm::create_door_time_alarm(create_alarm_req, alarm_dates, user_id.as_u128() as i64)
                .await
                .map_err(|e| {
                    Status::internal(format!("Failed to create door time alarm: {}", e))
                })?;
        }
        info!("Door time alarms created successfully");
        Ok(Response::new(()))
    }

    async fn create_alarm_automation(
        &self,
        _request: Request<api::CreateAlarmAutomationRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn get_alarm_automation(
        &self,
        _request: Request<api::GetAlarmAutomationRequest>,
    ) -> Result<Response<api::GetAlarmAutomationResponse>, Status> {
        todo!()
    }

    async fn delete_alarm_automation(
        &self,
        _request: Request<api::DeleteAlarmAutomationRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn update_alarm_automation(
        &self,
        _request: Request<api::UpdateAlarmAutomationRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn get_audit_logs(
        &self,
        request: Request<api::GetAuditLogsRequest>,
    ) -> Result<Response<api::GetAuditLogsResponse>, Status> {
        let dev_eui = &request.get_ref().dev_eui;
        if dev_eui.is_empty() {
            return Err(Status::invalid_argument("dev_eui must not be empty"));
        }

        // Fetch logs from storage
        let logs = crate::storage::alarm::get_audit_logs(dev_eui)
            .await
            .map_err(|e| Status::internal(format!("Failed to get audit logs: {}", e)))?;

        let mut result = Vec::new();
        for log in logs {
            let changed_at = log.changed_at.map(|naive| {
                let dt_utc = chrono::DateTime::<chrono::Utc>::from_utc(naive, chrono::Utc);
                helpers::datetime_to_prost_timestamp(&dt_utc)
            });
            let row = api::AuditLog {
                log_id: log.id as i64,
                alarm_id: log.alarm_id as i64,
                dev_eui: log.dev_eui.clone().unwrap_or_default(),
                change_type: log.change_type.clone().unwrap_or_default(),
                changed_by: log.changed_by.unwrap_or_default() as i64,
                old_values: log
                    .old_values
                    .as_ref()
                    .map_or(String::new(), |v| v.to_string()),
                new_values: log
                    .new_values
                    .as_ref()
                    .map_or(String::new(), |v| v.to_string()),
                changed_at,
            };
            result.push(row);
        }

        Ok(Response::new(api::GetAuditLogsResponse { result }))
    }
}
