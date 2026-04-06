//! Per-resource operational configuration for CloudControl API timeouts and retries.
//!
//! These configs override the default polling/retry parameters in cloudcontrol.rs
//! for resources that need longer timeouts due to slow AWS operations.

use carina_core::schema::OperationConfig;

/// Returns the OperationConfig for a resource type, if one is defined.
pub fn get(resource_type: &str) -> Option<OperationConfig> {
    match resource_type {
        // Transit Gateway deletion can take 10-20 minutes, especially during
        // create_before_destroy when VPC attachments are still being detached.
        "ec2.transit_gateway" | "ec2.transit_gateway_attachment" => Some(OperationConfig {
            delete_timeout_secs: Some(1800),
            delete_max_retries: Some(24),
            ..Default::default()
        }),
        // IPAM and IPAM Pool deletions can take 15-30 minutes via CloudControl API.
        "ec2.ipam" | "ec2.ipam_pool" => Some(OperationConfig {
            delete_timeout_secs: Some(1800),
            ..Default::default()
        }),
        // NatGateway deletion via CloudControl API can take 10-15 minutes.
        "ec2.nat_gateway" => Some(OperationConfig {
            delete_timeout_secs: Some(1200),
            ..Default::default()
        }),
        // VPCGatewayAttachment deletion can be slow when dependent resources
        // (e.g., NAT gateways) are still being cleaned up.
        "ec2.vpc_gateway_attachment" => Some(OperationConfig {
            delete_timeout_secs: Some(1800),
            ..Default::default()
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transit_gateway_has_extended_config() {
        let config = get("ec2.transit_gateway").unwrap();
        assert_eq!(config.delete_timeout_secs, Some(1800));
        assert_eq!(config.delete_max_retries, Some(24));
    }

    #[test]
    fn test_transit_gateway_attachment_has_extended_config() {
        let config = get("ec2.transit_gateway_attachment").unwrap();
        assert_eq!(config.delete_timeout_secs, Some(1800));
        assert_eq!(config.delete_max_retries, Some(24));
    }

    #[test]
    fn test_ipam_has_extended_timeout() {
        let config = get("ec2.ipam").unwrap();
        assert_eq!(config.delete_timeout_secs, Some(1800));
        assert_eq!(config.delete_max_retries, None);
    }

    #[test]
    fn test_nat_gateway_has_extended_timeout() {
        let config = get("ec2.nat_gateway").unwrap();
        assert_eq!(config.delete_timeout_secs, Some(1200));
    }

    #[test]
    fn test_vpc_gateway_attachment_has_extended_timeout() {
        let config = get("ec2.vpc_gateway_attachment").unwrap();
        assert_eq!(config.delete_timeout_secs, Some(1800));
    }

    #[test]
    fn test_regular_resource_has_no_config() {
        assert!(get("ec2.vpc").is_none());
        assert!(get("s3.bucket").is_none());
    }
}
