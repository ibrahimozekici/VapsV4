

use super::auth::{validator, AuthID};

pub struct Notification {
    validator: validator::RequestValidator,
}

impl Notification {
    pub fn new(validator: validator::RequestValidator) -> Self {
        Notification { validator }
    }
}


#[tonic::async_trait]
impl NotificationService for Notification {


}