use chrono::Utc;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::Insertable;
use diesel::Queryable;
use diesel::Identifiable;
use diesel_async::RunQueryDsl;
use lrwn::EUI64;
use serde_json::Value;
use uuid::Uuid;

use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive}; // ✅ use correct BigDecimal crate

use crate::storage::device::Device;
// ⚠️ no `use crate::storage::fields::*;` here to avoid name conflicts
use crate::storage::schema::device_data_2025;

#[derive(Debug, serde::Deserialize)]
pub struct LSN50V2JSON {
    pub bat_v: f32,
    pub adc_ch0v: f32,
    pub hum_sht: String,
    pub temp_c_sht: String,
    pub ext_sensor: String,
    pub temp_c_ds: String,
    pub digital_istatus: String,
    pub door_status: String,
    pub exti_trigger: String,
    pub temp_c1: String,
    pub work_mode: String,
}

#[derive(Debug, Clone, Queryable, QueryableByName, Identifiable)]
#[diesel(table_name = device_data_2025)]
pub struct DeviceData2025 {
    pub id: i32,
    pub dev_eui: String,
    pub device_type_id: i32,
    pub org_id: i32,
    pub air_temperature: Option<rust_decimal::Decimal>,
    pub air_humidity: Option<rust_decimal::Decimal>,
    pub sol_temperature: Option<rust_decimal::Decimal>,
    pub sol_water: Option<rust_decimal::Decimal>,
    pub sol_conduct_soil: Option<rust_decimal::Decimal>,
    pub submission_date: Option<NaiveDateTime>,
    pub water_leak_status: Option<i32>,
    pub water_leak_times: Option<i32>,
    pub last_water_leak_duration: Option<i32>,
    pub door_open_status: Option<i32>,
    pub door_open_times: Option<i32>,
    pub last_door_open_duration: Option<i32>,
    pub batv: Option<rust_decimal::Decimal>,
    pub ro1_status: Option<i32>,
    pub ro2_status: Option<i32>,
    pub ph_soil: Option<rust_decimal::Decimal>,
    pub co2_ppm: Option<rust_decimal::Decimal>,
    pub tvoc_ppm: Option<rust_decimal::Decimal>,
    pub sensecap_light: Option<rust_decimal::Decimal>,
    pub barometric_pressure: Option<rust_decimal::Decimal>,
    pub status: Option<i32>,
    pub current: Option<rust_decimal::Decimal>,
    pub factor: Option<rust_decimal::Decimal>,
    pub power: Option<rust_decimal::Decimal>,
    pub power_sum: Option<rust_decimal::Decimal>,
    pub voltage: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = device_data_2025)]
pub struct NewDeviceData2025<'a> {
    pub dev_eui: &'a str,
    pub air_temperature: Option<BigDecimal>,
    pub air_humidity: Option<BigDecimal>,
    pub batv: Option<BigDecimal>,
    pub org_id: i32,
    pub device_type_id: i32,
}

pub async fn write_data_from_object_json(
    conn: &mut diesel_async::AsyncPgConnection,
    device: &Device,
    object_json: &Value,
    org_id: i64,
) -> anyhow::Result<()> {
    match device.device_type {
        Some(1) => {
            let parsed: LSN50V2JSON = serde_json::from_value(object_json.clone())?;

            if parsed.temp_c_sht == "-45" {
                return Ok(()); // skip invalid reading
            }

            let temp_raw = parsed.temp_c_sht.parse::<f32>()?;
            let hum_raw = parsed.hum_sht.parse::<f32>()?;

            let temp_cal = device
                .temperature_calibration
                .as_ref()
                .and_then(|v| v.to_f32())
                .unwrap_or(0.0);

            let hum_cal = device
                .humadity_calibration
                .as_ref()
                .and_then(|v| v.to_f32())
                .unwrap_or(0.0);

            let temp = temp_raw + temp_cal;
            let hum = hum_raw + hum_cal;
            let dev_eui_string = device.dev_eui.to_string();

            let new_data = NewDeviceData2025 {
                dev_eui: &dev_eui_string,
                air_temperature: BigDecimal::from_f32(temp),
                air_humidity: BigDecimal::from_f32(hum),
                batv: BigDecimal::from_f32(parsed.bat_v),
                org_id: org_id as i32,
                device_type_id: device.device_type.unwrap_or(1),
            };

            diesel::insert_into(device_data_2025::table)
                .values(&new_data)
                .execute(conn)
                .await?;
        }

        _ => {
            tracing::warn!("Unsupported device type: {:?}", device.device_type);
        }
    }

    Ok(())
}
