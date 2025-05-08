use std::str::FromStr;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::storage::zone::{self, ZoneDataSerde, ZoneDeviceProfileSerde, ZoneDeviceSerde};
use crate::{api::error::ToStatus, storage::zone::GetZonesItemSerde};
use chirpstack_api::api::zone_service_server::ZoneService;
use chirpstack_api::api::{self, AddUserToZoneRequest, AddUserToZoneResponse, GetZonesItem, ZoneDevice, ZoneDeviceProfile, ZonesOrderRequest};

use super::auth::{validator, AuthID};

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
            devices: req_app.devices.clone().into_iter().map(Some).collect(),
            zone_order: Some(req_app.order),
            zone_id: 0,
            content_type: Some(req_app.content_type),
            tanent_id: Some(tenant_id),
        };

        // Call internal `create` function
        let created_zone = zone::create(a).await.map_err(|e| e.status())?;

        // Convert internal model -> API response model
        let api_zone: api::Zone = created_zone.clone().into();

        let mut resp = Response::new(api::GetZoneResponse {
            zone: Some(api_zone),
        });

        // Add log metadata (assumes created_zone.zone_id is usable)
        resp.metadata_mut().insert(
            "x-log-zone_id",
            created_zone.zone_id.to_string().parse().unwrap(),
        );

        Ok(resp)
    }

    async fn get(
        &self,
        request: Request<api::GetZoneRequest>,
    ) -> Result<Response<api::GetZoneResponse>, Status> {
        let zone_id = request.get_ref().zone_id;

        // Convert i64 to i32 (safe only if zone_id fits)
        let zone_id_i32 = i32::try_from(zone_id).map_err(|_| {
            Status::invalid_argument(format!("zone_id {} is out of range for i32", zone_id))
        })?;

        // Fetch from DB
        let z = zone::get(&zone_id_i32).await.map_err(|e| e.status())?;

        let resp = api::GetZoneResponse {
            zone: Some(z.into()),
        };

        Ok(Response::new(resp))
    }

    async fn list(
        &self,
        request: Request<api::ListZoneRequest>,
    ) -> Result<Response<api::ListZoneResponse>, Status> {
        let req = request.get_ref();

        let limit = req.limit;
        let offset = req.offset;
        // let org_id = &req.organization_id;

        let tenant_id = Uuid::from_str(&req.organization_id).map_err(|e| e.status())?;

        self.validator
            .validate(
                request.extensions(),
                validator::ValidateTenantAccess::new(validator::Flag::Read, tenant_id),
            )
            .await?;
        let auth_id = request.extensions().get::<AuthID>().unwrap();
        let user_id = match auth_id {
            AuthID::User(id) => id,
            _ => {
                return Err(Status::unauthenticated("no user id"));
            }
        };
        let zones = zone::list(Some(user_id.clone()), Some(tenant_id.clone()))
            .await
            .map_err(|e| e)?;

        let zones_proto: Vec<api::GetZonesItem> = zones
            .zones
            .into_iter()
            .map(api::GetZonesItem::from)
            .collect();

        let resp = api::ListZoneResponse { zones: zones_proto };

        Ok(Response::new(resp))
    }

    async fn update(
        &self,
        _request: Request<api::UpdateZoneRequest>,
    ) -> Result<Response<api::GetZoneResponse>, Status> {
        Err(Status::unimplemented("update not implemented yet"))
    }

    async fn delete(
        &self,
        request: Request<api::DeleteZoneRequest>,
    ) -> Result<Response<()>, Status> {
        let zone_id = request.get_ref().zone_id;

        // Convert i64 -> i32 (assuming i32 DB type)
        let zone_id_i32 = i32::try_from(zone_id).map_err(|_| {
            Status::invalid_argument(format!("zone_id {} is out of range for i32", zone_id))
        })?;

        let deleted = zone::delete(zone_id_i32).await.map_err(|e| e.status())?;

        if deleted == 0 {
            return Err(Status::not_found("Zone not found"));
        }
        let mut resp = Response::new(());
        // resp.metadata_mut()
        //     .insert("x-log-dev_eui", zone_id);

        Ok(resp)
    }

}
impl From<zone::Zone> for api::Zone {
    fn from(z: zone::Zone) -> Self {
        api::Zone {
            zone_id: z.zone_id as i64,
            zone_name: z.zone_name.unwrap_or_default(),
            order: z.zone_order.unwrap_or_default(),
            content_type: z.content_type.unwrap_or_default(),
            org_id: z.tanent_id.map(|id| id.to_string()).unwrap_or_default(),
            devices: z.devices.into_iter().filter_map(|d| d).collect(),
        }
    }
}

// impl From<zone::Zone> for api::GetZonesItem {
//     fn from(z: zone::Zone) -> Self {
//         api::GetZonesItem {
//             zone_id: z.zone_id as i64,
//             zone_name: z.zone_name.unwrap_or_default(),
//             order: z.zone_order.unwrap_or_default(),
//             content_type: z.content_type.unwrap_or_default(),
//             org_id: z.tanent_id.map(|id| id.to_string()).unwrap_or_default(),
//             devices: z.devices.into_iter().filter_map(|d| d).collect(),
//         }
//     }
// }
impl From<GetZonesItemSerde> for GetZonesItem {
    fn from(item: GetZonesItemSerde) -> Self {
        Self {
            zone_id: item.zone_id,
            zone_name: item.zone_name,
            org_id: item.org_id,
            order: item.order,
            devices: item.devices.into_iter().map(Into::into).collect(),
            content_type: item.content_type,
        }
    }
}

impl From<ZoneDeviceSerde> for ZoneDevice {
    fn from(d: ZoneDeviceSerde) -> Self {
        Self {
            device_dev_eui: d.device_dev_eui,
            device_profile_id: d.device_profile_id,
            device_name: d.device_name,
            device_description: d.device_description,
            device_last_seen_at: d.device_last_seen_at,
            data: d.data.into_iter().map(Into::into).collect(), // assuming Vec<ZoneDataSerde> == Vec<ZoneData>
            device_profile_name: d.device_profile_name.into_iter().map(Into::into).collect(),
            device_type: d.device_type,
            temperature_calibration: d.temperature_calibration,
            humadity_calibration: d.humadity_calibration,
            variables: d.variables,
            tags: d.tags,
        }
    }
}

impl From<ZoneDataSerde> for chirpstack_api::api::ZoneData {
    fn from(data: ZoneDataSerde) -> Self {
        Self {
            id: data.id,
            dev_eui: data.dev_eui,
            device_type_id: data.device_type_id,
            org_id: data.org_id,
            air_temperature: data.air_temperature,
            air_humidity: data.air_humidity,
            sol_temperature: data.sol_temperature,
            sol_water: data.sol_water,
            sol_conduct_soil: data.sol_conduct_soil,
            submission_date: data.submission_date,
            water_leak_status: data.water_leak_status,
            water_leak_times: data.water_leak_times,
            last_water_leak_duration: data.last_water_leak_duration,
            door_open_status: data.door_open_status,
            door_open_times: data.door_open_times,
            last_door_open_duration: data.last_door_open_duration,
            batv: data.batv,
            ro1_status: data.ro1_status,
            ro2_status: data.ro2_status,
            ph_soil: data.ph_soil,
            co2_ppm: data.co2_ppm,
            tvoc_ppm: data.tvoc_ppm,
            sensecap_light: data.sensecap_light,
            barometric_pressure: data.barometric_pressure,
            power: data.power,
            current: data.current,
            voltage: data.voltage,
            factor: data.factor,
            power_sum: data.power_sum,
            status: data.status,
            power_consumption: data.power_consumption,
            switch1: data.switch1,
            switch2: data.switch2,
            switch3: data.switch3,
            switch4: data.switch4,
            switch5: data.switch5,
            switch6: data.switch6,
            switch7: data.switch7,
            switch8: data.switch8,
            adc_1: data.adc_1,
            adc_2: data.adc_2,
            adv_1: data.adv_1,
            gpio_in_1: data.gpio_in_1,
            gpio_in_2: data.gpio_in_2,
            gpio_in_3: data.gpio_in_3,
            gpio_in_4: data.gpio_in_4,
            gpio_out_1: data.gpio_out_1,
            gpio_out_2: data.gpio_out_2,
            distance: data.distance,
            position: data.position,
            temperature1: data.temperature1,
            temperature2: data.temperature2,
        }
    }
}

impl From<ZoneDeviceProfileSerde> for ZoneDeviceProfile {
    fn from(p: ZoneDeviceProfileSerde) -> Self {
        Self {
            // fill all fields here, for example:
            name: p.name,
            // ... other fields
        }
    }
}
