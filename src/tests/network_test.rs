use std::time::Duration;

use crate::network::throttle::ThrottleOptions;
use crate::network::VoteThrottleService;

#[test]
fn test_throttle_rate_limiting_and_ban() {
    let options = ThrottleOptions {
        enabled: true,
        max_failures_before_throttle: 2,
        throttle_duration: Duration::from_secs(60),
        max_failures_before_ban: 4,
        ban_duration: Duration::from_secs(120),
        failure_reset_window: Duration::from_secs(60),
    };

    let throttle = VoteThrottleService::new(options);
    let client_ip = "192.0.2.1";

    assert!(throttle.check_client_allowed(client_ip).is_ok());

    throttle.record_failure(client_ip);
    assert!(throttle.check_client_allowed(client_ip).is_ok());

    throttle.record_failure(client_ip);
    assert!(throttle.check_client_allowed(client_ip).is_err());

    throttle.record_success(client_ip);
    assert!(throttle.check_client_allowed(client_ip).is_ok());
}
