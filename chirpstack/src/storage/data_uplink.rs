use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::upsert::excluded;
use diesel::Insertable;
use diesel::Queryable;
use diesel::Identifiable;
use diesel_async::RunQueryDsl;
use serde_json::to_string_pretty;
use serde_json::Value;
use tracing::info;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive}; // ✅ use correct BigDecimal crate
use serde::Deserialize;

use crate::storage::device::Device;
// ⚠️ no `use crate::storage::fields::*;` here to avoid name conflicts
use crate::storage::schema::{device_data_2025, device_data_latest};



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
                batv: BigDecimal::from_f32(parsed.batv),
                // org_id: org_id as i32,
                device_type_id: device.device_type.unwrap_or(1),
            };

            diesel::insert_into(device_data_2025::table)
                .values(&new_data)
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
        
                let new_data = diesel::insert_into(device_data_2025::table)
                    .values((
                        device_data_2025::dev_eui.eq(&dev_eui_string),
                        device_data_2025::sol_temperature.eq(temp),
                        device_data_2025::sol_water.eq(water),
                        device_data_2025::sol_conduct_soil.eq(conduct),
                        device_data_2025::batv.eq(batv),
                        device_data_2025::device_type_id.eq(device.device_type.unwrap_or(2)),
                    ))
                    .execute(conn)
                    .await?;
        
                info!("Inserted LSE01 data for dev_eui={}", dev_eui_string);
            }
        }
        Some(3) => {
            let parsed: LDS01JSON = serde_json::from_value(object_json.clone())?;
        
            // let batv = BigDecimal::from_f32(parsed.battery);
            let dev_eui_string = device.dev_eui.to_string();
        
            diesel::insert_into(device_data_2025::table)
                .values((
                    device_data_2025::dev_eui.eq(&dev_eui_string),
                    device_data_2025::door_open_status.eq(parsed.door_status as i32),
                    device_data_2025::door_open_times.eq(parsed.door_open_times as i32),
                    device_data_2025::last_door_open_duration.eq(parsed.last_door_open_duration as i32),
                    // device_data_2025::batv.eq(batv),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(3)),
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
                    device_data_2025::batv.eq(batv),
                    device_data_2025::device_type_id.eq(device.device_type.unwrap_or(4)),
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
        
            info!("Inserted/Updated LT22222L into device_data_latest for dev_eui={}", dev_eui_string);
        }
        Some(7) => {
            let parsed: LHT65JSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();
        
            let temp_raw = parsed.temperature.parse::<f32>()?;
            let hum_raw = parsed.humidity.parse::<f32>()?;
        
            let temp_cal = device.temperature_calibration.as_ref().and_then(|v| v.to_f32()).unwrap_or(0.0);
            let hum_cal = device.humadity_calibration.as_ref().and_then(|v| v.to_f32()).unwrap_or(0.0);
        
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
                    device_data_latest::air_temperature.eq(excluded(device_data_latest::air_temperature)),
                    device_data_latest::air_humidity.eq(excluded(device_data_latest::air_humidity)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id.eq(excluded(device_data_latest::device_type_id)),
                    device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .await?;
        
            info!("Inserted LHT65 data for dev_eui={}", dev_eui_string);
        }
        Some(8) => {
            let parsed: LAQ4JSON = serde_json::from_value(object_json.clone())?;
            let dev_eui_string = device.dev_eui.to_string();
        
            let temp_cal = device.temperature_calibration.as_ref().and_then(|v| v.to_f32()).unwrap_or(0.0);
            let hum_cal = device.humadity_calibration.as_ref().and_then(|v| v.to_f32()).unwrap_or(0.0);
        
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
                    device_data_latest::air_temperature.eq(excluded(device_data_latest::air_temperature)),
                    device_data_latest::air_humidity.eq(excluded(device_data_latest::air_humidity)),
                    device_data_latest::co2_ppm.eq(excluded(device_data_latest::co2_ppm)),
                    device_data_latest::tvoc_ppm.eq(excluded(device_data_latest::tvoc_ppm)),
                    device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                    device_data_latest::device_type_id.eq(excluded(device_data_latest::device_type_id)),
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
        
                let temp_cal = device.temperature_calibration.as_ref().and_then(|v| v.to_f32()).unwrap_or(0.0);
        
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
                        device_data_latest::sol_temperature.eq(excluded(device_data_latest::sol_temperature)),
                        device_data_latest::ph_soil.eq(excluded(device_data_latest::ph_soil)),
                        device_data_latest::batv.eq(excluded(device_data_latest::batv)),
                        device_data_latest::device_type_id.eq(excluded(device_data_latest::device_type_id)),
                        device_data_latest::submission_date.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
                    .await?;
        
                info!("Inserted LSPH01 soil pH data for dev_eui={}", dev_eui_string);
            }
        }

        
        _ => {
            tracing::warn!("Unsupported device type: {:?}", device.device_type);
        }
    }

    Ok(())
}
