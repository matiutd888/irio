use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use sqlx::{query, Pool, Postgres};
use std::{fmt::format, sync::Arc};
use uuid::Uuid;

use crate::{db::get_postgres_connection, domain::EndpointData, lib::DBQueryExecutor};

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
    ntf_first_responded";

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

    async fn execute_statement_returning_endpoints(&self, query: &str) -> Result<Vec<EndpointData>> {
        sqlx::query_as::<Postgres, EndpointData>(query)
            .fetch_all(self.postgres.as_ref())
            .await.map_err(anyhow::Error::msg)
    }

    // async fn update_endpoint_to_be_handled_lwt_str() -> String {
    //     let endpoint_id_is_equal_to = "endpoint_id = ?";
    //     let endpoint_is_still_not_handled = Self::sql_condition_should_row_be_handled();
    //     let update_endpoints_to_be_handled_str: String = format!(
    //         "UPDATE {} WHERE {} IF ({})",
    //         Self::ENDPOINTS_TABLE_NAME,
    //         endpoint_id_is_equal_to,
    //         endpoint_is_still_not_handled
    //     );
    //     log::debug!("update endpoints to be handled str query {}", update_endpoints_to_be_handled_str);
    //     update_endpoints_to_be_handled_str
    // }
}

#[async_trait::async_trait]
impl DBQueryExecutor for MyDBQueryExecutor {
    async fn get_endpoints_to_process(&self) -> Result<Vec<EndpointData>> {
        let sql_query = self.sql_update_and_select_endpoints_str();
        log::debug!("sql_query to select all endpoints {}", sql_query.clone());
        let ret = self.execute_statement_returning_endpoints(sql_query.as_str()).await; 
        ret
    }
}
