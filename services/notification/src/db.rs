use std::{env, sync::Arc};
use anyhow::Result;

use scylla::{Session, SessionBuilder};

pub async fn get_scylla_connection() -> Result<Arc<Session>> {
    let uri = env::var("SCYLLA_URI").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
    let session: Session = SessionBuilder::new().known_node(uri).build().await?;
    let session = Arc::new(session);
    session
}

