use crate::{
    db::{get_scylla_connection, ENDPOINTS_TABLE_NAME},
    lib::DBQueryExecutor,
    EndpointData,
};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use scylla::Session;
use scylla::{prepared_statement::PreparedStatement, IntoTypedRows, QueryResult};
use std::{fmt::format, sync::Arc};

struct MyDBQueryExecutor {
    scylla_session: Arc<Session>,
    secs_wait_when_handled: u32,
    n_endpoints_to_select: u32,
    prepared_select_endpoits_query: PreparedStatement,
}

impl MyDBQueryExecutor {
    pub async fn new(
        secs_wait_while_handled: u32,
        n_endpoints_to_select: u32,
    ) -> MyDBQueryExecutor {
        let scylla_session = get_scylla_connection().await.unwrap();

        let prepared_select_endpoints = scylla_session
            .prepare(Self::select_endpoints_str(n_endpoints_to_select))
            .await
            .unwrap();

        MyDBQueryExecutor {
            scylla_session: scylla_session,
            secs_wait_when_handled: secs_wait_while_handled,
            n_endpoints_to_select: n_endpoints_to_select,
            prepared_select_endpoits_query: prepared_select_endpoints,
        }
    }

    // fn condition_row_should_be_handled(&self) -> String {

    // let is_not_handled = {

    // };

    //     }

    fn select_endpoints_args(&self) -> (i64, i64) {
        let current_timestamp = Utc::now().timestamp_millis();
        let is_handled_too_long_timestamp_arg = {
            let threshold_millis_duration =
                Duration::seconds(self.secs_wait_when_handled.into()).num_milliseconds();
            let threshold_timestamp_millis = current_timestamp - threshold_millis_duration;
            threshold_timestamp_millis
        };

        let no_response_from_first_admin_too_long_arg = current_timestamp;

        (
            is_handled_too_long_timestamp_arg,
            no_response_from_first_admin_too_long_arg,
        )
    }

    fn select_endpoints_str(n_endpoints_to_select: u32) -> String {
        let condition_should_row_be_handled: String = {
            let notification_needs_to_be_sent_condition: &str = "(NOT ntf_first_responded) 
                    AND (
                        (NOT ntf_is_first_notification_sent) 
                        OR (
                            (ntf_first_notification_sent_plus_allowed_response_time < ?)
                            AND
                            (NOT ntf_is_second_notification_sent)
                        )
                    )";
            let is_not_handled: &str =
                "(NOT ntf_is_being_handled) OR (ntf_is_being_handled_timestamp < ?)";
            format!(
                "is_down AND ({}) AND ({})",
                is_not_handled, notification_needs_to_be_sent_condition
            )
        };
        let select_endpoints_str: String = {
            format!(
                "SELECT is_down,
                outage_id,
                ntf_is_being_handled,
                ntf_is_being_handled_timestamp,
                ntf_is_being_handled_service_id,
                ntf_is_first_notification_sent,
                ntf_first_notification_sent_plus_allowed_response_time,
                ntf_is_second_notification_sent,
                ntf_primary_admin,
                ntf_secondary_admin,
                ntf_allowed_response_duration,
                ntf_first_responded from {} where ({}) LIMIT ({})",
                ENDPOINTS_TABLE_NAME, condition_should_row_be_handled, n_endpoints_to_select
            )
        };
        log::debug!("select endpoints str query {}", select_endpoints_str);
        select_endpoints_str
    }

    async fn execute_select_statement(&self) -> QueryResult {
        let args = self.select_endpoints_args();
        log::debug!(
            "executing scylla qury: {} with arguments {:?}",
            self.prepared_select_endpoits_query.get_statement(),
            args
        );
        self.scylla_session
            .execute(&self.prepared_select_endpoits_query, args)
            .await
            .unwrap()
    }
}

#[async_trait::async_trait]
impl DBQueryExecutor for MyDBQueryExecutor {
    async fn get_endpoints_to_process(&self) -> Result<Vec<EndpointData>> {
        let select_query_result = self.execute_select_statement().await;
        
        if let Some(rows) = select_query_result
            .rows
        {
            for row in rows.into_typed::<(i32, i32, String)>() {
                let (a, b, c) = row?;
                println!("a, b, c: {}, {}, {}", a, b, c);
            }
            
            Ok(Vec::new())
        } else {
            log::warn!("select query result returned no rows!");
            Ok(Vec::new())
        }

    }
}
