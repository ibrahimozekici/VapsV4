use std::str::FromStr;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use chirpstack_api::api;
use chirpstack_api::api::zone_service_server::ZoneService;
use crate::storage::zone;

use super::auth::validator;

pub struct Zone {
    validator: validator::RequestValidator,
}

impl Zone {
    pub fn new(validator: validator::RequestValidator) -> Self {
        Zone { validator }
    }
}

#[tonic::async_trait]
impl ZoneService for Zone {
    async fn create(
        &self,
        request: Request<api::CreateZoneRequest>,
    ) -> Result<Response<api::GetZoneResponse>, Status> {
        let req_app = match &request.get_ref().zone {
            Some(v) => v,
            None => {
                return Err(Status::invalid_argument("Zone is missing"));
            }
        };
        let tenant_id = Uuid::from_str(&req_app.org_id).map_err(|e| e.status())?;

        self.validator
            .validate(
                request.extensions(),
                validator::ValidateTenantAccess::new(validator::Flag::Read, tenant_id),
            )
            .await?;

        let a = zone::Zone {
            zone_name: Some(req_app.zone_name.clone()),
            devices: req_app.devices.clone(),
            zone_order: Some(req_app.order.clone()),
            zone_id: 0,
            content_type: Some(req_app.content_type.clone()),
            tanent_id: Some(0),
        };

        let a = zone::create(a).await.map_err(|e| e.status())?;

        let mut resp = Response::new(api::CreateApplicationResponse {
            id: a.id.to_string(),
        });
        resp.metadata_mut()
            .insert("x-log-application_id", a.id.to_string().parse().unwrap());

        Ok(resp)
    }



} 