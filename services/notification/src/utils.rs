use std::env;
use uuid::Uuid;


pub struct Constants {
    endpoints_in_query: i32,
    secs_wait_when_handled: i32,
    service_uuid: Uuid
}

pub fn init_constants() -> Constants {
    let n_endpoints_in_query = env::var("ENDPOINTS_IN_QUERY").ok().and_then(|x| x.parse::<i32>().ok()).unwrap_or(1);
    let secs_wait_when_handled = env::var("SECS_WAIT_WHEN_HANDLED").ok().and_then(|x| x.parse::<i32>().ok()).unwrap_or(40);
    
    
    let id = Uuid::new_v4();
    Constants {
        endpoints_in_query: n_endpoints_in_query,
        secs_wait_when_handled: secs_wait_when_handled,
        service_uuid: id
    }
}