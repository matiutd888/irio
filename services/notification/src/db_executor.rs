use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use sqlx::{
    any,
    postgres::PgQueryResult,
    query::{self, Query},
    Pool, Postgres,
};
use std::{fmt::format, sync::Arc};
use uuid::Uuid;

use crate::{
    db::get_postgres_connection,
    domain::{EndpointData, EndpointId, OutageId},
    lib::DBQueryExecutor,
};

struct MyDBQueryExecutor {
    postgres: Arc<Pool<Postgres>>,
    secs_wait_when_handled: u32,
    service_id: Uuid,
    n_endpoints_to_select: u32,
}

impl MyDBQueryExecutor {
    const ENDPOINT_DB_LAYOUT: &str = "
    endpoint_id,
    is_down,
    outage_id,
    ntf_is_being_handled,
    ntf_is_being_handled_timestamp,
    ntf_is_being_handled_service_id,
    ntf_is_first_notification_sent,
    ntf_first_notification_sent_timestamp,
    ntf_is_second_notification_sent,
    ntf_primary_admin,
    ntf_secondary_admin,
    ntf_allowed_response_duration,
    ntf_responded";

    const ENDPOINTS_TABLE_NAME: &str = "endpoints";
    const CURRENT_TIMESTAMP: &str = "CURRENT_TIMESTAMP";

    pub async fn new(
        secs_wait_while_handled: u32,
        n_endpoints_to_select: u32,
        service_id: Uuid,
    ) -> MyDBQueryExecutor {
        let postgres = get_postgres_connection().await.unwrap();

        MyDBQueryExecutor {
            postgres: postgres,
            secs_wait_when_handled: secs_wait_while_handled,
            service_id: service_id,
            n_endpoints_to_select: n_endpoints_to_select,
        }
    }

    fn sql_condition_should_row_be_handled(&self) -> String {
        let notification_needs_to_be_sent_condition: String = format!(
            "(NOT ntf_first_responded) 
                AND (
                    (NOT ntf_is_first_notification_sent) 
                    OR (
                        (ntf_first_notification_sent_timestamp + ntf_allowed_response_time < {})
                        AND
                        (NOT ntf_is_second_notification_sent)
                    )
                )",
            Self::CURRENT_TIMESTAMP
        );

        let is_not_handled: String = format!("(NOT ntf_is_being_handled) OR (ntf_is_being_handled_timestamp + INTERVAL '{} seconds' < {})", self.secs_wait_when_handled, Self::CURRENT_TIMESTAMP);
        format!(
            "is_down AND ({}) AND ({})",
            is_not_handled, notification_needs_to_be_sent_condition
        )
    }

    fn sql_update_row_is_handled_by_me(&self) -> String {
        format!(
            "ntf_is_being_handled = true, ntf_is_being_handled_timestamp = {}, ntf_is_being_handled_service_id = {}", Self::CURRENT_TIMESTAMP, self.service_id
        )
    }

    fn sql_update_and_select_endpoints_str(&self) -> String {
        let select_endpoints_str: String = {
            format!(
                "UPDATE {} SET {} where ({}) RETURNING {}",
                Self::ENDPOINTS_TABLE_NAME,
                self.sql_update_row_is_handled_by_me(),
                self.sql_condition_should_row_be_handled(),
                Self::ENDPOINT_DB_LAYOUT
            )
        };
        log::debug!("select endpoints str query {}", select_endpoints_str);
        select_endpoints_str
    }

    async fn execute_statement_returning_endpoints(
        &self,
        query: &str,
    ) -> Result<Vec<EndpointData>> {
        sqlx::query_as::<Postgres, EndpointData>(query)
            .fetch_all(self.postgres.as_ref())
            .await
            .map_err(anyhow::Error::msg)
    }

    async fn set_endpoint_responded(
        &self,
        endpoint_id: EndpointId,
        outage_id: OutageId,
    ) -> Result<()> {
        let format = format!(
            "UPDATE {} 
            SET 
                ntf_responded = true 
            WHERE 
                endpoint_id = $1 AND outage_id = $2",
            Self::ENDPOINTS_TABLE_NAME
        );
        let ret = sqlx::query(&format)
            .bind(endpoint_id)
            .bind(outage_id)
            .execute(self.postgres.as_ref())
            .await
            .map_err(anyhow::Error::msg)?;
        log::debug!("Pgquery result = {:?}", ret);
        Ok(())
    }

    async fn set_first_notification_sent(
        &self,
        endpoint_id: EndpointId,
        outage_id: OutageId,
    ) -> Result<()> {
        let format = format!(
            "UPDATE {} 
            SET 
                ntf_is_being_handled=false, 
                ntf_is_being_handled_timestamp=null, 
                ntf_is_being_handled_service_id=null,
                ntf_is_first_notification_sent=true,
                ntf_first_notification_sent_timestamp=CURRENT_TIMESTAMP
            WHERE 
                endpoint_id = $1 AND outage_id = $2",
            Self::ENDPOINTS_TABLE_NAME
        );
        let ret = sqlx::query(&format)
            .bind(endpoint_id)
            .bind(outage_id)
            .execute(self.postgres.as_ref())
            .await
            .map_err(anyhow::Error::msg)?;
        log::debug!("Pgquery result = {:?}", ret);
        Ok(())
    }

    async fn set_second_notification_sent(
        &self,
        endpoint_id: EndpointId,
        outage_id: OutageId,
    ) -> Result<()> {
        let format = format!(
            "UPDATE {} 
            SET 
                ntf_is_being_handled=false, 
                ntf_is_being_handled_timestamp=null, 
                ntf_is_being_handled_service_id=null,
                ntf_is_second_notification_sent=true,
            WHERE
                endpoint_id = $1 AND outage_id = $2",
            Self::ENDPOINTS_TABLE_NAME
        );
        let ret = sqlx::query(&format)
            .bind(endpoint_id)
            .bind(outage_id)
            .execute(self.postgres.as_ref())
            .await
            .map_err(anyhow::Error::msg)?;
        log::debug!("Pgquery result = {:?}", ret);
        Ok(())
    }
}

#[async_trait::async_trait]
impl DBQueryExecutor for MyDBQueryExecutor {
    async fn get_endpoints_to_process(&self) -> Result<Vec<EndpointData>> {
        let sql_query = self.sql_update_and_select_endpoints_str();
        log::debug!("sql_query to select all endpoints {}", sql_query.clone());
        let ret = self
            .execute_statement_returning_endpoints(sql_query.as_str())
            .await;
        ret
    }

    async fn mark_endpoint_responded(
        &self,
        endpoint_id: EndpointId,
        outage_id: OutageId,
    ) -> Result<()> {
        self.set_endpoint_responded(endpoint_id, outage_id).await
    }

    async fn mark_first_notification_sent(
        &self,
        endpoint_id: EndpointId,
        outage_id: OutageId,
    ) -> Result<()> {
        self.set_first_notification_sent(endpoint_id, outage_id)
            .await
    }
    async fn mark_second_notification_sent(
        &self,
        endpoint_id: EndpointId,
        outage_id: OutageId,
    ) -> Result<()> {
        self.set_second_notification_sent(endpoint_id, outage_id)
            .await
    }
}