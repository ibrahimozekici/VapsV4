use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::{dsl, prelude::*};
use diesel_async::RunQueryDsl;
use tracing::info;
use uuid::Uuid;

use lrwn::region::{CommonName, MacVersion, Revision};

use super::error::Error;
use super::schema::device_profile;
use super::schema::device_type_tb;
use super::{error, fields, get_async_db_conn};
use crate::api::helpers::ToProto;
use crate::codec::Codec;
use chirpstack_api::internal;

#[derive(Clone, Queryable, Insertable, Debug, PartialEq, Eq)]
#[diesel(table_name = device_profile)]
pub struct DeviceProfile {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub region: CommonName,
    pub mac_version: MacVersion,
    pub reg_params_revision: Revision,
    pub adr_algorithm_id: String,
    pub payload_codec_runtime: Codec,
    pub uplink_interval: i32,
    pub device_status_req_interval: i32,
    pub supports_otaa: bool,
    pub supports_class_b: bool,
    pub supports_class_c: bool,
    pub tags: fields::KeyValue,
    pub payload_codec_script: String,
    pub flush_queue_on_activate: bool,
    pub description: String,
    pub measurements: fields::Measurements,
    pub auto_detect_measurements: bool,
    pub region_config_id: Option<String>,
    pub allow_roaming: bool,
    pub rx1_delay: i16,
    pub abp_params: Option<serde_json::Value>,
    pub class_b_params: Option<serde_json::Value>,
    pub class_c_params: Option<serde_json::Value>,
    pub relay_params: Option<serde_json::Value>,
    pub class_b_timeout: Option<i32>,
    pub class_b_ping_slot_nb_k: Option<i32>,
    pub class_b_ping_slot_dr: Option<i16>,
    pub class_b_ping_slot_freq: Option<i64>,
    pub class_c_timeout: Option<i32>,
    pub abp_rx1_delay: Option<i16>,
    pub abp_rx1_dr_offset: Option<i16>,
    pub abp_rx2_dr: Option<i16>,
    pub abp_rx2_freq: Option<i64>,
    pub is_relay: Option<bool>,
    pub is_relay_ed: Option<bool>,
    pub relay_ed_relay_only: Option<bool>,
    pub relay_enabled: Option<bool>,
    pub relay_cad_periodicity: Option<i16>,
    pub relay_default_channel_index: Option<i16>,
    pub relay_second_channel_freq: Option<i64>,
    pub relay_second_channel_dr: Option<i16>,
    pub relay_second_channel_ack_offset: Option<i16>,
    pub relay_ed_activation_mode: Option<i16>,
    pub relay_ed_smart_enable_level: Option<i16>,
    pub relay_ed_back_off: Option<i16>,
    pub relay_ed_uplink_limit_bucket_size: Option<i16>,
    pub relay_ed_uplink_limit_reload_rate: Option<i16>,
    pub relay_join_req_limit_reload_rate: Option<i16>,
    pub relay_notify_limit_reload_rate: Option<i16>,
    pub relay_global_uplink_limit_reload_rate: Option<i16>,
    pub relay_overall_limit_reload_rate: Option<i16>,
    pub relay_join_req_limit_bucket_size: Option<i16>,
    pub relay_notify_limit_bucket_size: Option<i16>,
    pub relay_global_uplink_limit_bucket_size: Option<i16>,
    pub relay_overall_limit_bucket_size: Option<i16>,
}

impl DeviceProfile {
    fn validate(&self) -> Result<(), Error> {
        if self.name.is_empty() {
            return Err(Error::Validation("name is not set".into()));
        }

        if self.rx1_delay < 0 || self.rx1_delay > 15 {
            return Err(Error::Validation("RX1 Delay must be between 0 - 15".into()));
        }

        Ok(())
    }
}

impl Default for DeviceProfile {
    fn default() -> Self {
        let now = Utc::now();

        DeviceProfile {
            id: Uuid::new_v4(),
            tenant_id: Uuid::nil(),
            created_at: now,
            updated_at: now,
            name: "".into(),
            description: "".into(),
            region: CommonName::EU868,
            mac_version: MacVersion::LORAWAN_1_0_0,
            reg_params_revision: Revision::A,
            adr_algorithm_id: "".into(),
            payload_codec_runtime: Codec::NONE,
            payload_codec_script: "".into(),
            flush_queue_on_activate: false,
            uplink_interval: 0,
            device_status_req_interval: 0,
            supports_otaa: false,
            supports_class_b: false,
            supports_class_c: false,
            tags: fields::KeyValue::new(HashMap::new()),
            measurements: fields::Measurements::new(HashMap::new()),
            auto_detect_measurements: false,
            region_config_id: None,
            abp_params: None,
            class_b_params: None,
            class_c_params: None,
            relay_params: None,
            class_b_timeout: Some(0),
            class_b_ping_slot_nb_k: Some(0),
            class_b_ping_slot_dr: Some(0),
            class_b_ping_slot_freq: Some(0),
            class_c_timeout: Some(0),
            abp_rx1_delay: Some(0),
            abp_rx1_dr_offset: Some(0),
            abp_rx2_dr: Some(0),
            abp_rx2_freq: Some(0),
            is_relay: Some(false),
            is_relay_ed: Some(false),
            relay_ed_relay_only: Some(false),
            relay_enabled: Some(false),
            relay_cad_periodicity: Some(0),
            relay_default_channel_index: Some(0),
            relay_second_channel_freq: Some(0),
            relay_second_channel_dr: Some(0),
            relay_second_channel_ack_offset: Some(0),
            relay_ed_activation_mode: Some(lrwn::RelayModeActivation::DisableRelayMode as i16),
            relay_ed_smart_enable_level: Some(0),
            relay_ed_back_off: Some(0),
            relay_ed_uplink_limit_bucket_size: Some(0),
            relay_ed_uplink_limit_reload_rate: Some(0),
            relay_join_req_limit_reload_rate: Some(0),
            relay_notify_limit_reload_rate: Some(0),
            relay_global_uplink_limit_reload_rate: Some(0),
            relay_overall_limit_reload_rate: Some(0),
            relay_join_req_limit_bucket_size: Some(0),
            relay_notify_limit_bucket_size: Some(0),
            relay_global_uplink_limit_bucket_size: Some(0),
            relay_overall_limit_bucket_size: Some(0),
            allow_roaming: false,
            rx1_delay: 0,
        }
    }
}

impl DeviceProfile {
    pub fn reset_session_to_boot_params(&self, ds: &mut internal::DeviceSession) {
        ds.mac_version = self.mac_version.to_proto().into();
        ds.class_b_ping_slot_dr = self.class_b_ping_slot_dr.unwrap_or(0) as u32;
        ds.class_b_ping_slot_freq = self.class_b_ping_slot_freq.unwrap_or(0) as u32;
        ds.class_b_ping_slot_nb = 1 << self.class_b_ping_slot_nb_k.unwrap_or(0);

        ds.nb_trans = 1;

        if self.is_relay_ed.unwrap_or(false) {
            ds.relay = Some(internal::Relay {
                ed_relay_only: self.relay_ed_relay_only.unwrap_or(false),
                ..Default::default()
            });
        }

        if !self.supports_otaa {
            ds.tx_power_index = 0;
            ds.min_supported_tx_power_index = 0;
            ds.max_supported_tx_power_index = 0;
            ds.extra_uplink_channels = HashMap::new();

            ds.rx1_delay = self.abp_rx1_delay.unwrap_or(0) as u32;
            ds.rx1_dr_offset = self.abp_rx1_dr_offset.unwrap_or(0) as u32;
            ds.rx2_dr = self.abp_rx2_dr.unwrap_or(0) as u32;
            ds.rx2_frequency = self.abp_rx2_freq.unwrap_or(0) as u32;

            ds.enabled_uplink_channel_indices = Vec::new();
        }
    }
}

#[derive(Queryable, PartialEq, Eq, Debug)]
pub struct DeviceProfileListItem {
    pub id: fields::Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub region: CommonName,
    pub mac_version: MacVersion,
    pub reg_params_revision: Revision,
    pub supports_otaa: bool,
    pub supports_class_b: bool,
    pub supports_class_c: bool,
}

#[derive(Default, Clone)]
pub struct Filters {
    pub tenant_id: Option<Uuid>,
    pub search: Option<String>,
}

pub async fn create(dp: DeviceProfile) -> Result<DeviceProfile, Error> {
    dp.validate()?;

    let dp: DeviceProfile = diesel::insert_into(device_profile::table)
        .values(&dp)
        .get_result(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| error::Error::from_diesel(e, dp.id.to_string()))?;
    info!(id = %dp.id, "Device-profile created");
    Ok(dp)
}

pub async fn get(id: &Uuid) -> Result<DeviceProfile, Error> {
    let dp = device_profile::dsl::device_profile
        .find(&fields::Uuid::from(id))
        .first(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| error::Error::from_diesel(e, id.to_string()))?;
    Ok(dp)
}
pub async fn get_internal(device_type_id: i64) -> Result<Uuid, Error> {
    use device_profile::dsl as dp;
    use device_type_tb::dsl as dt;

    let mut conn = get_async_db_conn().await?;

    let query = dp::device_profile
        .inner_join(dt::device_type_tb.on(dp::name.eq(dt::device_profile_name)))
        .filter(dt::id.eq(device_type_id as i32))
        .order_by(dp::created_at.desc())
        .select(dp::id);

    let result: Option<Uuid> = query.first::<Uuid>(&mut conn).await.optional()?;
    match result {
        Some(uuid) => Ok(uuid),
        None => Err(anyhow::anyhow!(
            "device_profile not found for device_type_id {}",
            device_type_id
        ).into()),
    }
}

pub async fn update(dp: DeviceProfile) -> Result<DeviceProfile, Error> {
    dp.validate()?;

    let dp: DeviceProfile = diesel::update(device_profile::dsl::device_profile.find(&dp.id))
        .set((
            device_profile::updated_at.eq(Utc::now()),
            device_profile::name.eq(&dp.name),
            device_profile::description.eq(&dp.description),
            device_profile::region.eq(&dp.region),
            device_profile::mac_version.eq(&dp.mac_version),
            device_profile::reg_params_revision.eq(&dp.reg_params_revision),
            device_profile::adr_algorithm_id.eq(&dp.adr_algorithm_id),
            device_profile::payload_codec_runtime.eq(&dp.payload_codec_runtime),
            device_profile::payload_codec_script.eq(&dp.payload_codec_script),
            device_profile::flush_queue_on_activate.eq(&dp.flush_queue_on_activate),
            device_profile::uplink_interval.eq(&dp.uplink_interval),
            device_profile::device_status_req_interval.eq(&dp.device_status_req_interval),
            device_profile::supports_otaa.eq(&dp.supports_otaa),
            device_profile::supports_class_b.eq(&dp.supports_class_b),
            device_profile::supports_class_c.eq(&dp.supports_class_c),
            device_profile::class_b_timeout.eq(&dp.class_b_timeout),
            device_profile::class_b_ping_slot_nb_k.eq(&dp.class_b_ping_slot_nb_k),
            device_profile::class_b_ping_slot_dr.eq(&dp.class_b_ping_slot_dr),
            device_profile::class_b_ping_slot_freq.eq(&dp.class_b_ping_slot_freq),
            device_profile::class_c_timeout.eq(&dp.class_c_timeout),
            device_profile::abp_rx1_delay.eq(&dp.abp_rx1_delay),
            device_profile::abp_rx1_dr_offset.eq(&dp.abp_rx1_dr_offset),
            device_profile::abp_rx2_dr.eq(&dp.abp_rx2_dr),
            device_profile::abp_rx2_freq.eq(&dp.abp_rx2_freq),
            device_profile::tags.eq(&dp.tags),
            device_profile::measurements.eq(&dp.measurements),
            device_profile::auto_detect_measurements.eq(&dp.auto_detect_measurements),
            device_profile::region_config_id.eq(&dp.region_config_id),
            device_profile::is_relay.eq(&dp.is_relay),
            device_profile::is_relay_ed.eq(&dp.is_relay_ed),
            device_profile::relay_ed_relay_only.eq(&dp.relay_ed_relay_only),
            device_profile::relay_enabled.eq(&dp.relay_enabled),
            device_profile::relay_cad_periodicity.eq(&dp.relay_cad_periodicity),
            device_profile::relay_default_channel_index.eq(&dp.relay_default_channel_index),
            device_profile::relay_second_channel_freq.eq(&dp.relay_second_channel_freq),
            device_profile::relay_second_channel_dr.eq(&dp.relay_second_channel_dr),
            device_profile::relay_second_channel_ack_offset.eq(&dp.relay_second_channel_ack_offset),
            device_profile::relay_ed_activation_mode.eq(&dp.relay_ed_activation_mode),
            device_profile::relay_ed_smart_enable_level.eq(&dp.relay_ed_smart_enable_level),
            device_profile::relay_ed_back_off.eq(&dp.relay_ed_back_off),
            device_profile::relay_ed_uplink_limit_bucket_size
                .eq(&dp.relay_ed_uplink_limit_bucket_size),
            device_profile::relay_ed_uplink_limit_reload_rate
                .eq(&dp.relay_ed_uplink_limit_reload_rate),
            device_profile::relay_join_req_limit_reload_rate
                .eq(&dp.relay_join_req_limit_reload_rate),
            device_profile::relay_notify_limit_reload_rate.eq(&dp.relay_notify_limit_reload_rate),
            device_profile::relay_global_uplink_limit_reload_rate
                .eq(&dp.relay_global_uplink_limit_reload_rate),
            device_profile::relay_overall_limit_reload_rate.eq(&dp.relay_overall_limit_reload_rate),
            device_profile::relay_join_req_limit_bucket_size
                .eq(&dp.relay_join_req_limit_bucket_size),
            device_profile::relay_notify_limit_bucket_size.eq(&dp.relay_notify_limit_bucket_size),
            device_profile::relay_global_uplink_limit_bucket_size
                .eq(&dp.relay_global_uplink_limit_bucket_size),
            device_profile::relay_overall_limit_bucket_size.eq(&dp.relay_overall_limit_bucket_size),
            device_profile::allow_roaming.eq(&dp.allow_roaming),
            device_profile::rx1_delay.eq(&dp.rx1_delay),
        ))
        .get_result(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| error::Error::from_diesel(e, dp.id.to_string()))?;

    info!(id = %dp.id, "Device-profile updated");
    Ok(dp)
}

pub async fn set_measurements(id: Uuid, m: &fields::Measurements) -> Result<DeviceProfile, Error> {
    let dp: DeviceProfile =
        diesel::update(device_profile::dsl::device_profile.find(&fields::Uuid::from(id)))
            .set(device_profile::measurements.eq(m))
            .get_result(&mut get_async_db_conn().await?)
            .await
            .map_err(|e| Error::from_diesel(e, id.to_string()))?;
    info!(id = %id, "Device-profile measurements updated");
    Ok(dp)
}

pub async fn delete(id: &Uuid) -> Result<(), Error> {
    let ra = diesel::delete(device_profile::dsl::device_profile.find(&fields::Uuid::from(id)))
        .execute(&mut get_async_db_conn().await?)
        .await?;
    if ra == 0 {
        return Err(error::Error::NotFound(id.to_string()));
    }
    info!(id = %id, "Device-profile deleted");
    Ok(())
}

pub async fn get_count(filters: &Filters) -> Result<i64, Error> {
    let mut q = device_profile::dsl::device_profile
        .select(dsl::count_star())
        .into_boxed();

    if let Some(tenant_id) = &filters.tenant_id {
        q = q.filter(device_profile::dsl::tenant_id.eq(fields::Uuid::from(tenant_id)));
    }

    if let Some(search) = &filters.search {
        #[cfg(feature = "postgres")]
        {
            q = q.filter(device_profile::dsl::name.ilike(format!("%{}%", search)));
        }
        #[cfg(feature = "sqlite")]
        {
            q = q.filter(device_profile::dsl::name.like(format!("%{}%", search)));
        }
    }

    Ok(q.first(&mut get_async_db_conn().await?).await?)
}

pub async fn list(
    limit: i64,
    offset: i64,
    filters: &Filters,
) -> Result<Vec<DeviceProfileListItem>, Error> {
    let mut q = device_profile::dsl::device_profile
        .select((
            device_profile::id,
            device_profile::created_at,
            device_profile::updated_at,
            device_profile::name,
            device_profile::region,
            device_profile::mac_version,
            device_profile::reg_params_revision,
            device_profile::supports_otaa,
            device_profile::supports_class_b,
            device_profile::supports_class_c,
        ))
        .into_boxed();

    if let Some(tenant_id) = &filters.tenant_id {
        q = q.filter(device_profile::dsl::tenant_id.eq(fields::Uuid::from(tenant_id)));
    }

    if let Some(search) = &filters.search {
        #[cfg(feature = "postgres")]
        {
            q = q.filter(device_profile::dsl::name.ilike(format!("%{}%", search)));
        }
        #[cfg(feature = "sqlite")]
        {
            q = q.filter(device_profile::dsl::name.like(format!("%{}%", search)));
        }
    }

    let items = q
        .order_by(device_profile::dsl::name)
        .limit(limit)
        .offset(offset)
        .load(&mut get_async_db_conn().await?)
        .await?;
    Ok(items)
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::storage;
    use crate::test;

    struct FilterTest<'a> {
        filters: Filters,
        dps: Vec<&'a DeviceProfile>,
        count: usize,
        limit: i64,
        offset: i64,
    }

    pub async fn create_device_profile(tenant_id: Option<Uuid>) -> DeviceProfile {
        let tenant_id = match tenant_id {
            Some(v) => v.into(),
            None => {
                let t = storage::tenant::test::create_tenant().await;
                t.id
            }
        };

        let mut kv = HashMap::new();
        kv.insert("foo".into(), "bar".into());

        let dp = DeviceProfile {
            tenant_id: tenant_id.into(),
            name: "test device-profile".into(),
            region: CommonName::EU868,
            mac_version: MacVersion::LORAWAN_1_0_2,
            reg_params_revision: Revision::B,
            adr_algorithm_id: "default".into(),
            payload_codec_runtime: Codec::JS,
            uplink_interval: 60,
            supports_otaa: true,
            tags: fields::KeyValue::new(kv),
            ..Default::default()
        };

        create(dp).await.unwrap()
    }

    #[tokio::test]
    async fn test_device_profile() {
        let _guard = test::prepare().await;
        let mut dp = create_device_profile(None).await;

        // get
        let dp_get = get(&dp.id).await.unwrap();
        assert_eq!(dp, dp_get);

        // update
        dp.name = "update device-profile".into();
        dp = update(dp).await.unwrap();
        let dp_get = get(&dp.id).await.unwrap();
        assert_eq!(dp, dp_get);

        // get count and list
        let tests = vec![
            FilterTest {
                filters: Filters {
                    tenant_id: None,
                    search: None,
                },
                dps: vec![&dp],
                count: 1,
                limit: 10,
                offset: 0,
            },
            FilterTest {
                filters: Filters {
                    tenant_id: None,
                    search: Some("proof".into()),
                },
                dps: vec![],
                count: 0,
                limit: 10,
                offset: 0,
            },
            FilterTest {
                filters: Filters {
                    tenant_id: None,
                    search: Some("prof".into()),
                },
                dps: vec![&dp],
                count: 1,
                limit: 10,
                offset: 0,
            },
            FilterTest {
                filters: Filters {
                    tenant_id: Some(dp.tenant_id.into()),
                    search: None,
                },
                dps: vec![&dp],
                count: 1,
                limit: 10,
                offset: 0,
            },
            FilterTest {
                filters: Filters {
                    tenant_id: Some(Uuid::new_v4()),
                    search: None,
                },
                dps: vec![],
                count: 0,
                limit: 10,
                offset: 0,
            },
        ];

        for tst in tests {
            let count = get_count(&tst.filters).await.unwrap() as usize;
            assert_eq!(tst.count, count);

            let items = list(tst.limit, tst.offset, &tst.filters).await.unwrap();
            assert_eq!(
                tst.dps
                    .iter()
                    .map(|dp| { dp.id.to_string() })
                    .collect::<String>(),
                items
                    .iter()
                    .map(|dp| { dp.id.to_string() })
                    .collect::<String>()
            );
        }

        // delete
        delete(&dp.id).await.unwrap();
        assert!(delete(&dp.id).await.is_err());
    }
}
