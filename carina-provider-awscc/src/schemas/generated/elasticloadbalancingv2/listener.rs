//! listener schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::ElasticLoadBalancingV2::Listener
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};

const VALID_ACTION_TYPE: &[&str] = &[
    "forward",
    "authenticate-oidc",
    "authenticate-cognito",
    "redirect",
    "fixed-response",
    "jwt-validation",
];

const VALID_FIXED_RESPONSE_CONFIG_CONTENT_TYPE: &[&str] = &[
    "text/plain",
    "text/css",
    "text/html",
    "application/javascript",
    "application/json",
];

const VALID_MUTUAL_AUTHENTICATION_MODE: &[&str] = &["off", "passthrough", "verify"];

const VALID_PROTOCOL: &[&str] = &[
    "HTTP", "HTTPS", "TCP", "TLS", "UDP", "TCP_UDP", "QUIC", "TCP_QUIC",
];

const VALID_REDIRECT_CONFIG_STATUS_CODE: &[&str] = &["HTTP_301", "HTTP_302"];

/// Returns the schema config for elasticloadbalancingv2_listener (AWS::ElasticLoadBalancingV2::Listener)
pub fn elasticloadbalancingv2_listener_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::ElasticLoadBalancingV2::Listener",
        resource_type_name: "elasticloadbalancingv2.Listener",
        primary_identifier: &["ListenerArn"],
        has_tags: false,
        schema: ResourceSchema::new("elasticloadbalancingv2.Listener")
        .with_description("Specifies a listener for an Application Load Balancer, Network Load Balancer, or Gateway Load Balancer.")
        .attribute(
            AttributeSchema::new("alpn_policy", AttributeType::list(AttributeType::string()))
                .with_description("[TLS listener] The name of the Application-Layer Protocol Negotiation (ALPN) policy.")
                .with_provider_name("AlpnPolicy"),
        )
        .attribute(
            AttributeSchema::new("certificates", AttributeType::list(AttributeType::struct_("Certificate".to_string(), vec![StructField::new("certificate_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the certificate.").with_provider_name("CertificateArn")])))
                .with_description("The default SSL server certificate for a secure listener. You must provide exactly one certificate if the listener protocol is HTTPS or TLS. For an HTTPS listener, update requires some interruptions. For a TLS listener, update requires no interruption. To create a certificate list for a secure listener, use [AWS::ElasticLoadBalancingV2::ListenerCertificate](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-elasticloadbalancingv2-listenercertificate.html).")
                .with_provider_name("Certificates")
                .with_block_name("certificate"),
        )
        .attribute(
            AttributeSchema::new("default_actions", AttributeType::list(AttributeType::struct_("Action".to_string(), vec![StructField::new("authenticate_cognito_config", AttributeType::struct_("AuthenticateCognitoConfig".to_string(), vec![StructField::new("authentication_request_extra_params", AttributeType::map(AttributeType::string())).with_description("The query parameters (up to 10) to include in the redirect request to the authorization endpoint.").with_provider_name("AuthenticationRequestExtraParams"),
                    StructField::new("on_unauthenticated_request", AttributeType::string()).with_description("The behavior if the user is not authenticated. The following are possible values: + deny```` - Return an HTTP 401 Unauthorized error. + allow```` - Allow the request to be forwarded to the target. + authenticate```` - Redirect the request to the IdP authorization endpoint. This is the default value.").with_provider_name("OnUnauthenticatedRequest"),
                    StructField::new("scope", AttributeType::string()).with_description("The set of user claims to be requested from the IdP. The default is ``openid``. To verify which scope values your IdP supports and how to separate multiple values, see the documentation for your IdP.").with_provider_name("Scope"),
                    StructField::new("session_cookie_name", AttributeType::string()).with_description("The name of the cookie used to maintain session information. The default is AWSELBAuthSessionCookie.").with_provider_name("SessionCookieName"),
                    StructField::new("session_timeout", AttributeType::string()).with_description("The maximum duration of the authentication session, in seconds. The default is 604800 seconds (7 days).").with_provider_name("SessionTimeout"),
                    StructField::new("user_pool_arn", carina_aws_types::arn()).required().with_description("The Amazon Resource Name (ARN) of the Amazon Cognito user pool.").with_provider_name("UserPoolArn"),
                    StructField::new("user_pool_client_id", AttributeType::string()).required().with_description("The ID of the Amazon Cognito user pool client.").with_provider_name("UserPoolClientId"),
                    StructField::new("user_pool_domain", AttributeType::string()).required().with_description("The domain prefix or fully-qualified domain name of the Amazon Cognito user pool.").with_provider_name("UserPoolDomain")])).with_description("[HTTPS listeners] Information for using Amazon Cognito to authenticate users. Specify only when ``Type`` is ``authenticate-cognito``.").with_provider_name("AuthenticateCognitoConfig"),
                    StructField::new("authenticate_oidc_config", AttributeType::struct_("AuthenticateOidcConfig".to_string(), vec![StructField::new("authentication_request_extra_params", AttributeType::map(AttributeType::string())).with_description("The query parameters (up to 10) to include in the redirect request to the authorization endpoint.").with_provider_name("AuthenticationRequestExtraParams"),
                    StructField::new("authorization_endpoint", AttributeType::string()).required().with_description("The authorization endpoint of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path.").with_provider_name("AuthorizationEndpoint"),
                    StructField::new("client_id", AttributeType::string()).required().with_description("The OAuth 2.0 client identifier.").with_provider_name("ClientId"),
                    StructField::new("client_secret", AttributeType::string()).with_description("The OAuth 2.0 client secret. This parameter is required if you are creating a rule. If you are modifying a rule, you can omit this parameter if you set ``UseExistingClientSecret`` to true.").with_provider_name("ClientSecret"),
                    StructField::new("issuer", AttributeType::string()).required().with_description("The OIDC issuer identifier of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path.").with_provider_name("Issuer"),
                    StructField::new("on_unauthenticated_request", AttributeType::string()).with_description("The behavior if the user is not authenticated. The following are possible values: + deny```` - Return an HTTP 401 Unauthorized error. + allow```` - Allow the request to be forwarded to the target. + authenticate```` - Redirect the request to the IdP authorization endpoint. This is the default value.").with_provider_name("OnUnauthenticatedRequest"),
                    StructField::new("scope", AttributeType::string()).with_description("The set of user claims to be requested from the IdP. The default is ``openid``. To verify which scope values your IdP supports and how to separate multiple values, see the documentation for your IdP.").with_provider_name("Scope"),
                    StructField::new("session_cookie_name", AttributeType::string()).with_description("The name of the cookie used to maintain session information. The default is AWSELBAuthSessionCookie.").with_provider_name("SessionCookieName"),
                    StructField::new("session_timeout", AttributeType::string()).with_description("The maximum duration of the authentication session, in seconds. The default is 604800 seconds (7 days).").with_provider_name("SessionTimeout"),
                    StructField::new("token_endpoint", AttributeType::string()).required().with_description("The token endpoint of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path.").with_provider_name("TokenEndpoint"),
                    StructField::new("use_existing_client_secret", AttributeType::bool()).with_description("Indicates whether to use the existing client secret when modifying a rule. If you are creating a rule, you can omit this parameter or set it to false.").with_provider_name("UseExistingClientSecret"),
                    StructField::new("user_info_endpoint", AttributeType::string()).required().with_description("The user info endpoint of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path.").with_provider_name("UserInfoEndpoint")])).with_description("[HTTPS listeners] Information about an identity provider that is compliant with OpenID Connect (OIDC). Specify only when ``Type`` is ``authenticate-oidc``.").with_provider_name("AuthenticateOidcConfig"),
                    StructField::new("fixed_response_config", AttributeType::struct_("FixedResponseConfig".to_string(), vec![StructField::new("content_type", AttributeType::enum_(carina_core::schema::enum_identity("ContentType", Some("aws.elasticloadbalancingv2.Listener.Action.FixedResponseConfig")), Some(vec!["text/plain".to_string(), "text/css".to_string(), "text/html".to_string(), "application/javascript".to_string(), "application/json".to_string()]), vec![("text/plain".to_string(), "text_plain".to_string()), ("text/css".to_string(), "text_css".to_string()), ("text/html".to_string(), "text_html".to_string()), ("application/javascript".to_string(), "application_javascript".to_string()), ("application/json".to_string(), "application_json".to_string())], None, None)).with_description("The content type. Valid Values: text/plain | text/css | text/html | application/javascript | application/json").with_provider_name("ContentType"),
                    StructField::new("message_body", AttributeType::string()).with_description("The message.").with_provider_name("MessageBody"),
                    StructField::new("status_code", carina_core::schema::types::http_response_status_code()).required().with_description("The HTTP response code (2XX, 4XX, or 5XX).").with_provider_name("StatusCode")])).with_description("[Application Load Balancer] Information for creating an action that returns a custom HTTP response. Specify only when ``Type`` is ``fixed-response``.").with_provider_name("FixedResponseConfig"),
                    StructField::new("forward_config", AttributeType::struct_("ForwardConfig".to_string(), vec![StructField::new("target_group_stickiness_config", AttributeType::struct_("TargetGroupStickinessConfig".to_string(), vec![StructField::new("duration_seconds", AttributeType::int()).with_description("[Application Load Balancers] The time period, in seconds, during which requests from a client should be routed to the same target group. The range is 1-604800 seconds (7 days). You must specify this value when enabling target group stickiness.").with_provider_name("DurationSeconds"),
                    StructField::new("enabled", AttributeType::bool()).with_description("Indicates whether target group stickiness is enabled.").with_provider_name("Enabled")])).with_description("Information about the target group stickiness for a rule.").with_provider_name("TargetGroupStickinessConfig"),
                    StructField::new("target_groups", AttributeType::list(AttributeType::struct_("TargetGroupTuple".to_string(), vec![StructField::new("target_group_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the target group.").with_provider_name("TargetGroupArn"),
                    StructField::new("weight", AttributeType::int()).with_description("The weight. The range is 0 to 999.").with_provider_name("Weight")]))).with_description("Information about how traffic will be distributed between multiple target groups in a forward rule.").with_provider_name("TargetGroups").with_block_name("target_group")])).with_description("Information for creating an action that distributes requests among multiple target groups. Specify only when ``Type`` is ``forward``. If you specify both ``ForwardConfig`` and ``TargetGroupArn``, you can specify only one target group using ``ForwardConfig`` and it must be the same target group specified in ``TargetGroupArn``.").with_provider_name("ForwardConfig").with_block_name("forward_config"),
                    StructField::new("jwt_validation_config", AttributeType::struct_("JwtValidationConfig".to_string(), vec![StructField::new("additional_claims", AttributeType::list(AttributeType::struct_("JwtValidationActionAdditionalClaim".to_string(), vec![StructField::new("format", AttributeType::string()).required().with_description("The format of the claim value.").with_provider_name("Format"),
                    StructField::new("name", AttributeType::string()).required().with_description("The name of the claim. You can't specify ``exp``, ``iss``, ``nbf``, or ``iat`` because we validate them by default.").with_provider_name("Name"),
                    StructField::new("values", AttributeType::list(AttributeType::string())).required().with_description("The claim value. The maximum size of the list is 10. Each value can be up to 256 characters in length. If the format is ``space-separated-values``, the values can't include spaces.").with_provider_name("Values")]))).with_description("").with_provider_name("AdditionalClaims").with_block_name("additional_claim"),
                    StructField::new("issuer", AttributeType::string()).required().with_description("").with_provider_name("Issuer"),
                    StructField::new("jwks_endpoint", AttributeType::string()).required().with_description("").with_provider_name("JwksEndpoint")])).with_description("[HTTPS listeners] Information for validating JWT access tokens in client requests. Specify only when ``Type`` is ``jwt-validation``.").with_provider_name("JwtValidationConfig").with_block_name("jwt_validation_config"),
                    StructField::new("order", AttributeType::int()).with_description("The order for the action. This value is required for rules with multiple actions. The action with the lowest value for order is performed first.").with_provider_name("Order"),
                    StructField::new("redirect_config", AttributeType::struct_("RedirectConfig".to_string(), vec![StructField::new("host", carina_core::schema::types::redirect_host()).with_description("The hostname. This component is not percent-encoded. The hostname can contain #{host}.").with_provider_name("Host"),
                    StructField::new("path", carina_core::schema::types::redirect_path()).with_description("The absolute path, starting with the leading \"/\". This component is not percent-encoded. The path can contain #{host}, #{path}, and #{port}.").with_provider_name("Path"),
                    StructField::new("port", carina_core::schema::types::redirect_port()).with_description("The port. You can specify a value from 1 to 65535 or #{port}.").with_provider_name("Port"),
                    StructField::new("protocol", carina_core::schema::types::redirect_protocol()).with_description("The protocol. You can specify HTTP, HTTPS, or #{protocol}. You can redirect HTTP to HTTP, HTTP to HTTPS, and HTTPS to HTTPS. You can't redirect HTTPS to HTTP.").with_provider_name("Protocol"),
                    StructField::new("query", carina_core::schema::types::redirect_query()).with_description("The query parameters, URL-encoded when necessary, but not percent-encoded. Do not include the leading \"?\", as it is automatically added. You can specify any of the reserved keywords.").with_provider_name("Query"),
                    StructField::new("status_code", AttributeType::enum_(carina_core::schema::enum_identity("StatusCode", Some("aws.elasticloadbalancingv2.Listener.Action.RedirectConfig")), Some(vec!["HTTP_301".to_string(), "HTTP_302".to_string()]), vec![("HTTP_301".to_string(), "http_301".to_string()), ("HTTP_302".to_string(), "http_302".to_string())], None, None)).required().with_description("The HTTP redirect code. The redirect is either permanent (HTTP 301) or temporary (HTTP 302).").with_provider_name("StatusCode")])).with_description("[Application Load Balancer] Information for creating a redirect action. Specify only when ``Type`` is ``redirect``.").with_provider_name("RedirectConfig"),
                    StructField::new("target_group_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the target group. Specify only when ``Type`` is ``forward`` and you want to route to a single target group. To route to multiple target groups, you must use ``ForwardConfig`` instead.").with_provider_name("TargetGroupArn"),
                    StructField::new("type", AttributeType::enum_(carina_core::schema::enum_identity("Type", Some("aws.elasticloadbalancingv2.Listener.Action")), Some(vec!["forward".to_string(), "authenticate-oidc".to_string(), "authenticate-cognito".to_string(), "redirect".to_string(), "fixed-response".to_string(), "jwt-validation".to_string()]), vec![("forward".to_string(), "forward".to_string()), ("authenticate-oidc".to_string(), "authenticate_oidc".to_string()), ("authenticate-cognito".to_string(), "authenticate_cognito".to_string()), ("redirect".to_string(), "redirect".to_string()), ("fixed-response".to_string(), "fixed_response".to_string()), ("jwt-validation".to_string(), "jwt_validation".to_string())], None, None)).required().with_description("The type of action.").with_provider_name("Type")])))
                .required()
                .with_description("The actions for the default rule. You cannot define a condition for a default rule. To create additional rules for an Application Load Balancer, use [AWS::ElasticLoadBalancingV2::ListenerRule](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-elasticloadbalancingv2-listenerrule.html).")
                .with_provider_name("DefaultActions")
                .with_block_name("default_action"),
        )
        .attribute(
            AttributeSchema::new("listener_arn", carina_aws_types::arn())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("ListenerArn"),
        )
        .attribute(
            AttributeSchema::new("listener_attributes", AttributeType::unordered_list(AttributeType::struct_("ListenerAttribute".to_string(), vec![StructField::new("key", AttributeType::string()).with_description("The name of the attribute. The following attribute is supported by Network Load Balancers, and Gateway Load Balancers. + ``tcp.idle_timeout.seconds`` - The tcp idle timeout value, in seconds. The valid range is 60-6000 seconds. The default is 350 seconds. The following attributes are only supported by Application Load Balancers. + ``routing.http.request.x_amzn_mtls_clientcert_serial_number.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Serial-Number* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert_issuer.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Issuer* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert_subject.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Subject* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert_validity.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Validity* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert_leaf.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Leaf* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert* HTTP request header. + ``routing.http.request.x_amzn_tls_version.header_name`` - Enables you to modify the header name of the *X-Amzn-Tls-Version* HTTP request header. + ``routing.http.request.x_amzn_tls_cipher_suite.header_name`` - Enables you to modify the header name of the *X-Amzn-Tls-Cipher-Suite* HTTP request header. + ``routing.http.response.server.enabled`` - Enables you to allow or remove the HTTP response server header. + ``routing.http.response.strict_transport_security.header_value`` - Informs browsers that the site should only be accessed using HTTPS, and that any future attempts to access it using HTTP should automatically be converted to HTTPS. + ``routing.http.response.access_control_allow_origin.header_value`` - Specifies which origins are allowed to access the server. + ``routing.http.response.access_control_allow_methods.header_value`` - Returns which HTTP methods are allowed when accessing the server from a different origin. + ``routing.http.response.access_control_allow_headers.header_value`` - Specifies which headers can be used during the request. + ``routing.http.response.access_control_allow_credentials.header_value`` - Indicates whether the browser should include credentials such as cookies or authentication when making requests. + ``routing.http.response.access_control_expose_headers.header_value`` - Returns which headers the browser can expose to the requesting client. + ``routing.http.response.access_control_max_age.header_value`` - Specifies how long the results of a preflight request can be cached, in seconds. + ``routing.http.response.content_security_policy.header_value`` - Specifies restrictions enforced by the browser to help minimize the risk of certain types of security threats. + ``routing.http.response.x_content_type_options.header_value`` - Indicates whether the MIME types advertised in the *Content-Type* headers should be followed and not be changed. + ``routing.http.response.x_frame_options.header_value`` - Indicates whether the browser is allowed to render a page in a *frame*, *iframe*, *embed* or *object*.").with_provider_name("Key"),
                    StructField::new("value", AttributeType::string()).with_description("The value of the attribute.").with_provider_name("Value")])))
                .with_description("The listener attributes. Attributes that you do not modify retain their current values.")
                .with_provider_name("ListenerAttributes")
                .with_block_name("listener_attribute"),
        )
        .attribute(
            AttributeSchema::new("load_balancer_arn", carina_aws_types::arn())
                .required()
                .create_only()
                .with_description("The Amazon Resource Name (ARN) of the load balancer.")
                .with_provider_name("LoadBalancerArn"),
        )
        .attribute(
            AttributeSchema::new("mutual_authentication", AttributeType::struct_("MutualAuthentication".to_string(), vec![StructField::new("advertise_trust_store_ca_names", AttributeType::string()).with_description("Indicates whether trust store CA certificate names are advertised.").with_provider_name("AdvertiseTrustStoreCaNames"),
                    StructField::new("ignore_client_certificate_expiry", AttributeType::bool()).with_description("Indicates whether expired client certificates are ignored.").with_provider_name("IgnoreClientCertificateExpiry"),
                    StructField::new("mode", AttributeType::enum_(carina_core::schema::enum_identity("Mode", Some("aws.elasticloadbalancingv2.Listener.MutualAuthentication")), Some(vec!["off".to_string(), "passthrough".to_string(), "verify".to_string()]), vec![("off".to_string(), "off".to_string()), ("passthrough".to_string(), "passthrough".to_string()), ("verify".to_string(), "verify".to_string())], None, None)).with_description("The client certificate handling method. Options are ``off``, ``passthrough`` or ``verify``. The default value on initial resource creation is ``off``. After mutual authentication is turned on, you must explicitly set the ``Mode`` to ``off`` to turn it off; removing the property from your template will not turn it off.").with_provider_name("Mode"),
                    StructField::new("trust_store_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the trust store.").with_provider_name("TrustStoreArn")]))
                .with_description("The mutual authentication configuration information.")
                .with_provider_name("MutualAuthentication"),
        )
        .attribute(
            AttributeSchema::new("port", AttributeType::int())
                .with_description("The port on which the load balancer is listening. You can't specify a port for a Gateway Load Balancer.")
                .with_provider_name("Port"),
        )
        .attribute(
            AttributeSchema::new("protocol", AttributeType::enum_(carina_core::schema::enum_identity("Protocol", Some("aws.elasticloadbalancingv2.Listener")), Some(vec!["HTTP".to_string(), "HTTPS".to_string(), "TCP".to_string(), "TLS".to_string(), "UDP".to_string(), "TCP_UDP".to_string(), "QUIC".to_string(), "TCP_QUIC".to_string()]), vec![("HTTP".to_string(), "http".to_string()), ("HTTPS".to_string(), "https".to_string()), ("TCP".to_string(), "tcp".to_string()), ("TLS".to_string(), "tls".to_string()), ("UDP".to_string(), "udp".to_string()), ("TCP_UDP".to_string(), "tcp_udp".to_string()), ("QUIC".to_string(), "quic".to_string()), ("TCP_QUIC".to_string(), "tcp_quic".to_string())], None, None))
                .with_description("The protocol for connections from clients to the load balancer. For Application Load Balancers, the supported protocols are HTTP and HTTPS. For Network Load Balancers, the supported protocols are TCP, TLS, UDP, TCP_UDP, QUIC, and TCP_QUIC. You can’t specify the UDP, TCP_UDP, QUIC, or TCP_QUIC protocol if dual-stack mode is enabled. You can't specify a protocol for a Gateway Load Balancer.")
                .with_provider_name("Protocol"),
        )
        .attribute(
            AttributeSchema::new("ssl_policy", AttributeType::string())
                .with_description("[HTTPS and TLS listeners] The security policy that defines which protocols and ciphers are supported. For more information, see [Security policies](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/describe-ssl-policies.html) in the *Application Load Balancers Guide* and [Security policies](https://docs.aws.amazon.com/elasticloadbalancing/latest/network/describe-ssl-policies.html) in the *Network Load Balancers Guide*. [HTTPS listeners] Updating the security policy can result in interruptions if the load balancer is handling a high volume of traffic. To decrease the possibility of an interruption if your load balancer is handling a high volume of traffic, create an additional load balancer or request an LCU reservation.")
                .with_provider_name("SslPolicy"),
        )
        .with_def("AuthenticateCognitoConfig", AttributeType::struct_("AuthenticateCognitoConfig".to_string(), vec![StructField::new("authentication_request_extra_params", AttributeType::map(AttributeType::string())).with_description("The query parameters (up to 10) to include in the redirect request to the authorization endpoint.").with_provider_name("AuthenticationRequestExtraParams"),
                    StructField::new("on_unauthenticated_request", AttributeType::string()).with_description("The behavior if the user is not authenticated. The following are possible values: + deny```` - Return an HTTP 401 Unauthorized error. + allow```` - Allow the request to be forwarded to the target. + authenticate```` - Redirect the request to the IdP authorization endpoint. This is the default value.").with_provider_name("OnUnauthenticatedRequest"),
                    StructField::new("scope", AttributeType::string()).with_description("The set of user claims to be requested from the IdP. The default is ``openid``. To verify which scope values your IdP supports and how to separate multiple values, see the documentation for your IdP.").with_provider_name("Scope"),
                    StructField::new("session_cookie_name", AttributeType::string()).with_description("The name of the cookie used to maintain session information. The default is AWSELBAuthSessionCookie.").with_provider_name("SessionCookieName"),
                    StructField::new("session_timeout", AttributeType::string()).with_description("The maximum duration of the authentication session, in seconds. The default is 604800 seconds (7 days).").with_provider_name("SessionTimeout"),
                    StructField::new("user_pool_arn", carina_aws_types::arn()).required().with_description("The Amazon Resource Name (ARN) of the Amazon Cognito user pool.").with_provider_name("UserPoolArn"),
                    StructField::new("user_pool_client_id", AttributeType::string()).required().with_description("The ID of the Amazon Cognito user pool client.").with_provider_name("UserPoolClientId"),
                    StructField::new("user_pool_domain", AttributeType::string()).required().with_description("The domain prefix or fully-qualified domain name of the Amazon Cognito user pool.").with_provider_name("UserPoolDomain")]))
        .with_def("AuthenticateOidcConfig", AttributeType::struct_("AuthenticateOidcConfig".to_string(), vec![StructField::new("authentication_request_extra_params", AttributeType::map(AttributeType::string())).with_description("The query parameters (up to 10) to include in the redirect request to the authorization endpoint.").with_provider_name("AuthenticationRequestExtraParams"),
                    StructField::new("authorization_endpoint", AttributeType::string()).required().with_description("The authorization endpoint of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path.").with_provider_name("AuthorizationEndpoint"),
                    StructField::new("client_id", AttributeType::string()).required().with_description("The OAuth 2.0 client identifier.").with_provider_name("ClientId"),
                    StructField::new("client_secret", AttributeType::string()).with_description("The OAuth 2.0 client secret. This parameter is required if you are creating a rule. If you are modifying a rule, you can omit this parameter if you set ``UseExistingClientSecret`` to true.").with_provider_name("ClientSecret"),
                    StructField::new("issuer", AttributeType::string()).required().with_description("The OIDC issuer identifier of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path.").with_provider_name("Issuer"),
                    StructField::new("on_unauthenticated_request", AttributeType::string()).with_description("The behavior if the user is not authenticated. The following are possible values: + deny```` - Return an HTTP 401 Unauthorized error. + allow```` - Allow the request to be forwarded to the target. + authenticate```` - Redirect the request to the IdP authorization endpoint. This is the default value.").with_provider_name("OnUnauthenticatedRequest"),
                    StructField::new("scope", AttributeType::string()).with_description("The set of user claims to be requested from the IdP. The default is ``openid``. To verify which scope values your IdP supports and how to separate multiple values, see the documentation for your IdP.").with_provider_name("Scope"),
                    StructField::new("session_cookie_name", AttributeType::string()).with_description("The name of the cookie used to maintain session information. The default is AWSELBAuthSessionCookie.").with_provider_name("SessionCookieName"),
                    StructField::new("session_timeout", AttributeType::string()).with_description("The maximum duration of the authentication session, in seconds. The default is 604800 seconds (7 days).").with_provider_name("SessionTimeout"),
                    StructField::new("token_endpoint", AttributeType::string()).required().with_description("The token endpoint of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path.").with_provider_name("TokenEndpoint"),
                    StructField::new("use_existing_client_secret", AttributeType::bool()).with_description("Indicates whether to use the existing client secret when modifying a rule. If you are creating a rule, you can omit this parameter or set it to false.").with_provider_name("UseExistingClientSecret"),
                    StructField::new("user_info_endpoint", AttributeType::string()).required().with_description("The user info endpoint of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path.").with_provider_name("UserInfoEndpoint")]))
        .with_def("FixedResponseConfig", AttributeType::struct_("FixedResponseConfig".to_string(), vec![StructField::new("content_type", AttributeType::enum_(carina_core::schema::enum_identity("ContentType", Some("aws.elasticloadbalancingv2.Listener.Action.FixedResponseConfig")), Some(vec!["text/plain".to_string(), "text/css".to_string(), "text/html".to_string(), "application/javascript".to_string(), "application/json".to_string()]), vec![("text/plain".to_string(), "text_plain".to_string()), ("text/css".to_string(), "text_css".to_string()), ("text/html".to_string(), "text_html".to_string()), ("application/javascript".to_string(), "application_javascript".to_string()), ("application/json".to_string(), "application_json".to_string())], None, None)).with_description("The content type. Valid Values: text/plain | text/css | text/html | application/javascript | application/json").with_provider_name("ContentType"),
                    StructField::new("message_body", AttributeType::string()).with_description("The message.").with_provider_name("MessageBody"),
                    StructField::new("status_code", carina_core::schema::types::http_response_status_code()).required().with_description("The HTTP response code (2XX, 4XX, or 5XX).").with_provider_name("StatusCode")]))
        .with_def("ForwardConfig", AttributeType::struct_("ForwardConfig".to_string(), vec![StructField::new("target_group_stickiness_config", AttributeType::struct_("TargetGroupStickinessConfig".to_string(), vec![StructField::new("duration_seconds", AttributeType::int()).with_description("[Application Load Balancers] The time period, in seconds, during which requests from a client should be routed to the same target group. The range is 1-604800 seconds (7 days). You must specify this value when enabling target group stickiness.").with_provider_name("DurationSeconds"),
                    StructField::new("enabled", AttributeType::bool()).with_description("Indicates whether target group stickiness is enabled.").with_provider_name("Enabled")])).with_description("Information about the target group stickiness for a rule.").with_provider_name("TargetGroupStickinessConfig"),
                    StructField::new("target_groups", AttributeType::list(AttributeType::struct_("TargetGroupTuple".to_string(), vec![StructField::new("target_group_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the target group.").with_provider_name("TargetGroupArn"),
                    StructField::new("weight", AttributeType::int()).with_description("The weight. The range is 0 to 999.").with_provider_name("Weight")]))).with_description("Information about how traffic will be distributed between multiple target groups in a forward rule.").with_provider_name("TargetGroups").with_block_name("target_group")]))
        .with_def("JwtValidationConfig", AttributeType::struct_("JwtValidationConfig".to_string(), vec![StructField::new("additional_claims", AttributeType::list(AttributeType::struct_("JwtValidationActionAdditionalClaim".to_string(), vec![StructField::new("format", AttributeType::string()).required().with_description("The format of the claim value.").with_provider_name("Format"),
                    StructField::new("name", AttributeType::string()).required().with_description("The name of the claim. You can't specify ``exp``, ``iss``, ``nbf``, or ``iat`` because we validate them by default.").with_provider_name("Name"),
                    StructField::new("values", AttributeType::list(AttributeType::string())).required().with_description("The claim value. The maximum size of the list is 10. Each value can be up to 256 characters in length. If the format is ``space-separated-values``, the values can't include spaces.").with_provider_name("Values")]))).with_description("").with_provider_name("AdditionalClaims").with_block_name("additional_claim"),
                    StructField::new("issuer", AttributeType::string()).required().with_description("").with_provider_name("Issuer"),
                    StructField::new("jwks_endpoint", AttributeType::string()).required().with_description("").with_provider_name("JwksEndpoint")]))
        .with_def("MutualAuthentication", AttributeType::struct_("MutualAuthentication".to_string(), vec![StructField::new("advertise_trust_store_ca_names", AttributeType::string()).with_description("Indicates whether trust store CA certificate names are advertised.").with_provider_name("AdvertiseTrustStoreCaNames"),
                    StructField::new("ignore_client_certificate_expiry", AttributeType::bool()).with_description("Indicates whether expired client certificates are ignored.").with_provider_name("IgnoreClientCertificateExpiry"),
                    StructField::new("mode", AttributeType::enum_(carina_core::schema::enum_identity("Mode", Some("aws.elasticloadbalancingv2.Listener.MutualAuthentication")), Some(vec!["off".to_string(), "passthrough".to_string(), "verify".to_string()]), vec![("off".to_string(), "off".to_string()), ("passthrough".to_string(), "passthrough".to_string()), ("verify".to_string(), "verify".to_string())], None, None)).with_description("The client certificate handling method. Options are ``off``, ``passthrough`` or ``verify``. The default value on initial resource creation is ``off``. After mutual authentication is turned on, you must explicitly set the ``Mode`` to ``off`` to turn it off; removing the property from your template will not turn it off.").with_provider_name("Mode"),
                    StructField::new("trust_store_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the trust store.").with_provider_name("TrustStoreArn")]))
        .with_def("RedirectConfig", AttributeType::struct_("RedirectConfig".to_string(), vec![StructField::new("host", carina_core::schema::types::redirect_host()).with_description("The hostname. This component is not percent-encoded. The hostname can contain #{host}.").with_provider_name("Host"),
                    StructField::new("path", carina_core::schema::types::redirect_path()).with_description("The absolute path, starting with the leading \"/\". This component is not percent-encoded. The path can contain #{host}, #{path}, and #{port}.").with_provider_name("Path"),
                    StructField::new("port", carina_core::schema::types::redirect_port()).with_description("The port. You can specify a value from 1 to 65535 or #{port}.").with_provider_name("Port"),
                    StructField::new("protocol", carina_core::schema::types::redirect_protocol()).with_description("The protocol. You can specify HTTP, HTTPS, or #{protocol}. You can redirect HTTP to HTTP, HTTP to HTTPS, and HTTPS to HTTPS. You can't redirect HTTPS to HTTP.").with_provider_name("Protocol"),
                    StructField::new("query", carina_core::schema::types::redirect_query()).with_description("The query parameters, URL-encoded when necessary, but not percent-encoded. Do not include the leading \"?\", as it is automatically added. You can specify any of the reserved keywords.").with_provider_name("Query"),
                    StructField::new("status_code", AttributeType::enum_(carina_core::schema::enum_identity("StatusCode", Some("aws.elasticloadbalancingv2.Listener.Action.RedirectConfig")), Some(vec!["HTTP_301".to_string(), "HTTP_302".to_string()]), vec![("HTTP_301".to_string(), "http_301".to_string()), ("HTTP_302".to_string(), "http_302".to_string())], None, None)).required().with_description("The HTTP redirect code. The redirect is either permanent (HTTP 301) or temporary (HTTP 302).").with_provider_name("StatusCode")]))
        .with_def("TargetGroupStickinessConfig", AttributeType::struct_("TargetGroupStickinessConfig".to_string(), vec![StructField::new("duration_seconds", AttributeType::int()).with_description("[Application Load Balancers] The time period, in seconds, during which requests from a client should be routed to the same target group. The range is 1-604800 seconds (7 days). You must specify this value when enabling target group stickiness.").with_provider_name("DurationSeconds"),
                    StructField::new("enabled", AttributeType::bool()).with_description("Indicates whether target group stickiness is enabled.").with_provider_name("Enabled")]))
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "elasticloadbalancingv2.Listener",
        &[
            ("type", VALID_ACTION_TYPE),
            ("content_type", VALID_FIXED_RESPONSE_CONFIG_CONTENT_TYPE),
            ("mode", VALID_MUTUAL_AUTHENTICATION_MODE),
            ("protocol", VALID_PROTOCOL),
            ("status_code", VALID_REDIRECT_CONFIG_STATUS_CODE),
        ],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "elasticloadbalancing:CreateListener",
            "elasticloadbalancing:DescribeListeners",
            "cognito-idp:DescribeUserPoolClient",
            "elasticloadbalancing:ModifyListenerAttributes",
        ],
        carina_core::effect::PlanOp::Read => &[
            "elasticloadbalancing:DescribeListeners",
            "elasticloadbalancing:DescribeListenerAttributes",
        ],
        carina_core::effect::PlanOp::Update => &[
            "elasticloadbalancing:ModifyListener",
            "elasticloadbalancing:DescribeListeners",
            "cognito-idp:DescribeUserPoolClient",
            "elasticloadbalancing:ModifyListenerAttributes",
            "elasticloadbalancing:DescribeListenerAttributes",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "elasticloadbalancing:DeleteListener",
            "elasticloadbalancing:DescribeListeners",
        ],
    }
}
