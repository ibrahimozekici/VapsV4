use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive}; // ✅ use correct BigDecimal crate
use chrono::NaiveDateTime;
use diesel::prelude::*;
// use diesel::query_dsl::methods::OnConflictDsl;
use diesel::upsert::excluded;
use diesel::Identifiable;
use diesel::Insertable;
use diesel::Queryable;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use serde_json::to_string_pretty;
use serde_json::Value;
use tracing::info;

use crate::storage::device::Device;
// ⚠️ no `use crate::storage::fields::*;` here to avoid name conflicts
use crate::storage::schema::{
    am103, dds45lb, device_data_2025, device_data_latest, em400mud, ltc2lb,
};

#[derive(Debug, Clone, Queryable, QueryableByName, Identifiable)]
#[diesel(table_name = device_data_2025)]
pub struct DeviceData2025 {
    pub id: i32,
    pub dev_eui: String,
    pub device_type_id: i32,
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
    // pub org_id: i32,
    pub device_type_id: i32,
}
#[derive(Debug, serde::Deserialize)]
pub struct LSN50V2JSON {
    pub batv: f32,
    pub hum_sht: String,
    pub temp_c_sht: String,
}

#[derive(Debug, Deserialize)]
pub struct LSE01JSON {
    #[serde(rename = "BatV")]
    pub battery: f32,
    #[serde(rename = "conduct_SOIL")]
    pub conduct_soil: f32,
    #[serde(rename = "temp_SOIL")]
    pub temp_soil: String,
    #[serde(rename = "water_SOIL")]
    pub water_soil: String,
}
#[derive(Debug, Deserialize)]
pub struct LDS01JSON {
    // #[serde(rename = "BatV")]
    // pub battery: f32,
    #[serde(rename = "door_open_status")]
    pub door_status: f64,
    #[serde(rename = "door_open_times")]
    pub door_open_times: f64,
    #[serde(rename = "last_door_open_duration")]
    pub last_door_open_duration: f64,
}
#[derive(Debug, Deserialize)]
pub struct LT22222L {
    #[serde(rename = "ro1_status")]
    pub ro1_status: i32,
    #[serde(rename = "ro2_status")]
    pub ro2_status: i32,
    #[serde(rename = "gpio_in_1")]
    pub gpio_in_1: String,
    #[serde(rename = "gpio_in_2")]
    pub gpio_in_2: String,
    #[serde(rename = "gpio_out_1")]
    pub gpio_out_1: String,
    #[serde(rename = "gpio_out_2")]
    pub gpio_out_2: String,
}

#[derive(Debug, Deserialize)]
pub struct LWL01JSON {
    #[serde(rename = "BatV")]
    pub battery: f32,
    #[serde(rename = "WATER_LEAK_STATUS")]
    pub water_status: i32,
    #[serde(rename = "WATER_LEAK_TIMES")]
    pub water_leak_times: i32,
    #[serde(rename = "LAST_WATER_LEAK_DURATION")]
    pub last_water_leak_duration: i32,
}

#[derive(Debug, Deserialize)]
pub struct LHT65JSON {
    #[serde(rename = "BatV")]
    pub battery: f32,
    #[serde(rename = "Hum_SHT")]
    pub humidity: String,
    #[serde(rename = "TempC_SHT")]
    pub temperature: String,
    #[serde(rename = "Ext_sensor")]
    pub exp: Option<String>, // if needed
}

#[derive(Debug, serde::Deserialize)]
pub struct AM107JSON {
    pub battery: f32,
    pub humidity: f32,
    pub temperature: f32,
    pub co2: f32,
    pub tvoc: f32,
    pub pressure: f32,
}

#[derive(Debug, serde::Deserialize)]
pub struct LAQ4JSON {
    #[serde(rename = "BatV")]
    pub battery: f32,
    #[serde(rename = "Hum_SHT")]
    pub humidity: f32,
    #[serde(rename = "TempC_SHT")]
    pub temperature: f32,
    #[serde(rename = "CO2_ppm")]
    pub co2_ppm: f32,
    #[serde(rename = "TVOC_ppm")]
    pub tvoc_ppm: f32,
}

#[derive(Debug, serde::Deserialize)]
pub struct LSPH01JSON {
    #[serde(rename = "BatV")]
    pub battery: f32,
    #[serde(rename = "PH1_SOIL")]
    pub ph_soil: String,
    #[serde(rename = "TEMP_SOIL")]
    pub temp_soil: String,
}
#[derive(Debug, serde::Deserialize)]
pub struct WS101JSON {
    #[serde(rename = "press")]
    pub alarm: i32,
}
#[derive(Debug, serde::Deserialize)]
pub struct EM300MCS {
    #[serde(rename = "BatV")]
    pub battery: f32,
    #[serde(rename = "door_open_status")]
    pub door_status: i32,
}

#[derive(Debug, Deserialize)]
pub struct SenseCapLight {
    #[serde(rename = "measurementId")]
    pub measurement_id: i64,
    #[serde(rename = "measurementValue")]
    pub measurement_value: f32,
    #[serde(rename = "type")]
    pub r#type: String,
}
#[derive(Debug, serde::Deserialize)]
pub struct EM500PPJSON {
    pub battery: f32,
    pub pressure: f32, // in Pascals
}

#[derive(Debug, serde::Deserialize)]
pub struct WS522JSON {
    pub current: f32,
    pub factor: f32,
    pub power: f32,
    pub voltage: f32,
    pub state: i32,
    #[serde(rename = "power_sum")]
    pub power_sum: f32,
}

#[derive(Debug, serde::Deserialize)]
pub struct AM103Json {
    #[serde(rename = "battery")]
    pub battery: i64,
    pub humidity: f32,
    pub temperature: f32,
    pub co2: f32,
}

#[derive(Debug, serde::Deserialize)]
pub struct WS558Json {
    #[serde(rename = "active_power")]
    pub active_power: f32,
    #[serde(rename = "power_consumption")]
    pub power_consumption: f32,
    #[serde(rename = "power_factor")]
    pub power_factor: f32,
    #[serde(rename = "switch_1")]
    pub switch1: i32,
    #[serde(rename = "switch_2")]
    pub switch2: i32,
    #[serde(rename = "switch_3")]
    pub switch3: i32,
    #[serde(rename = "switch_4")]
    pub switch4: i32,
    #[serde(rename = "switch_5")]
    pub switch5: i32,
    #[serde(rename = "switch_6")]
    pub switch6: i32,
    #[serde(rename = "switch_7")]
    pub switch7: i32,
    #[serde(rename = "switch_8")]
    pub switch8: i32,
    pub voltage: f32,
    #[serde(rename = "total_current")]
    pub total_current: f32,
}

#[derive(Debug, serde::Deserialize)]
pub struct DDS45LB {
    #[serde(rename = "Bat")]
    pub battery: f32,
    #[serde(rename = "Distance")]
    pub distance: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct LTC2LB {
    pub temperature1: f32,
    pub temperature2: f32,
    #[serde(rename = "BatV")]
    pub battery: f32,
}

#[derive(Debug, serde::Deserialize)]
pub struct EM400MUD {
    #[serde(rename = "battery")]
    pub battery: i64,
    pub distance: i64,
    pub position: Option<String>,
    pub temperature: f32,
}

#[derive(Debug, serde::Deserialize)]
pub struct UC300Json {
    pub adc_1: Option<String>,
    pub adc_2: Option<String>,
    pub adv_1: Option<String>,
    pub gpio_in_1: Option<String>,
    pub gpio_in_2: Option<String>,
    pub gpio_in_3: Option<String>,
    pub gpio_in_4: Option<String>,
    pub gpio_out_1: Option<String>,
    pub gpio_out_2: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SensecapMessage {
    pub messages: Vec<SenseCapLight>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EM500PT100JSON {
    pub battery: f32,
    pub temperature: f32,
}

#[derive(Debug, serde::Deserialize)]
pub struct EM300THJSON {
    pub battery: f32,
    pub humidity: f32,
    pub temperature: f32,
}

#[derive(Debug, serde::Deserialize)]
pub struct EM300ZLDJSON {
    #[serde(rename = "water_leak")]
    pub water_leak: i32,
}

pub async fn write_data_from_object_json(
    conn: &mut diesel_async::AsyncPgConnection,
    device: &Device,
    object_json: &Value,
    // org_id: i64,
) -> anyhow::Result<()> {
    match to_string_pretty(device) {
        Ok(json) => info!("Device object:\n{}", json),
        Err(e) => info!("Failed to serialize device object: {}", e),
    }
    match device.device_type {
        Some(1) => {
            let parsed: LSN50V2JSON = serde_json::from_value(object_json.clone())?;
        
            if parsed.temp_c_sht == "-45" {
                return Ok(()); // skip invalid reading
            }
        
            let temp_raw = parsed.temp_c_sht.parse::<f32>()?;
            let hum_raw = parsed.hum_sht.parse::<f32>()?;
        
            let temp_cal = device.temperature_calibration.as_ref().and_then(|v| v.to_f32()).unwrap_or(0.0);
            let hum_cal = device.humadity_calibration.as_ref().and_then(|v| v.to_f32()).unwrap_or(0.0);
        
            let temp = temp_raw + temp_cal;
            let hum = hum_raw + hum_cal;
            let dev_eui_string = device.dev_eui.to_string();
        
            let air_temp = BigDecimal::from_f32(temp);
            let air_hum = BigDecimal::from_f32(hum);
            let batv = BigDecimal::from_f32(parsed.batv);
        
            // Insert device_data_2025
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::air_temperature.eq(air_temp.clone()),
                    device_data_2025::air_humidity.eq(air_hum.clone()),
                    device_data_2025::batv.eq(batv.clone()),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(1)),
                ))
                .execute(conn)
                .await?;
        
            // Upsert device_data_latest
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::air_temperature.eq(air_temp.clone()),
                    device_data_latest::air_humidity.eq(air_hum.clone()),
                    device_data_latest::batv.eq(batv.clone()),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(1)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::air_temperature.eq(excluded(device_data_latest::air_temperature)),
                    device_data_latest::air_humidity.eq(excluded(device_data_latest::air_humidity)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id.eq(excluded(device_data_latest::device_type_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;
        }
        
        Some(2) => {
            let parsed: LSE01JSON = serde_json::from_value(object_json.clone())?;
        
            if parsed.temp_soil != "0.00" {
                let temp_raw = parsed.temp_soil.parse::<f32>()?;
                let water_raw = parsed.water_soil.parse::<f32>()?;
        
                let temp_cal = device.temperature_calibration.as_ref().and_then(|v| v.to_f32()).unwrap_or(0.0);
                let hum_cal = device.humadity_calibration.as_ref().and_then(|v| v.to_f32()).unwrap_or(0.0);
        
                let temp = BigDecimal::from_f32(temp_raw + temp_cal);
                let water = BigDecimal::from_f32(water_raw + hum_cal);
                let conduct = BigDecimal::from_f32(parsed.conduct_soil);
                let batv = BigDecimal::from_f32(parsed.battery);
                let dev_eui_string = device.dev_eui.to_string();
        
                // Insert device_data_2025
                diesel::insert_into(device_data_2025::table)
                    .values((
                        device_data_2025::dev_eui.eq(&dev_eui_string),
                        device_data_2025::sol_temperature.eq(temp.clone()),
                        device_data_2025::sol_water.eq(water.clone()),
                        device_data_2025::sol_conduct_soil.eq(conduct.clone()),
                        device_data_2025::batv.eq(batv.clone()),
                        device_data_2025::device_type_id.eq(device.device_type.unwrap_or(2)),
                    ))
                    .execute(conn)
                    .await?;
        
                // Upsert device_data_latest
                diesel::insert_into(device_data_latest::table)
                    .values((
                        device_data_latest::dev_eui.eq(&dev_eui_string),
                        device_data_latest::sol_temperature.eq(temp),
                        device_data_latest::sol_water.eq(water),
                        device_data_latest::sol_conduct_soil.eq(conduct),
                        device_data_latest::batv.eq(batv),
                        device_data_latest::device_type_id.eq(device.device_type.unwrap_or(2)),
                    ))
                    .on_conflict(device_data_latest::dev_eui)
                    .do_update()
                    .set((
                        device_data_latest::sol_temperature.eq(excluded(device_data_latest::sol_temperature)),
                        device_data_latest::sol_water.eq(excluded(device_data_latest::sol_water)),
                        device_data_latest::sol_conduct_soil.eq(excluded(device_data_latest::sol_conduct_soil)),
                        device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                        device_data_latest::device_type_id.eq(excluded(device_data_latest::device_type_id)),
                        device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
                    .await?;
        
                info!("Inserted LSE01 data for dev_eui={}", dev_eui_string);
            }
        }
        
        Some(3) => {
            let parsed: LDS01JSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();
        
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::door_open_status.eq(parsed.door_status as i32),
                    device_data_2025::door_open_times.eq(parsed.door_open_times as i32),
                    device_data_2025::last_door_open_duration.eq(parsed.last_door_open_duration as i32),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(3)),
                ))
                .execute(conn)
                .await?;
        
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::door_open_status.eq(parsed.door_status as i32),
                    device_data_latest::door_open_times.eq(parsed.door_open_times as i32),
                    device_data_latest::last_door_open_duration.eq(parsed.last_door_open_duration as i32),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(3)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::door_open_status.eq(excluded(device_data_latest::door_open_status)),
                    device_data_latest::door_open_times.eq(excluded(device_data_latest::door_open_times)),
                    device_data_latest::last_door_open_duration.eq(excluded(device_data_latest::last_door_open_duration)),
                    device_data_latest::device_type_id.eq(excluded(device_data_latest::device_type_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;
        
            info!("Inserted LDS01 data for dev_eui={}", dev_eui_string);
        }
        
        Some(4) => {
            let parsed: LWL01JSON = serde_json::from_value(object_json.clone())?;
            let batv = BigDecimal::from_f32(parsed.battery);
            let dev_eui_string = device.dev_eui.to_string();
        
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::water_leak_status.eq(parsed.water_status),
                    device_data_2025::water_leak_times.eq(parsed.water_leak_times),
                    device_data_2025::last_water_leak_duration.eq(parsed.last_water_leak_duration),
                    device_data_2025::batv.eq(batv.clone()),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(4)),
                ))
                .execute(conn)
                .await?;
        
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::water_leak_status.eq(parsed.water_status),
                    device_data_latest::water_leak_times.eq(parsed.water_leak_times),
                    device_data_latest::last_water_leak_duration.eq(parsed.last_water_leak_duration),
                    device_data_latest::batv.eq(batv),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(4)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::water_leak_status.eq(excluded(device_data_latest::water_leak_status)),
                    device_data_latest::water_leak_times.eq(excluded(device_data_latest::water_leak_times)),
                    device_data_latest::last_water_leak_duration.eq(excluded(device_data_latest::last_water_leak_duration)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id.eq(excluded(device_data_latest::device_type_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;
        
            info!("Inserted LWL01 water leak data for dev_eui={}", dev_eui_string);
        }
        Some(6) => {
            let parsed: LT22222L = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    // device_data_latest::org_id.eq(device.organization_id.unwrap_or(1)),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(6)),
                    device_data_latest::ro1_status.eq(parsed.ro1_status),
                    device_data_latest::ro2_status.eq(parsed.ro2_status),
                    device_data_latest::gpio_in_1.eq(&parsed.gpio_in_1),
                    device_data_latest::gpio_in_2.eq(&parsed.gpio_in_2),
                    device_data_latest::gpio_out_1.eq(&parsed.gpio_out_1),
                    device_data_latest::gpio_out_2.eq(&parsed.gpio_out_2),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::ro1_status.eq(parsed.ro1_status),
                    device_data_latest::ro2_status.eq(parsed.ro2_status),
                    device_data_latest::gpio_in_1.eq(&parsed.gpio_in_1),
                    device_data_latest::gpio_in_2.eq(&parsed.gpio_in_2),
                    device_data_latest::gpio_out_1.eq(&parsed.gpio_out_1),
                    device_data_latest::gpio_out_2.eq(&parsed.gpio_out_2),
                    // device_data_latest::org_id.eq(device.organization_id.unwrap_or(1)),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(6)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!(
                "Inserted/Updated LT22222L into device_data_latest for dev_eui={}",
                dev_eui_string
            );
        }
        Some(7) => {
            let parsed: LHT65JSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            let temp_raw = parsed.temperature.parse::<f32>()?;
            let hum_raw = parsed.humidity.parse::<f32>()?;

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

            let temperature = BigDecimal::from_f32(temp_raw + temp_cal);
            let humidity = BigDecimal::from_f32(hum_raw + hum_cal);
            let batv = BigDecimal::from_f32(parsed.battery);

            // Insert into device_data_2025
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::air_temperature.eq(temperature.clone()),
                    device_data_2025::air_humidity.eq(humidity.clone()),
                    device_data_2025::batv.eq(batv.clone()),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(7)),
                ))
                .execute(conn)
                .await?;

            // Upsert into device_data_latest
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::air_temperature.eq(temperature),
                    device_data_latest::air_humidity.eq(humidity),
                    device_data_latest::batv.eq(batv),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(7)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::air_temperature
                        .eq(excluded(device_data_latest::air_temperature)),
                    device_data_latest::air_humidity.eq(excluded(device_data_latest::air_humidity)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!("Inserted LHT65 data for dev_eui={}", dev_eui_string);
        }
        Some(8) => {
            let parsed: LAQ4JSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

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

            let temperature = BigDecimal::from_f32(parsed.temperature + temp_cal);
            let humidity = BigDecimal::from_f32(parsed.humidity + hum_cal);
            let co2 = BigDecimal::from_f32(parsed.co2_ppm);
            let tvoc = BigDecimal::from_f32(parsed.tvoc_ppm);
            let batv = BigDecimal::from_f32(parsed.battery);

            // Insert into device_data_2025
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::air_temperature.eq(temperature.clone()),
                    device_data_2025::air_humidity.eq(humidity.clone()),
                    device_data_2025::co2_ppm.eq(co2.clone()),
                    device_data_2025::tvoc_ppm.eq(tvoc.clone()),
                    device_data_2025::batv.eq(batv.clone()),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(8)),
                ))
                .execute(conn)
                .await?;

            // Upsert into device_data_latest
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::air_temperature.eq(temperature),
                    device_data_latest::air_humidity.eq(humidity),
                    device_data_latest::co2_ppm.eq(co2),
                    device_data_latest::tvoc_ppm.eq(tvoc),
                    device_data_latest::batv.eq(batv),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(8)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::air_temperature
                        .eq(excluded(device_data_latest::air_temperature)),
                    device_data_latest::air_humidity.eq(excluded(device_data_latest::air_humidity)),
                    device_data_latest::co2_ppm.eq(excluded(device_data_latest::co2_ppm)),
                    device_data_latest::tvoc_ppm.eq(excluded(device_data_latest::tvoc_ppm)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!("Inserted LAQ4 data for dev_eui={}", dev_eui_string);
        }

        Some(9) => {
            let parsed: LSPH01JSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            if parsed.temp_soil != "0.00" {
                let temp_raw = parsed.temp_soil.parse::<f32>()?;
                let ph_raw = parsed.ph_soil.parse::<f32>()?;

                let temp_cal = device
                    .temperature_calibration
                    .as_ref()
                    .and_then(|v| v.to_f32())
                    .unwrap_or(0.0);

                let sol_temperature = BigDecimal::from_f32(temp_raw + temp_cal);
                let ph_soil = BigDecimal::from_f32(ph_raw);
                let batv = BigDecimal::from_f32(parsed.battery);

                // Insert into device_data_2025
                diesel::insert_into(device_data_2025::table)
                    .values((
                        device_data_2025::dev_eui.eq(&dev_eui_string),
                        device_data_2025::sol_temperature.eq(sol_temperature.clone()),
                        device_data_2025::ph_soil.eq(ph_soil.clone()),
                        device_data_2025::batv.eq(batv.clone()),
                        device_data_2025::device_type_id.eq(device.device_type.unwrap_or(9)),
                    ))
                    .execute(conn)
                    .await?;

                // Upsert into device_data_latest
                diesel::insert_into(device_data_latest::table)
                    .values((
                        device_data_latest::dev_eui.eq(&dev_eui_string),
                        device_data_latest::sol_temperature.eq(sol_temperature),
                        device_data_latest::ph_soil.eq(ph_soil),
                        device_data_latest::batv.eq(batv),
                        device_data_latest::device_type_id.eq(device.device_type.unwrap_or(9)),
                    ))
                    .on_conflict(device_data_latest::dev_eui)
                    .do_update()
                    .set((
                        device_data_latest::sol_temperature
                            .eq(excluded(device_data_latest::sol_temperature)),
                        device_data_latest::ph_soil.eq(excluded(device_data_latest::ph_soil)),
                        device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                        device_data_latest::device_type_id
                            .eq(excluded(device_data_latest::device_type_id)),
                        device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
                    .await?;

                info!(
                    "Inserted LSPH01 soil pH data for dev_eui={}",
                    dev_eui_string
                );
            }
        }

        // Some(11) => {
        //     let parsed: SensecapMessage = serde_json::from_value(object_json.clone())?;
        //     let dev_eui_string = device.dev_eui.to_string();

        //     if let Some(light) = parsed.messages.first() {
        //         if light.r#type == "report_telemetry" {
        //             let sensecap_light = BigDecimal::from_f32(light.measurement_value);

        //             // Insert into device_data_2025
        //             diesel::insert_into(device_data_2025::table)
        //                 .values((
        //                     device_data_2025::dev_eui.eq(&dev_eui_string),
        //                     device_data_2025::sensecap_light.eq(sensecap_light.clone()),
        //                     device_data_2025::device_type_id.eq(device.device_type.unwrap_or(11)),
        //                     // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
        //                 ))
        //                 .execute(conn)
        //                 .await?;

        //             // Upsert into device_data_latest
        //             diesel::insert_into(device_data_latest::table)
        //                 .values((
        //                     device_data_latest::dev_eui.eq(&dev_eui_string),
        //                     device_data_latest::sensecap_light.eq(sensecap_light),
        //                     device_data_latest::device_type_id.eq(device.device_type.unwrap_or(11)),
        //                 ))
        //                 .on_conflict(device_data_latest::dev_eui)
        //                 .do_update()
        //                 .set((
        //                     device_data_latest::sensecap_light.eq(excluded(device_data_latest::sensecap_light)),
        //                     device_data_latest::device_type_id.eq(excluded(device_data_latest::device_type_id)),
        //                     device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
        //                 ))
        //                 .execute(conn)
        //                 .await?;

        //             info!("Inserted SenseCAP light telemetry for dev_eui={}", dev_eui_string);
        //         } else {
        //             tracing::warn!("Unexpected message type: {:?}", light.r#type);
        //         }
        //     } else {
        //         tracing::warn!("No messages found in SenseCAP payload");
        //     }
        // }
        Some(12) => {
            let parsed: EM300THJSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            // Skip zeroed payloads
            if parsed.temperature == 0.0 && parsed.humidity == 0.0 {
                info!("EM300TH has zeroed data, skipping...");
                return Ok(());
            }

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

            let temperature = BigDecimal::from_f32(parsed.temperature + temp_cal);
            let humidity = BigDecimal::from_f32(parsed.humidity + hum_cal);
            let batv = BigDecimal::from_f32(parsed.battery);

            // Insert into device_data_2025
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::air_temperature.eq(temperature.clone()),
                    device_data_2025::air_humidity.eq(humidity.clone()),
                    device_data_2025::batv.eq(batv.clone()),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(12)),
                    // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .execute(conn)
                .await?;

            // Upsert into device_data_latest
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::air_temperature.eq(temperature),
                    device_data_latest::air_humidity.eq(humidity),
                    device_data_latest::batv.eq(batv),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(12)),
                    // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::air_temperature
                        .eq(excluded(device_data_latest::air_temperature)),
                    device_data_latest::air_humidity.eq(excluded(device_data_latest::air_humidity)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!(
                "Inserted EM300TH temperature & humidity data for dev_eui={}",
                dev_eui_string
            );
        }

        Some(13) => {
            let parsed: AM107JSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

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

            let temperature = BigDecimal::from_f32(parsed.temperature + temp_cal);
            let humidity = BigDecimal::from_f32(parsed.humidity + hum_cal);
            let co2 = BigDecimal::from_f32(parsed.co2);
            let tvoc = BigDecimal::from_f32(parsed.tvoc);
            let pressure = BigDecimal::from_f32(parsed.pressure);
            let batv = BigDecimal::from_f32(parsed.battery);

            // Insert into device_data_2025
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::air_temperature.eq(temperature.clone()),
                    device_data_2025::air_humidity.eq(humidity.clone()),
                    device_data_2025::co2_ppm.eq(co2.clone()),
                    device_data_2025::tvoc_ppm.eq(tvoc.clone()),
                    device_data_2025::barometric_pressure.eq(pressure.clone()),
                    device_data_2025::batv.eq(batv.clone()),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(13)),
                    // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .execute(conn)
                .await?;

            // Upsert into device_data_latest
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::air_temperature.eq(temperature),
                    device_data_latest::air_humidity.eq(humidity),
                    device_data_latest::co2_ppm.eq(co2),
                    device_data_latest::tvoc_ppm.eq(tvoc),
                    device_data_latest::barometric_pressure.eq(pressure),
                    device_data_latest::batv.eq(batv),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(13)),
                    // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::air_temperature
                        .eq(excluded(device_data_latest::air_temperature)),
                    device_data_latest::air_humidity.eq(excluded(device_data_latest::air_humidity)),
                    device_data_latest::co2_ppm.eq(excluded(device_data_latest::co2_ppm)),
                    device_data_latest::tvoc_ppm.eq(excluded(device_data_latest::tvoc_ppm)),
                    device_data_latest::barometric_pressure
                        .eq(excluded(device_data_latest::barometric_pressure)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!(
                "Inserted AM107 air quality data for dev_eui={}",
                dev_eui_string
            );
        }

        Some(14) => {
            let parsed: WS101JSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            // Store alarm as water_leak_status (DB compatibility)
            let alarm_status = parsed.alarm;

            // Insert into device_data_2025
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::water_leak_status.eq(alarm_status),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(14)),
                    // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .execute(conn)
                .await?;

            // Upsert into device_data_latest
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::water_leak_status.eq(alarm_status),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(14)),
                    // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::water_leak_status
                        .eq(excluded(device_data_latest::water_leak_status)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!(
                "Inserted WS101 alarm press event for dev_eui={}",
                dev_eui_string
            );
        }
        Some(16) => {
            let parsed: EM300MCS = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            let batv = BigDecimal::from_f32(parsed.battery);
            let door_status = parsed.door_status;

            // Insert into device_data_2025
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::door_open_status.eq(door_status),
                    device_data_2025::batv.eq(batv.clone()),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(16)),
                    // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .execute(conn)
                .await?;

            // Upsert into device_data_latest
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::door_open_status.eq(door_status),
                    device_data_latest::batv.eq(batv),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(16)),
                    // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::door_open_status
                        .eq(excluded(device_data_latest::door_open_status)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!(
                "Inserted EM300-MCS door sensor data for dev_eui={}",
                dev_eui_string
            );
        }

        Some(18) => {
            let parsed: EM300ZLDJSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            // Insert into device_data_2025
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::water_leak_status.eq(parsed.water_leak),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(18)),
                    // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .execute(conn)
                .await?;

            // Upsert into device_data_latest
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::water_leak_status.eq(parsed.water_leak),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(18)),
                    // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::water_leak_status
                        .eq(excluded(device_data_latest::water_leak_status)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!(
                "Inserted EM300-ZLD leak status for dev_eui={}",
                dev_eui_string
            );
        }

        Some(19) => {
            let parsed: EM300ZLDJSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::water_leak_status.eq(parsed.water_leak),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(19)),
                    // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .execute(conn)
                .await?;

            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::water_leak_status.eq(parsed.water_leak),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(19)),
                    // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::water_leak_status
                        .eq(excluded(device_data_latest::water_leak_status)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!(
                "Inserted EM300-ZLD (device_type=19) water leak status for dev_eui={}",
                dev_eui_string
            );
        }
        Some(20) => {
            let parsed: EM500PT100JSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            let temp_cal = device
                .temperature_calibration
                .as_ref()
                .and_then(|v| v.to_f32())
                .unwrap_or(0.0);
            let temperature = BigDecimal::from_f32(parsed.temperature + temp_cal);
            let batv = BigDecimal::from_f32(parsed.battery);

            // Insert into device_data_2025
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::air_temperature.eq(temperature.clone()),
                    device_data_2025::batv.eq(batv.clone()),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(20)),
                    // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .execute(conn)
                .await?;

            // Upsert into device_data_latest
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::air_temperature.eq(temperature),
                    device_data_latest::batv.eq(batv),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(20)),
                    // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::air_temperature
                        .eq(excluded(device_data_latest::air_temperature)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!(
                "Inserted EM500-PT100 temp data for dev_eui={}",
                dev_eui_string
            );
        }

        Some(21) => {
            let parsed: EM500PPJSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            // Convert Pa to hPa (divide by 100.0)
            let pressure_hpa = parsed.pressure / 100.0;

            let barometric_pressure = BigDecimal::from_f32(pressure_hpa);
            let batv = BigDecimal::from_f32(parsed.battery);

            // Insert into device_data_2025
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::barometric_pressure.eq(barometric_pressure.clone()),
                    device_data_2025::batv.eq(batv.clone()),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(21)),
                    // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .execute(conn)
                .await?;

            // Upsert into device_data_latest
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::barometric_pressure.eq(barometric_pressure),
                    device_data_latest::batv.eq(batv),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(21)),
                    // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::barometric_pressure
                        .eq(excluded(device_data_latest::barometric_pressure)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!(
                "Inserted EM500-PP barometric pressure for dev_eui={}",
                dev_eui_string
            );
        }

        Some(24) => {
            let parsed: WS522JSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            let current = BigDecimal::from_f32(parsed.current);
            let factor = BigDecimal::from_f32(parsed.factor);
            let power = BigDecimal::from_f32(parsed.power);
            let voltage = BigDecimal::from_f32(parsed.voltage);
            let power_sum = BigDecimal::from_f32(parsed.power_sum);
            let status = parsed.state;

            if parsed.current > 0.0 {
                // Insert into WS522 table (optional - skip if not defined in your schema)
                // Insert into device_data_latest
                diesel::insert_into(device_data_latest::table)
                    .values((
                        device_data_latest::dev_eui.eq(&dev_eui_string),
                        device_data_latest::current.eq(current),
                        device_data_latest::factor.eq(factor),
                        device_data_latest::power.eq(power),
                        device_data_latest::voltage.eq(voltage),
                        device_data_latest::power_sum.eq(power_sum),
                        device_data_latest::status.eq(status),
                        device_data_latest::device_type_id.eq(device.device_type.unwrap_or(24)),
                        // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                    ))
                    .on_conflict(device_data_latest::dev_eui)
                    .do_update()
                    .set((
                        device_data_latest::current.eq(excluded(device_data_latest::current)),
                        device_data_latest::factor.eq(excluded(device_data_latest::factor)),
                        device_data_latest::power.eq(excluded(device_data_latest::power)),
                        device_data_latest::voltage.eq(excluded(device_data_latest::voltage)),
                        device_data_latest::power_sum.eq(excluded(device_data_latest::power_sum)),
                        device_data_latest::status.eq(excluded(device_data_latest::status)),
                        device_data_latest::device_type_id
                            .eq(excluded(device_data_latest::device_type_id)),
                        // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                        device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
                    .await?;

                info!(
                    "WS522: full power data inserted for dev_eui={}",
                    dev_eui_string
                );
            } else {
                // Only update `status`
                diesel::update(
                    device_data_latest::table
                        .filter(device_data_latest::dev_eui.eq(&dev_eui_string)),
                )
                .set((
                    device_data_latest::status.eq(status),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

                info!("WS522: only status updated for dev_eui={}", dev_eui_string);
            }
        }

        Some(27) => {
            let parsed: WS558Json = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            let power = BigDecimal::from_f32(parsed.active_power);
            let power_consumption = BigDecimal::from_f32(parsed.power_consumption);
            let factor = BigDecimal::from_f32(parsed.power_factor);
            let current = BigDecimal::from_f32(parsed.total_current);
            let voltage = BigDecimal::from_f32(parsed.voltage);

            // If we have power factor, insert full record
            if parsed.power_factor > 0.0 {
                diesel::insert_into(device_data_latest::table)
                    .values((
                        device_data_latest::dev_eui.eq(&dev_eui_string),
                        device_data_latest::power.eq(power),
                        device_data_latest::power_sum.eq(power_consumption),
                        device_data_latest::factor.eq(factor),
                        device_data_latest::current.eq(current),
                        device_data_latest::voltage.eq(voltage),
                        device_data_latest::switch1.eq(parsed.switch1),
                        device_data_latest::switch2.eq(parsed.switch2),
                        device_data_latest::switch3.eq(parsed.switch3),
                        device_data_latest::switch4.eq(parsed.switch4),
                        device_data_latest::switch5.eq(parsed.switch5),
                        device_data_latest::switch6.eq(parsed.switch6),
                        device_data_latest::switch7.eq(parsed.switch7),
                        device_data_latest::switch8.eq(parsed.switch8),
                        device_data_latest::device_type_id.eq(device.device_type.unwrap_or(27)),
                        // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                    ))
                    .on_conflict(device_data_latest::dev_eui)
                    .do_update()
                    .set((
                        device_data_latest::power.eq(excluded(device_data_latest::power)),
                        device_data_latest::power_sum.eq(excluded(device_data_latest::power_sum)),
                        device_data_latest::factor.eq(excluded(device_data_latest::factor)),
                        device_data_latest::current.eq(excluded(device_data_latest::current)),
                        device_data_latest::voltage.eq(excluded(device_data_latest::voltage)),
                        device_data_latest::switch1.eq(excluded(device_data_latest::switch1)),
                        device_data_latest::switch2.eq(excluded(device_data_latest::switch2)),
                        device_data_latest::switch3.eq(excluded(device_data_latest::switch3)),
                        device_data_latest::switch4.eq(excluded(device_data_latest::switch4)),
                        device_data_latest::switch5.eq(excluded(device_data_latest::switch5)),
                        device_data_latest::switch6.eq(excluded(device_data_latest::switch6)),
                        device_data_latest::switch7.eq(excluded(device_data_latest::switch7)),
                        device_data_latest::switch8.eq(excluded(device_data_latest::switch8)),
                        device_data_latest::device_type_id
                            .eq(excluded(device_data_latest::device_type_id)),
                        // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                        device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
                    .await?;

                info!(
                    "Inserted WS558 full power and switch data for dev_eui={}",
                    dev_eui_string
                );
            } else {
                // Power data missing — update only switches
                diesel::update(
                    device_data_latest::table
                        .filter(device_data_latest::dev_eui.eq(&dev_eui_string)),
                )
                .set((
                    device_data_latest::switch1.eq(parsed.switch1),
                    device_data_latest::switch2.eq(parsed.switch2),
                    device_data_latest::switch3.eq(parsed.switch3),
                    device_data_latest::switch4.eq(parsed.switch4),
                    device_data_latest::switch5.eq(parsed.switch5),
                    device_data_latest::switch6.eq(parsed.switch6),
                    device_data_latest::switch7.eq(parsed.switch7),
                    device_data_latest::switch8.eq(parsed.switch8),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

                info!(
                    "WS558: updated switches only for dev_eui={}",
                    dev_eui_string
                );
            }
        }

        Some(28) => {
            let parsed: UC300Json = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            // Update device_data_latest with full GPIO/ADC state
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::adc_1.eq(parsed.adc_1.clone()),
                    device_data_latest::adc_2.eq(parsed.adc_2.clone()),
                    device_data_latest::adv_1.eq(parsed.adv_1.clone()),
                    device_data_latest::gpio_in_1.eq(parsed.gpio_in_1.clone()),
                    device_data_latest::gpio_in_2.eq(parsed.gpio_in_2.clone()),
                    device_data_latest::gpio_in_3.eq(parsed.gpio_in_3.clone()),
                    device_data_latest::gpio_in_4.eq(parsed.gpio_in_4.clone()),
                    device_data_latest::gpio_out_1.eq(parsed.gpio_out_1.clone()),
                    device_data_latest::gpio_out_2.eq(parsed.gpio_out_2.clone()),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(28)),
                    // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::adc_1.eq(excluded(device_data_latest::adc_1)),
                    device_data_latest::adc_2.eq(excluded(device_data_latest::adc_2)),
                    device_data_latest::adv_1.eq(excluded(device_data_latest::adv_1)),
                    device_data_latest::gpio_in_1.eq(excluded(device_data_latest::gpio_in_1)),
                    device_data_latest::gpio_in_2.eq(excluded(device_data_latest::gpio_in_2)),
                    device_data_latest::gpio_in_3.eq(excluded(device_data_latest::gpio_in_3)),
                    device_data_latest::gpio_in_4.eq(excluded(device_data_latest::gpio_in_4)),
                    device_data_latest::gpio_out_1.eq(excluded(device_data_latest::gpio_out_1)),
                    device_data_latest::gpio_out_2.eq(excluded(device_data_latest::gpio_out_2)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!(
                "Inserted UC300 GPIO/ADC state for dev_eui={}",
                dev_eui_string
            );
        }

        Some(33) => {
            let parsed: EM400MUD = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            if parsed.distance > 0 {
                let temperature = BigDecimal::from_f32(parsed.temperature);
                let batv = Some(parsed.battery as i32); // 👈 convert to correct type
                let distance = Some(parsed.distance as i32);

                // Insert into device_data_2025
                diesel::insert_into(em400mud::table)
                    .values((
                        em400mud::dev_eui.eq(&dev_eui_string),
                        em400mud::air_temperature.eq(temperature.clone()),
                        em400mud::batv.eq(batv),
                        em400mud::distance.eq(distance),
                        em400mud::device_type_id.eq(device.device_type.unwrap_or(33)),
                        // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
                        em400mud::position.eq(parsed.position.clone()),
                    ))
                    .execute(conn)
                    .await?;

                // Upsert into device_data_latest
                // diesel::insert_into(device_data_latest::table)
                //     .values((
                //         device_data_latest::dev_eui.eq(&dev_eui_string),
                //         device_data_latest::air_temperature.eq(temperature),
                //         device_data_latest::batv.eq(batv),
                //         device_data_latest::distance.eq(distance),
                //         device_data_latest::device_type_id.eq(device.device_type.unwrap_or(33)),
                //         // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                //         device_data_latest::position.eq(parsed.position.clone()),
                //     ))
                //     .on_conflict(device_data_latest::dev_eui)
                //     .do_update()
                //     .set((
                //         device_data_latest::air_temperature
                //             .eq(excluded(device_data_latest::air_temperature)),
                //         device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                //         // device_data_latest::distance.eq(excluded(device_data_latest::distance)),
                //         // device_data_latest::position.eq(excluded(device_data_latest::position)),
                //         device_data_latest::device_type_id
                //             .eq(excluded(device_data_latest::device_type_id)),
                //         // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                //         device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                //     ))
                //     .execute(conn)
                //     .await?;

                info!("Inserted EM400-MUD data for dev_eui={}", dev_eui_string);
            } else {
                info!(
                    "EM400-MUD: Ignored reading with distance <= 0 for dev_eui={}",
                    dev_eui_string
                );
            }
        }

        Some(35) => {
            let parsed: AM103Json = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            if parsed.temperature != 0.0 && parsed.humidity != 0.0 {
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

                let temperature = BigDecimal::from_f32(parsed.temperature + temp_cal);
                let humidity = BigDecimal::from_f32(parsed.humidity + hum_cal);
                let co2 = BigDecimal::from_f32(parsed.co2);
                let batv = BigDecimal::from_i64(parsed.battery).and_then(|v| v.to_i32());

                // Insert into device_data_2025
                diesel::insert_into(am103::table)
                    .values((
                        am103::dev_eui.eq(&dev_eui_string),
                        am103::air_temperature.eq(temperature.clone()),
                        am103::air_humidity.eq(humidity.clone()),
                        am103::co2_ppm.eq(co2.clone()),
                        am103::batv.eq(batv.clone()),
                        am103::device_type_id.eq(device.device_type.unwrap_or(35)),
                        // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
                    ))
                    .execute(conn)
                    .await?;

                // Upsert into device_data_latest
                // diesel::insert_into(device_data_latest::table)
                //     .values((
                //         device_data_latest::dev_eui.eq(&dev_eui_string),
                //         device_data_latest::air_temperature.eq(temperature),
                //         device_data_latest::air_humidity.eq(humidity),
                //         device_data_latest::co2_ppm.eq(co2),
                //         device_data_latest::batv.eq(batv),
                //         device_data_latest::device_type_id.eq(device.device_type.unwrap_or(35)),
                //         // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                //     ))
                //     .on_conflict(device_data_latest::dev_eui)
                //     .do_update()
                //     .set((
                //         device_data_latest::air_temperature
                //             .eq(excluded(device_data_latest::air_temperature)),
                //         device_data_latest::air_humidity
                //             .eq(excluded(device_data_latest::air_humidity)),
                //         device_data_latest::co2_ppm.eq(excluded(device_data_latest::co2_ppm)),
                //         device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                //         device_data_latest::device_type_id
                //             .eq(excluded(device_data_latest::device_type_id)),
                //         // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                //         device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                //     ))
                //     .execute(conn)
                //     .await?;

                info!(
                    "Inserted AM103 air quality data for dev_eui={}",
                    dev_eui_string
                );
            } else {
                info!(
                    "Skipped AM103 reading due to zero temperature/humidity for dev_eui={}",
                    dev_eui_string
                );
            }
        }

        Some(36) => {
            let parsed: LTC2LB = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            let temp_cal = device
                .temperature_calibration
                .as_ref()
                .and_then(|v| v.to_f32())
                .unwrap_or(0.0);
            let temperature1 = BigDecimal::from_f32(parsed.temperature1 + temp_cal);
            let temperature2 = BigDecimal::from_f32(parsed.temperature2); // optional to calibrate
            let batv = BigDecimal::from_f32(parsed.battery);

            // Insert into device_data_2025
            diesel::insert_into(ltc2lb::table)
                .values((
                    ltc2lb::dev_eui.eq(&dev_eui_string),
                    ltc2lb::temperature1.eq(temperature1.clone()),
                    ltc2lb::temperature2.eq(temperature2.clone()),
                    ltc2lb::batv.eq(batv.clone()),
                    ltc2lb::device_type_id.eq(device.device_type.unwrap_or(36)),
                    // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .execute(conn)
                .await?;

            // Upsert into device_data_latest
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::air_temperature.eq(temperature1),
                    device_data_latest::sol_temperature.eq(temperature2),
                    device_data_latest::batv.eq(batv),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(36)),
                    // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::air_temperature
                        .eq(excluded(device_data_latest::air_temperature)),
                    device_data_latest::sol_temperature
                        .eq(excluded(device_data_latest::sol_temperature)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!(
                "Inserted LTC2LB dual-temp data for dev_eui={}",
                dev_eui_string
            );
        }

        Some(37) => {
            let parsed: DDS45LB = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();

            let distance: Option<i32> = BigDecimal::from_i64(parsed.distance)
                .and_then(|v| v.to_i64())
                .and_then(|v| i32::try_from(v).ok());

            let batv = BigDecimal::from_f32(parsed.battery);

            // Insert into device_data_2025
            diesel::insert_into(dds45lb::table)
                .values((
                    dds45lb::dev_eui.eq(&dev_eui_string),
                    dds45lb::distance.eq(distance),
                    dds45lb::batv.eq(batv.clone()),
                    dds45lb::device_type_id.eq(device.device_type.unwrap_or(37)),
                    // device_data_2025::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .execute(conn)
                .await?;

            // Upsert into device_data_latest
            diesel::insert_into(device_data_latest::table)
                .values((
                    device_data_latest::dev_eui.eq(&dev_eui_string),
                    device_data_latest::distance.eq(distance),
                    device_data_latest::batv.eq(batv),
                    device_data_latest::device_type_id.eq(device.device_type.unwrap_or(37)),
                    // device_data_latest::org_id.eq(device.org_id.unwrap_or(1)),
                ))
                .on_conflict(device_data_latest::dev_eui)
                .do_update()
                .set((
                    device_data_latest::distance.eq(excluded(device_data_latest::distance)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id
                        .eq(excluded(device_data_latest::device_type_id)),
                    // device_data_latest::org_id.eq(excluded(device_data_latest::org_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;

            info!(
                "Inserted DDS45LB distance + battery for dev_eui={}",
                dev_eui_string
            );
        }

        _ => {
            tracing::warn!("Unsupported device type: {:?}", device.device_type);
        }
    }

    Ok(())
}
