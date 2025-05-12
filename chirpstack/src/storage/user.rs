use anyhow::Result;
use chirpstack_api::api::GetLandingResponse;
use chirpstack_api::api::LandingOrganization;
use chirpstack_api::api::LandingZoneList;
use chirpstack_api::api::LandingAlarm;
use chirpstack_api::api::LandingDevice;
use chirpstack_api::api::LandingDeviceProfile;
use chirpstack_api::api::LandingZone;
use chirpstack_api::api::LandingOrganizationList;
use diesel::sql_query;
use chrono::{DateTime, Utc};
use diesel::{dsl, prelude::*};
use diesel_async::RunQueryDsl;
use diesel::sql_types::{Text, Nullable, Uuid as SqlUuid};
use email_address::EmailAddress;
use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Pbkdf2,
};
use rand_core::OsRng;
use tracing::info;
use uuid::Uuid;
use serde::{Deserialize};
use super::error::Error;
use super::schema::user;
use super::{fields, get_async_db_conn};
use tonic::Status;

#[derive(Queryable, Insertable, PartialEq, Eq, Debug, Clone)]
#[diesel(table_name = user)]
pub struct User {
    pub id: fields::Uuid,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_admin: bool,
    pub is_active: bool,
    pub email: String,
    pub email_verified: bool,
    pub password_hash: String,
    pub note: String,
    pub android_key: Option<String>,
    pub phone_number: String,
    pub name: Option<String>,
    pub username: Option<String>,
    pub zone_id_list: Option<Vec<Option<i64>>>,
    pub training: bool,
    pub expo_key: Option<String>,
    pub web_key: Option<String>,

}

#[derive(Debug, Deserialize)]
pub struct SerdeLandingZoneList {
    pub zones: Vec<SerdeLandingZone>,
}

#[derive(Debug, Deserialize)]
pub struct SerdeLandingZone {
    pub zone_id: i64,
    pub zone_name: String,
    pub org_id: i64,
    pub order: i64,
    pub contentType: i64,
    pub devices: Vec<SerdeLandingDevice>,
}

#[derive(Debug, Deserialize)]
pub struct SerdeLandingDevice {
    pub device_dev_eui: String,
    pub device_created_at: String,
    pub device_updated_at: String,
    pub device_profile_id: String,
    pub device_name: String,
    pub device_description: String,
    pub device_last_seen_at: String,
    pub device_data_time: i64,
    pub device_lat: f64,
    pub device_lng: f64,
    pub device_application_id: i64,
    pub alerts: Vec<SerdeLandingAlarm>,
    pub device_profile_name: Vec<SerdeLandingDeviceProfile>,
}

#[derive(Debug, Deserialize)]
pub struct SerdeLandingDeviceProfile {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SerdeLandingAlarm {
    pub id: i64,
    pub dev_eui: String,
    pub min_treshold: Option<f64>,
    pub max_treshold: Option<f64>,
    pub temperature: Option<bool>,
    pub humadity: Option<bool>,
    pub ec: Option<bool>,
    pub door: Option<bool>,
    pub w_leak: Option<bool>,
    pub sms: Option<bool>,
    pub email: Option<bool>,
}

impl Default for User {
    fn default() -> Self {
        let now = Utc::now();

        User {
            id: Uuid::new_v4().into(),
            external_id: None,
            created_at: now,
            updated_at: now,
            is_admin: false,
            is_active: false,
            email: "".into(),
            email_verified: false,
            password_hash: "".into(),
            note: "".into(),
            android_key: None,
            phone_number: "".into(),
            name: None,
            username: None,
            zone_id_list: None,
            training: false,
            expo_key: None,
            web_key: None,
        }
    }
}
#[derive(Debug, Deserialize)]
pub struct GetLandingResponseSerde {
    pub id: i64,
    pub email: String,
    pub is_active: bool,
    pub web_key: String,
    pub ios_key: String,
    pub android_key: String,
    pub phone_number: String,
    pub name: String,
    pub note: String,
    pub username: String,
    pub training: bool,
    pub organization_id_list: Vec<i64>,
    pub organizationList: SerdeLandingOrganizationList,
    pub zoneList: SerdeLandingZoneList,
}

#[derive(Debug, Deserialize)]
pub struct SerdeLandingOrganizationList {
    pub organizations: Vec<SerdeLandingOrganization>,
}

#[derive(Debug, Deserialize)]
pub struct SerdeLandingOrganization {
    pub organization_id: i64,
    pub organization_name: String,
    pub organization_display_name: String,
}
impl User {
    pub fn validate(&self) -> Result<(), Error> {
        if self.email != "admin" && !EmailAddress::is_valid(&self.email) {
            return Err(Error::InvalidEmail);
        }

        Ok(())
    }

    pub fn set_password_hash(&mut self, pw: &str, rounds: u32) -> Result<(), Error> {
        self.password_hash = hash_password(pw, rounds)?;
        Ok(())
    }
}

pub async fn create(u: User) -> Result<User, Error> {
    u.validate()?;

    let u: User = diesel::insert_into(user::table)
        .values(&u)
        .get_result(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, u.id.to_string()))?;
    info!(id = %u.id, "User created");
    Ok(u)
}

pub async fn get(id: &Uuid) -> Result<User, Error> {
    let u = user::dsl::user
        .find(&fields::Uuid::from(id))
        .first(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, id.to_string()))?;
    Ok(u)
}

pub async fn get_by_email(email: &str) -> Result<User, Error> {
    let u = user::dsl::user
        .filter(user::dsl::email.eq(email))
        .first(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, email.to_string()))?;
    Ok(u)
}

pub async fn get_by_external_id(external_id: &str) -> Result<User, Error> {
    let u = user::dsl::user
        .filter(user::dsl::external_id.eq(external_id))
        .first(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, external_id.to_string()))?;
    Ok(u)
}

pub async fn get_by_email_and_pw(username: &str, pw: &str) -> Result<User, Error> {
    let u: User = match user::dsl::user
        .filter(user::dsl::username.eq(username))
        .first(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, username.to_string()))
    {
        Ok(v) => v,
        Err(Error::NotFound(_)) => {
            return Err(Error::InvalidUsernameOrPassword);
        }
        Err(v) => {
            return Err(v);
        }
    };

    if verify_password(pw, &u.password_hash) {
        return Ok(u);
    }

    Err(Error::InvalidUsernameOrPassword)
}

pub async fn get_login(userid: &Uuid) -> Result<GetLandingResponse, Status> {
      let mut query = r#"   WITH device_data_2025 AS (
    SELECT 
        dev.dev_eui,
        json_build_object(
            'device_dev_eui', dev.dev_eui,
            'device_name', dev.name,
            'device_description', dev.description,
            'tags', dev.tags,
            'variables', dev.variables,
            'device_data_time', dev.data_time,
            'device_created_at', dev.created_at,
            'device_updated_at', dev.updated_at,
            'device_profile_id', dev.device_profile_id,
            'device_last_seen_at', dev.last_seen_at,
            'latitude', dev.latitude,
            'longitude', dev.longitude,
            'device_application_id', dev.application_id,
            'device_lat', dev.lat,
            'device_lng', dev.lng,
            'device_profile_name', COALESCE(array_agg(dp.list), ARRAY[]::json[]),
            'alerts', COALESCE(array_agg(al.list), ARRAY[]::json[])
        ) AS device_json
    FROM public.device AS dev
    LEFT JOIN (
        SELECT dp.device_profile_id,
               json_build_object('name', dp.name) AS list
        FROM public.device_profile dp
        GROUP BY dp.device_profile_id
    ) dp ON dev.device_profile_id = dp.device_profile_id
    LEFT JOIN (
        SELECT al.id, al.dev_eui,
               json_build_object(
                   'id', al.id,
                   'dev_eui', al.dev_eui,
                   'min_treshold', al.min_treshold,
                   'max_treshold', al.max_treshold,
                   'temperature', al.temperature,
                   'humadity', al.humadity,
                   'ec', al.ec,
                   'door', al.door,
                   'w_leak', al.w_leak,
                   'sms', al.sms,
                   'email', al.email
               ) AS list
        FROM public.alarm_refactor2 al
        WHERE al.is_active = true
        GROUP BY al.id
    ) al ON al.dev_eui = encode(dev.dev_eui, 'hex')
    GROUP BY dev.dev_eui
),
zone_data AS (
    SELECT 
        z.zone_id,
        json_build_object(
            'zone_id', z.zone_id,
            'zone_name', z.zone_name,
            'org_id', z.org_id,
            'order', z.zone_order,
            'contentType', z.content_type,
            'devices', COALESCE(array_agg(dd.device_json), ARRAY[]::json[])
        ) AS list
    FROM public.zone z
    LEFT JOIN public.device d ON d.dev_eui::text = ANY(z.devices)
    LEFT JOIN device_data_2025 dd ON d.dev_eui = dd.dev_eui
    GROUP BY z.zone_id
)
SELECT json_build_object(
    'id', u.id,
    'email', u.email,
    'is_active', u.is_active,
    'web_key', u.web_key,
    'ios_key', u.ios_key,
    'android_key', u.android_key,
    'phone_number', u.phone_number,
    'name', u.name,
    'note', u.note,
    'username', u.username,
    'training', u.training,
    'organization_id_list', (
        SELECT array_agg(ou.organization_id)
        FROM organization_user ou
        WHERE ou.user_id = u.id
    ),
    'organizationList', json_build_object(
        'organizations', (
            SELECT array_agg(
                json_build_object(
                    'organization_id', org.id,
                    'organization_name', org.name,
                    'organization_display_name', org.display_name
                )
            )
            FROM organization_user ou
            JOIN organization org ON org.id = ou.organization_id
            WHERE ou.user_id = u.id
        )
    ),
    'zoneList', json_build_object(
        'zones', (
            SELECT array_agg(zd.list)
            FROM zone_data zd
            WHERE zd.zone_id = ANY(u.zone_id_list)
        )
    )
) AS login_response
FROM public.user u
WHERE u.id = $1;
    "#.to_string();


   let conn = &mut get_async_db_conn().await.map_err(|e| {
        Status::internal(format!("DB connection failed: {e}"))
    })?;

     #[derive(QueryableByName)]
    struct LoginRow {
        #[sql_type = "Text"]
        login_response: String,
    }


    let row: LoginRow = sql_query(query)
        .bind::<SqlUuid, _>(userid)
        .get_result(conn)
        .await
        .map_err(|e| Status::internal(format!("Query failed: {e}")))?;

    let parsed: GetLandingResponseSerde = serde_json::from_str(&row.login_response)
        .map_err(|e| Status::internal(format!("Deserialization failed: {e}")))?;

    Ok(parsed.into())

}

pub async fn update(u: User) -> Result<User, Error> {
    u.validate()?;

    let u: User = diesel::update(user::dsl::user.find(&u.id))
        .set((
            user::updated_at.eq(Utc::now()),
            user::is_admin.eq(&u.is_admin),
            user::is_active.eq(&u.is_active),
            user::email.eq(&u.email),
            user::email_verified.eq(&u.email_verified),
            user::note.eq(&u.note),
            user::external_id.eq(&u.external_id),
        ))
        .get_result(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, u.id.to_string()))?;
    info!(user_id = %u.id, "User updated");
    Ok(u)
}

pub async fn set_password_hash(id: &Uuid, hash: &str) -> Result<User, Error> {
    let u: User = diesel::update(user::dsl::user.find(&fields::Uuid::from(id)))
        .set(user::password_hash.eq(&hash))
        .get_result(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, id.to_string()))?;
    info!(id = %id, "Password set");
    Ok(u)
}

pub async fn delete(id: &Uuid) -> Result<(), Error> {
    let ra = diesel::delete(user::dsl::user.find(&fields::Uuid::from(id)))
        .execute(&mut get_async_db_conn().await?)
        .await
        .map_err(|e| Error::from_diesel(e, id.to_string()))?;

    if ra == 0 {
        return Err(Error::NotFound(id.to_string()));
    }
    info!(user_id = %id, "User deleted");
    Ok(())
}

pub async fn get_count() -> Result<i64, Error> {
    let count = user::dsl::user
        .select(dsl::count_star())
        .first(&mut get_async_db_conn().await?)
        .await?;
    Ok(count)
}

pub async fn list(limit: i64, offset: i64) -> Result<Vec<User>, Error> {
    let items = user::dsl::user
        .order_by(user::dsl::email)
        .limit(limit)
        .offset(offset)
        .load(&mut get_async_db_conn().await?)
        .await?;
    Ok(items)
}

// The output format is documented here:
// https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md#specification
fn hash_password(pw: &str, rounds: u32) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash_resp = Pbkdf2.hash_password_customized(
        pw.as_bytes(),
        Some(Algorithm::Pbkdf2Sha512.ident()),
        None,
        pbkdf2::Params {
            rounds,
            ..Default::default()
        },
        salt.as_salt(),
    );

    match hash_resp {
        Ok(v) => Ok(v.to_string()),
        Err(e) => Err(Error::HashPassword(format!("{}", e))),
    }
}

fn verify_password(pw: &str, hash: &str) -> bool {
    let parsed = match PasswordHash::new(hash) {
        Ok(v) => v,
        Err(_) => {
            return false;
        }
    };

    Pbkdf2.verify_password(pw.as_bytes(), &parsed).is_ok()
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::test;

    pub async fn create_user() -> User {
        let mut user = User {
            is_admin: true,
            is_active: true,
            email: "test@example.com".into(),
            email_verified: true,
            ..Default::default()
        };
        user.set_password_hash("password!", 1).unwrap();
        create(user).await.unwrap()
    }

    #[test]
    fn test_hash_password() {
        assert!(hash_password("foobar", 1000).is_ok());
    }

    #[test]
    fn test_verify_password() {
        // this is the ChirpStack Application Server default admin hash, with == removed
        // to test the compatibility betweeh the two pbkdf2 implementations.
        assert!(verify_password("admin", "$pbkdf2-sha512$i=1,l=64$l8zGKtxRESq3PA2kFhHRWA$H3lGMxOt55wjwoc+myeOoABofJY9oDpldJa7fhqdjbh700V6FLPML75UmBOt9J5VFNjAL1AvqCozA1HJM0QVGA"));
    }

    #[tokio::test]
    async fn test_user() {
        let _guard = test::prepare().await;
        let mut user = create_user().await;

        // get
        let user_get = get(&user.id).await.unwrap();
        assert_eq!(user, user_get);

        // update
        user.external_id = Some("external_id".into());
        user = update(user).await.unwrap();

        // get by external id
        let user_get = get_by_external_id("external_id").await.unwrap();
        assert_eq!(user, user_get);

        // get_by_email_and_pw
        assert!(get_by_email_and_pw("test@example.com", "bar")
            .await
            .is_err());
        let user_get = get_by_email_and_pw("test@example.com", "password!")
            .await
            .unwrap();
        assert_eq!(user, user_get);

        // delete
        delete(&user.id).await.unwrap();
        assert!(delete(&user.id).await.is_err());
    }
}

impl From<GetLandingResponseSerde> for GetLandingResponse {
    fn from(s: GetLandingResponseSerde) -> Self {
        GetLandingResponse {
            id: s.id,
            email: s.email,
            is_active: s.is_active,
            web_key: s.web_key,
            ios_key: s.ios_key,
            android_key: s.android_key,
            phone_number: s.phone_number,
            name: s.name,
            note: s.note,
            username: s.username,
            training: s.training,
            organization_id_list: s.organization_id_list,
            organization_list: Some(s.organizationList.into()),
            zone_list: Some(s.zoneList.into()),
        }
    }
}

impl From<SerdeLandingOrganizationList> for LandingOrganizationList {
    fn from(serde_list: SerdeLandingOrganizationList) -> Self {
        LandingOrganizationList {
            organizations: serde_list.organizations
                .into_iter()
                .map(|org| LandingOrganization {
                    organization_id: org.organization_id,
                    organization_name: org.organization_name,
                    organization_display_name: org.organization_display_name,
                })
                .collect(),
        }
    }
}

impl From<SerdeLandingZoneList> for LandingZoneList {
    fn from(s: SerdeLandingZoneList) -> Self {
        LandingZoneList {
            zones: s.zones.into_iter().map(|z| z.into()).collect(),
        }
    }
}

impl From<SerdeLandingZone> for LandingZone {
    fn from(z: SerdeLandingZone) -> Self {
        LandingZone {
            zone_id: z.zone_id,
            zone_name: z.zone_name,
            org_id: z.org_id,
            order: z.order,
            content_type: z.contentType,
            devices: z.devices.into_iter().map(|d| d.into()).collect(),
        }
    }
}

impl From<SerdeLandingDevice> for LandingDevice {
    fn from(d: SerdeLandingDevice) -> Self {
        LandingDevice {
            device_dev_eui: d.device_dev_eui,
            device_created_at: d.device_created_at,
            device_updated_at: d.device_updated_at,
            device_profile_id: d.device_profile_id,
            device_name: d.device_name,
            device_description: d.device_description,
            device_last_seen_at: d.device_last_seen_at,
            device_data_time: d.device_data_time,
            device_lat: d.device_lat,
            device_lng: d.device_lng,
            device_application_id: d.device_application_id,
            alerts: d.alerts.into_iter().map(|a| a.into()).collect(),
            device_profile_name: d.device_profile_name.into_iter().map(|p| p.into()).collect(),
        }
    }
}

impl From<SerdeLandingDeviceProfile> for LandingDeviceProfile {
    fn from(p: SerdeLandingDeviceProfile) -> Self {
        LandingDeviceProfile { name: p.name }
    }
}

impl From<SerdeLandingAlarm> for LandingAlarm {
    fn from(a: SerdeLandingAlarm) -> Self {
        LandingAlarm {
            id: a.id,
            dev_eui: a.dev_eui,
            min_treshold: a.min_treshold.unwrap_or_default() as f32,
            max_treshold: a.max_treshold.unwrap_or_default() as f32,
            temperature: a.temperature.unwrap_or_default(),
            humadity: a.humadity.unwrap_or_default(),
            ec: a.ec.unwrap_or_default(),
            door: a.door.unwrap_or_default(),
            w_leak: a.w_leak.unwrap_or_default(),
            sms: a.sms.unwrap_or_default(),
            email: a.email.unwrap_or_default(),
        }
    }
}