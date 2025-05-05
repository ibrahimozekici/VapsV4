use anyhow::Result;
use tracing::{span, Instrument, Level};

use crate::{storage::get_async_db_conn, uplink::error};

use super::{data, UplinkFrameSet};

pub struct Data {}

impl Data {
    pub async fn handle(ufs: UplinkFrameSet) -> Result<()> {
        let span = span!(Level::INFO, "data_up_sns", dev_eui = tracing::field::Empty);

        let mut db_conn = match get_async_db_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!(error = %e, "Failed to get DB connection");
                return Err(e.into()); // ✅ return here
            }
        };


        data::Data::_handle(ufs, &mut db_conn).instrument(span).await
    }
}
