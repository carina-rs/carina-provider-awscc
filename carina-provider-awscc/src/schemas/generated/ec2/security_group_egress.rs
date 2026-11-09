//! security_group_egress schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::SecurityGroupEgress
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, types};

const VALID_IP_PROTOCOL: &[&str] = &[
    "tcp", "udp", "icmp", "icmpv6", "-1", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10",
    "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26",
    "27", "28", "29", "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42",
    "43", "44", "45", "46", "47", "48", "49", "50", "51", "52", "53", "54", "55", "56", "57", "58",
    "59", "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", "70", "71", "72", "73", "74",
    "75", "76", "77", "78", "79", "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", "90",
    "91", "92", "93", "94", "95", "96", "97", "98", "99", "100", "101", "102", "103", "104", "105",
    "106", "107", "108", "109", "110", "111", "112", "113", "114", "115", "116", "117", "118",
    "119", "120", "121", "122", "123", "124", "125", "126", "127", "128", "129", "130", "131",
    "132", "133", "134", "135", "136", "137", "138", "139", "140", "141", "142", "143", "144",
    "145", "146", "147", "148", "149", "150", "151", "152", "153", "154", "155", "156", "157",
    "158", "159", "160", "161", "162", "163", "164", "165", "166", "167", "168", "169", "170",
    "171", "172", "173", "174", "175", "176", "177", "178", "179", "180", "181", "182", "183",
    "184", "185", "186", "187", "188", "189", "190", "191", "192", "193", "194", "195", "196",
    "197", "198", "199", "200", "201", "202", "203", "204", "205", "206", "207", "208", "209",
    "210", "211", "212", "213", "214", "215", "216", "217", "218", "219", "220", "221", "222",
    "223", "224", "225", "226", "227", "228", "229", "230", "231", "232", "233", "234", "235",
    "236", "237", "238", "239", "240", "241", "242", "243", "244", "245", "246", "247", "248",
    "249", "250", "251", "252", "253", "254", "255",
];

/// Returns the schema config for ec2_security_group_egress (AWS::EC2::SecurityGroupEgress)
#[rustfmt::skip]
pub fn ec2_security_group_egress_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.SecurityGroupEgress")
        .with_description("Adds the specified outbound (egress) rule to a security group.  An outbound rule permits instances to send traffic to the specified IPv4 or IPv6 address range, the IP addresses that are specified by a prefix list, or the instances that are associated with a destination security group. For more information, see [Security group rules](https://docs.aws.amazon.com/vpc/latest/userguide/security-group-rules.html).  You must specify exactly one of the following destinations: an IPv4 address range, an IPv6 address range, a prefix list, or a security group.  You must specify a protocol for each rule (for example, TCP). If the protocol is TCP or UDP, you must also specify a port or port range. If the protocol is ICMP or ICMPv6, you must also specify the ICMP/ICMPv6 type and code. To specify all types or all codes, use -1.  Rule changes are propagated to instances associated with the security group as quickly as possible. However, a small delay might occur.");
    #[inline(never)]
    fn build_cidr_ip_attribute() -> AttributeSchema {
        AttributeSchema::optional("cidr_ip", types::ipv4_cidr())
                .create_only()
                .with_description("The IPv4 address range, in CIDR format. You must specify exactly one of the following: ``CidrIp``, ``CidrIpv6``, ``DestinationPrefixListId``, or ``DestinationSecurityGroupId``. For examples of rules that you can add to security groups for specific access scenarios, see [Security group rules for different use cases](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/security-group-rules-reference.html) in the *User Guide*.")
                .with_provider_name("CidrIp")
            .build()
    }
    schema = schema.attribute(build_cidr_ip_attribute());
    #[inline(never)]
    fn build_cidr_ipv6_attribute() -> AttributeSchema {
        AttributeSchema::optional("cidr_ipv6", types::ipv6_cidr())
                .create_only()
                .with_description("The IPv6 address range, in CIDR format. You must specify exactly one of the following: ``CidrIp``, ``CidrIpv6``, ``DestinationPrefixListId``, or ``DestinationSecurityGroupId``. For examples of rules that you can add to security groups for specific access scenarios, see [Security group rules for different use cases](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/security-group-rules-reference.html) in the *User Guide*.")
                .with_provider_name("CidrIpv6")
            .build()
    }
    schema = schema.attribute(build_cidr_ipv6_attribute());
    #[inline(never)]
    fn build_description_attribute() -> AttributeSchema {
        AttributeSchema::optional("description", AttributeType::refined_string(None, Some(carina_core::schema::PatternSpec::new("^[a-zA-Z0-9. _:/()#,@\\[\\]+=&;{}!$*-]*$", carina_core::schema::PatternMatch::Full)), Some((None, Some(255))), None))
                .with_description("The description of an egress (outbound) security group rule. Constraints: Up to 255 characters in length. Allowed characters are a-z, A-Z, 0-9, spaces, and ._-:/()#,@[]+=;{}!$*")
                .with_provider_name("Description")
            .build()
    }
    schema = schema.attribute(build_description_attribute());
    #[inline(never)]
    fn build_destination_prefix_list_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("destination_prefix_list_id", AttributeType::refined_string_with_validator(crate::schemas::string_identity::StringIdentity::Ec2PrefixListId.identity(), None, None, crate::schemas::string_identity::StringIdentity::Ec2PrefixListId.check().expect("shared string check"), None))
                .create_only()
                .with_description("The prefix list IDs for an AWS service. This is the AWS service to access through a VPC endpoint from instances associated with the security group. You must specify exactly one of the following: ``CidrIp``, ``CidrIpv6``, ``DestinationPrefixListId``, or ``DestinationSecurityGroupId``.")
                .with_provider_name("DestinationPrefixListId")
            .build()
    }
    schema = schema.attribute(build_destination_prefix_list_id_attribute());
    #[inline(never)]
    fn build_destination_security_group_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("destination_security_group_id", AttributeType::refined_string_with_validator(crate::schemas::string_identity::StringIdentity::Ec2SecurityGroupId.identity(), None, None, crate::schemas::string_identity::StringIdentity::Ec2SecurityGroupId.check().expect("shared string check"), None))
                .create_only()
                .with_description("The ID of the security group. You must specify exactly one of the following: ``CidrIp``, ``CidrIpv6``, ``DestinationPrefixListId``, or ``DestinationSecurityGroupId``.")
                .with_provider_name("DestinationSecurityGroupId")
            .build()
    }
    schema = schema.attribute(build_destination_security_group_id_attribute());
    #[inline(never)]
    fn build_from_port_attribute() -> AttributeSchema {
        AttributeSchema::optional("from_port", AttributeType::refined_int(None, Some((Some(-1), Some(65535)))))
                .create_only()
                .with_description("If the protocol is TCP or UDP, this is the start of the port range. If the protocol is ICMP or ICMPv6, this is the ICMP type or -1 (all ICMP types).")
                .with_provider_name("FromPort")
            .build()
    }
    schema = schema.attribute(build_from_port_attribute());
    #[inline(never)]
    fn build_group_id_attribute() -> AttributeSchema {
        AttributeSchema::required("group_id", AttributeType::refined_string_with_validator(crate::schemas::string_identity::StringIdentity::Ec2SecurityGroupId.identity(), None, None, crate::schemas::string_identity::StringIdentity::Ec2SecurityGroupId.check().expect("shared string check"), None))
                .create_only()
                .with_description("The ID of the security group. You must specify either the security group ID or the security group name in the request. For security groups in a nondefault VPC, you must specify the security group ID.")
                .with_provider_name("GroupId")
            .build()
    }
    schema = schema.attribute(build_group_id_attribute());
    #[inline(never)]
    fn build_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("id", AttributeType::string())
                .with_description(" (read-only)")
                .with_provider_name("Id")
            .build()
    }
    schema = schema.attribute(build_id_attribute());
    #[inline(never)]
    fn build_ip_protocol_attribute() -> AttributeSchema {
        AttributeSchema::required("ip_protocol", AttributeType::enum_(carina_core::schema::enum_identity("IpProtocol", Some("aws.ec2.SecurityGroupEgress")), Some(vec!["tcp".to_string(), "udp".to_string(), "icmp".to_string(), "icmpv6".to_string(), "-1".to_string(), "0".to_string(), "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(), "10".to_string(), "11".to_string(), "12".to_string(), "13".to_string(), "14".to_string(), "15".to_string(), "16".to_string(), "17".to_string(), "18".to_string(), "19".to_string(), "20".to_string(), "21".to_string(), "22".to_string(), "23".to_string(), "24".to_string(), "25".to_string(), "26".to_string(), "27".to_string(), "28".to_string(), "29".to_string(), "30".to_string(), "31".to_string(), "32".to_string(), "33".to_string(), "34".to_string(), "35".to_string(), "36".to_string(), "37".to_string(), "38".to_string(), "39".to_string(), "40".to_string(), "41".to_string(), "42".to_string(), "43".to_string(), "44".to_string(), "45".to_string(), "46".to_string(), "47".to_string(), "48".to_string(), "49".to_string(), "50".to_string(), "51".to_string(), "52".to_string(), "53".to_string(), "54".to_string(), "55".to_string(), "56".to_string(), "57".to_string(), "58".to_string(), "59".to_string(), "60".to_string(), "61".to_string(), "62".to_string(), "63".to_string(), "64".to_string(), "65".to_string(), "66".to_string(), "67".to_string(), "68".to_string(), "69".to_string(), "70".to_string(), "71".to_string(), "72".to_string(), "73".to_string(), "74".to_string(), "75".to_string(), "76".to_string(), "77".to_string(), "78".to_string(), "79".to_string(), "80".to_string(), "81".to_string(), "82".to_string(), "83".to_string(), "84".to_string(), "85".to_string(), "86".to_string(), "87".to_string(), "88".to_string(), "89".to_string(), "90".to_string(), "91".to_string(), "92".to_string(), "93".to_string(), "94".to_string(), "95".to_string(), "96".to_string(), "97".to_string(), "98".to_string(), "99".to_string(), "100".to_string(), "101".to_string(), "102".to_string(), "103".to_string(), "104".to_string(), "105".to_string(), "106".to_string(), "107".to_string(), "108".to_string(), "109".to_string(), "110".to_string(), "111".to_string(), "112".to_string(), "113".to_string(), "114".to_string(), "115".to_string(), "116".to_string(), "117".to_string(), "118".to_string(), "119".to_string(), "120".to_string(), "121".to_string(), "122".to_string(), "123".to_string(), "124".to_string(), "125".to_string(), "126".to_string(), "127".to_string(), "128".to_string(), "129".to_string(), "130".to_string(), "131".to_string(), "132".to_string(), "133".to_string(), "134".to_string(), "135".to_string(), "136".to_string(), "137".to_string(), "138".to_string(), "139".to_string(), "140".to_string(), "141".to_string(), "142".to_string(), "143".to_string(), "144".to_string(), "145".to_string(), "146".to_string(), "147".to_string(), "148".to_string(), "149".to_string(), "150".to_string(), "151".to_string(), "152".to_string(), "153".to_string(), "154".to_string(), "155".to_string(), "156".to_string(), "157".to_string(), "158".to_string(), "159".to_string(), "160".to_string(), "161".to_string(), "162".to_string(), "163".to_string(), "164".to_string(), "165".to_string(), "166".to_string(), "167".to_string(), "168".to_string(), "169".to_string(), "170".to_string(), "171".to_string(), "172".to_string(), "173".to_string(), "174".to_string(), "175".to_string(), "176".to_string(), "177".to_string(), "178".to_string(), "179".to_string(), "180".to_string(), "181".to_string(), "182".to_string(), "183".to_string(), "184".to_string(), "185".to_string(), "186".to_string(), "187".to_string(), "188".to_string(), "189".to_string(), "190".to_string(), "191".to_string(), "192".to_string(), "193".to_string(), "194".to_string(), "195".to_string(), "196".to_string(), "197".to_string(), "198".to_string(), "199".to_string(), "200".to_string(), "201".to_string(), "202".to_string(), "203".to_string(), "204".to_string(), "205".to_string(), "206".to_string(), "207".to_string(), "208".to_string(), "209".to_string(), "210".to_string(), "211".to_string(), "212".to_string(), "213".to_string(), "214".to_string(), "215".to_string(), "216".to_string(), "217".to_string(), "218".to_string(), "219".to_string(), "220".to_string(), "221".to_string(), "222".to_string(), "223".to_string(), "224".to_string(), "225".to_string(), "226".to_string(), "227".to_string(), "228".to_string(), "229".to_string(), "230".to_string(), "231".to_string(), "232".to_string(), "233".to_string(), "234".to_string(), "235".to_string(), "236".to_string(), "237".to_string(), "238".to_string(), "239".to_string(), "240".to_string(), "241".to_string(), "242".to_string(), "243".to_string(), "244".to_string(), "245".to_string(), "246".to_string(), "247".to_string(), "248".to_string(), "249".to_string(), "250".to_string(), "251".to_string(), "252".to_string(), "253".to_string(), "254".to_string(), "255".to_string()]), vec![("tcp".to_string(), "tcp".to_string()), ("udp".to_string(), "udp".to_string()), ("icmp".to_string(), "icmp".to_string()), ("icmpv6".to_string(), "icmpv6".to_string()), ("-1".to_string(), "all".to_string()), ("0".to_string(), "0".to_string()), ("1".to_string(), "1".to_string()), ("2".to_string(), "2".to_string()), ("3".to_string(), "3".to_string()), ("4".to_string(), "4".to_string()), ("5".to_string(), "5".to_string()), ("6".to_string(), "6".to_string()), ("7".to_string(), "7".to_string()), ("8".to_string(), "8".to_string()), ("9".to_string(), "9".to_string()), ("10".to_string(), "10".to_string()), ("11".to_string(), "11".to_string()), ("12".to_string(), "12".to_string()), ("13".to_string(), "13".to_string()), ("14".to_string(), "14".to_string()), ("15".to_string(), "15".to_string()), ("16".to_string(), "16".to_string()), ("17".to_string(), "17".to_string()), ("18".to_string(), "18".to_string()), ("19".to_string(), "19".to_string()), ("20".to_string(), "20".to_string()), ("21".to_string(), "21".to_string()), ("22".to_string(), "22".to_string()), ("23".to_string(), "23".to_string()), ("24".to_string(), "24".to_string()), ("25".to_string(), "25".to_string()), ("26".to_string(), "26".to_string()), ("27".to_string(), "27".to_string()), ("28".to_string(), "28".to_string()), ("29".to_string(), "29".to_string()), ("30".to_string(), "30".to_string()), ("31".to_string(), "31".to_string()), ("32".to_string(), "32".to_string()), ("33".to_string(), "33".to_string()), ("34".to_string(), "34".to_string()), ("35".to_string(), "35".to_string()), ("36".to_string(), "36".to_string()), ("37".to_string(), "37".to_string()), ("38".to_string(), "38".to_string()), ("39".to_string(), "39".to_string()), ("40".to_string(), "40".to_string()), ("41".to_string(), "41".to_string()), ("42".to_string(), "42".to_string()), ("43".to_string(), "43".to_string()), ("44".to_string(), "44".to_string()), ("45".to_string(), "45".to_string()), ("46".to_string(), "46".to_string()), ("47".to_string(), "47".to_string()), ("48".to_string(), "48".to_string()), ("49".to_string(), "49".to_string()), ("50".to_string(), "50".to_string()), ("51".to_string(), "51".to_string()), ("52".to_string(), "52".to_string()), ("53".to_string(), "53".to_string()), ("54".to_string(), "54".to_string()), ("55".to_string(), "55".to_string()), ("56".to_string(), "56".to_string()), ("57".to_string(), "57".to_string()), ("58".to_string(), "58".to_string()), ("59".to_string(), "59".to_string()), ("60".to_string(), "60".to_string()), ("61".to_string(), "61".to_string()), ("62".to_string(), "62".to_string()), ("63".to_string(), "63".to_string()), ("64".to_string(), "64".to_string()), ("65".to_string(), "65".to_string()), ("66".to_string(), "66".to_string()), ("67".to_string(), "67".to_string()), ("68".to_string(), "68".to_string()), ("69".to_string(), "69".to_string()), ("70".to_string(), "70".to_string()), ("71".to_string(), "71".to_string()), ("72".to_string(), "72".to_string()), ("73".to_string(), "73".to_string()), ("74".to_string(), "74".to_string()), ("75".to_string(), "75".to_string()), ("76".to_string(), "76".to_string()), ("77".to_string(), "77".to_string()), ("78".to_string(), "78".to_string()), ("79".to_string(), "79".to_string()), ("80".to_string(), "80".to_string()), ("81".to_string(), "81".to_string()), ("82".to_string(), "82".to_string()), ("83".to_string(), "83".to_string()), ("84".to_string(), "84".to_string()), ("85".to_string(), "85".to_string()), ("86".to_string(), "86".to_string()), ("87".to_string(), "87".to_string()), ("88".to_string(), "88".to_string()), ("89".to_string(), "89".to_string()), ("90".to_string(), "90".to_string()), ("91".to_string(), "91".to_string()), ("92".to_string(), "92".to_string()), ("93".to_string(), "93".to_string()), ("94".to_string(), "94".to_string()), ("95".to_string(), "95".to_string()), ("96".to_string(), "96".to_string()), ("97".to_string(), "97".to_string()), ("98".to_string(), "98".to_string()), ("99".to_string(), "99".to_string()), ("100".to_string(), "100".to_string()), ("101".to_string(), "101".to_string()), ("102".to_string(), "102".to_string()), ("103".to_string(), "103".to_string()), ("104".to_string(), "104".to_string()), ("105".to_string(), "105".to_string()), ("106".to_string(), "106".to_string()), ("107".to_string(), "107".to_string()), ("108".to_string(), "108".to_string()), ("109".to_string(), "109".to_string()), ("110".to_string(), "110".to_string()), ("111".to_string(), "111".to_string()), ("112".to_string(), "112".to_string()), ("113".to_string(), "113".to_string()), ("114".to_string(), "114".to_string()), ("115".to_string(), "115".to_string()), ("116".to_string(), "116".to_string()), ("117".to_string(), "117".to_string()), ("118".to_string(), "118".to_string()), ("119".to_string(), "119".to_string()), ("120".to_string(), "120".to_string()), ("121".to_string(), "121".to_string()), ("122".to_string(), "122".to_string()), ("123".to_string(), "123".to_string()), ("124".to_string(), "124".to_string()), ("125".to_string(), "125".to_string()), ("126".to_string(), "126".to_string()), ("127".to_string(), "127".to_string()), ("128".to_string(), "128".to_string()), ("129".to_string(), "129".to_string()), ("130".to_string(), "130".to_string()), ("131".to_string(), "131".to_string()), ("132".to_string(), "132".to_string()), ("133".to_string(), "133".to_string()), ("134".to_string(), "134".to_string()), ("135".to_string(), "135".to_string()), ("136".to_string(), "136".to_string()), ("137".to_string(), "137".to_string()), ("138".to_string(), "138".to_string()), ("139".to_string(), "139".to_string()), ("140".to_string(), "140".to_string()), ("141".to_string(), "141".to_string()), ("142".to_string(), "142".to_string()), ("143".to_string(), "143".to_string()), ("144".to_string(), "144".to_string()), ("145".to_string(), "145".to_string()), ("146".to_string(), "146".to_string()), ("147".to_string(), "147".to_string()), ("148".to_string(), "148".to_string()), ("149".to_string(), "149".to_string()), ("150".to_string(), "150".to_string()), ("151".to_string(), "151".to_string()), ("152".to_string(), "152".to_string()), ("153".to_string(), "153".to_string()), ("154".to_string(), "154".to_string()), ("155".to_string(), "155".to_string()), ("156".to_string(), "156".to_string()), ("157".to_string(), "157".to_string()), ("158".to_string(), "158".to_string()), ("159".to_string(), "159".to_string()), ("160".to_string(), "160".to_string()), ("161".to_string(), "161".to_string()), ("162".to_string(), "162".to_string()), ("163".to_string(), "163".to_string()), ("164".to_string(), "164".to_string()), ("165".to_string(), "165".to_string()), ("166".to_string(), "166".to_string()), ("167".to_string(), "167".to_string()), ("168".to_string(), "168".to_string()), ("169".to_string(), "169".to_string()), ("170".to_string(), "170".to_string()), ("171".to_string(), "171".to_string()), ("172".to_string(), "172".to_string()), ("173".to_string(), "173".to_string()), ("174".to_string(), "174".to_string()), ("175".to_string(), "175".to_string()), ("176".to_string(), "176".to_string()), ("177".to_string(), "177".to_string()), ("178".to_string(), "178".to_string()), ("179".to_string(), "179".to_string()), ("180".to_string(), "180".to_string()), ("181".to_string(), "181".to_string()), ("182".to_string(), "182".to_string()), ("183".to_string(), "183".to_string()), ("184".to_string(), "184".to_string()), ("185".to_string(), "185".to_string()), ("186".to_string(), "186".to_string()), ("187".to_string(), "187".to_string()), ("188".to_string(), "188".to_string()), ("189".to_string(), "189".to_string()), ("190".to_string(), "190".to_string()), ("191".to_string(), "191".to_string()), ("192".to_string(), "192".to_string()), ("193".to_string(), "193".to_string()), ("194".to_string(), "194".to_string()), ("195".to_string(), "195".to_string()), ("196".to_string(), "196".to_string()), ("197".to_string(), "197".to_string()), ("198".to_string(), "198".to_string()), ("199".to_string(), "199".to_string()), ("200".to_string(), "200".to_string()), ("201".to_string(), "201".to_string()), ("202".to_string(), "202".to_string()), ("203".to_string(), "203".to_string()), ("204".to_string(), "204".to_string()), ("205".to_string(), "205".to_string()), ("206".to_string(), "206".to_string()), ("207".to_string(), "207".to_string()), ("208".to_string(), "208".to_string()), ("209".to_string(), "209".to_string()), ("210".to_string(), "210".to_string()), ("211".to_string(), "211".to_string()), ("212".to_string(), "212".to_string()), ("213".to_string(), "213".to_string()), ("214".to_string(), "214".to_string()), ("215".to_string(), "215".to_string()), ("216".to_string(), "216".to_string()), ("217".to_string(), "217".to_string()), ("218".to_string(), "218".to_string()), ("219".to_string(), "219".to_string()), ("220".to_string(), "220".to_string()), ("221".to_string(), "221".to_string()), ("222".to_string(), "222".to_string()), ("223".to_string(), "223".to_string()), ("224".to_string(), "224".to_string()), ("225".to_string(), "225".to_string()), ("226".to_string(), "226".to_string()), ("227".to_string(), "227".to_string()), ("228".to_string(), "228".to_string()), ("229".to_string(), "229".to_string()), ("230".to_string(), "230".to_string()), ("231".to_string(), "231".to_string()), ("232".to_string(), "232".to_string()), ("233".to_string(), "233".to_string()), ("234".to_string(), "234".to_string()), ("235".to_string(), "235".to_string()), ("236".to_string(), "236".to_string()), ("237".to_string(), "237".to_string()), ("238".to_string(), "238".to_string()), ("239".to_string(), "239".to_string()), ("240".to_string(), "240".to_string()), ("241".to_string(), "241".to_string()), ("242".to_string(), "242".to_string()), ("243".to_string(), "243".to_string()), ("244".to_string(), "244".to_string()), ("245".to_string(), "245".to_string()), ("246".to_string(), "246".to_string()), ("247".to_string(), "247".to_string()), ("248".to_string(), "248".to_string()), ("249".to_string(), "249".to_string()), ("250".to_string(), "250".to_string()), ("251".to_string(), "251".to_string()), ("252".to_string(), "252".to_string()), ("253".to_string(), "253".to_string()), ("254".to_string(), "254".to_string()), ("255".to_string(), "255".to_string())], None, None))
                .create_only()
                .with_description("The IP protocol name (``tcp``, ``udp``, ``icmp``, ``icmpv6``) or number (see [Protocol Numbers](https://docs.aws.amazon.com/http://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml)). Use ``-1`` to specify all protocols. When authorizing security group rules, specifying ``-1`` or a protocol number other than ``tcp``, ``udp``, ``icmp``, or ``icmpv6`` allows traffic on all ports, regardless of any port range you specify. For ``tcp``, ``udp``, and ``icmp``, you must specify a port range. For ``icmpv6``, the port range is optional; if you omit the port range, traffic for all types and codes is allowed.")
                .with_provider_name("IpProtocol")
            .build()
    }
    schema = schema.attribute(build_ip_protocol_attribute());
    #[inline(never)]
    fn build_to_port_attribute() -> AttributeSchema {
        AttributeSchema::optional("to_port", AttributeType::refined_int(None, Some((Some(-1), Some(65535)))))
                .create_only()
                .with_description("If the protocol is TCP or UDP, this is the end of the port range. If the protocol is ICMP or ICMPv6, this is the ICMP code or -1 (all ICMP codes). If the start port is -1 (all ICMP types), then the end port must be -1 (all ICMP codes).")
                .with_provider_name("ToPort")
            .build()
    }
    schema = schema.attribute(build_to_port_attribute());
    schema = schema
        .with_conflicting_replacement();
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::SecurityGroupEgress",
        resource_type_name: "ec2.SecurityGroupEgress",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Id", dsl_name: "id" }],
        exclusive_groups: &[],
        tags: None,
        await_on_read_back: vec![],
        schema: schema.validate().expect("generated schema must pass admission"),
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "ec2.SecurityGroupEgress",
        &[("ip_protocol", VALID_IP_PROTOCOL)],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:AuthorizeSecurityGroupEgress",
            "ec2:RevokeSecurityGroupEgress",
            "ec2:DescribeSecurityGroupRules",
        ],
        carina_core::effect::PlanOp::Read => &["ec2:DescribeSecurityGroupRules"],
        carina_core::effect::PlanOp::Update => &["ec2:UpdateSecurityGroupRuleDescriptionsEgress"],
        carina_core::effect::PlanOp::Delete => &[
            "ec2:RevokeSecurityGroupEgress",
            "ec2:DescribeSecurityGroupRules",
        ],
    }
}
