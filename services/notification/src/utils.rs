use std::env;

pub struct Constants {
    endpoints_in_query: i32,
    secs_wait_when_handled: i32
}

pub fn init_constants() -> Constants {
    let n_endpoints_in_query = env::var("ENDPOINTS_IN_QUERY").ok().and_then(|x| x.parse::<i32>().ok()).unwrap_or(3);
    let secs_wait_when_handled = env::var("SECS_WAIT_WHEN_HANDLED").ok().and_then(|x| x.parse::<i32>().ok()).unwrap_or(10);
    Constants {
        endpoints_in_query: n_endpoints_in_query,
        secs_wait_when_handled: secs_wait_when_handled
    }
}