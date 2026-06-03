---
title: "awscc.elasticloadbalancingv2.Listener"
description: "AWSCC Elastic Load Balancing v2 Listener resource reference"
---


CloudFormation Type: `AWS::ElasticLoadBalancingV2::Listener`

Specifies a listener for an Application Load Balancer, Network Load Balancer, or Gateway Load Balancer.

## Example

```crn
awscc.elasticloadbalancingv2.Listener {
  load_balancer_arn = 'arn:aws:elasticloadbalancing:ap-northeast-1:123456789012:loadbalancer/app/registry-alb/abc123'
  port              = 443
  protocol          = 'HTTPS'

  certificate {
    certificate_arn = 'arn:aws:acm:ap-northeast-1:123456789012:certificate/aaaa-bbbb'
  }

  default_action {
    type             = 'forward'
    target_group_arn = 'arn:aws:elasticloadbalancing:ap-northeast-1:123456789012:targetgroup/registry-tg/def456'
  }
}
```

## Argument Reference

### `alpn_policy`

- **Type:** `List<String>`
- **Required:** No

[TLS listener] The name of the Application-Layer Protocol Negotiation (ALPN) policy.

### `certificates`

- **Type:** [List\<Certificate\>](#certificate)
- **Required:** No

The default SSL server certificate for a secure listener. You must provide exactly one certificate if the listener protocol is HTTPS or TLS. For an HTTPS listener, update requires some interruptions. For a TLS listener, update requires no interruption. To create a certificate list for a secure listener, use [AWS::ElasticLoadBalancingV2::ListenerCertificate](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-elasticloadbalancingv2-listenercertificate.html).

### `default_actions`

- **Type:** [List\<Action\>](#action)
- **Required:** Yes

The actions for the default rule. You cannot define a condition for a default rule. To create additional rules for an Application Load Balancer, use [AWS::ElasticLoadBalancingV2::ListenerRule](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-elasticloadbalancingv2-listenerrule.html).

### `listener_attributes`

- **Type:** [List\<ListenerAttribute\>](#listenerattribute)
- **Required:** No

The listener attributes. Attributes that you do not modify retain their current values.

### `load_balancer_arn`

- **Type:** Arn
- **Required:** Yes
- **Create-only:** Yes

The Amazon Resource Name (ARN) of the load balancer.

### `mutual_authentication`

- **Type:** [Struct(MutualAuthentication)](#mutualauthentication)
- **Required:** No

The mutual authentication configuration information.

### `port`

- **Type:** Int
- **Required:** No

The port on which the load balancer is listening. You can't specify a port for a Gateway Load Balancer.

### `protocol`

- **Type:** String
- **Required:** No

The protocol for connections from clients to the load balancer. For Application Load Balancers, the supported protocols are HTTP and HTTPS. For Network Load Balancers, the supported protocols are TCP, TLS, UDP, TCP_UDP, QUIC, and TCP_QUIC. You can’t specify the UDP, TCP_UDP, QUIC, or TCP_QUIC protocol if dual-stack mode is enabled. You can't specify a protocol for a Gateway Load Balancer.

### `ssl_policy`

- **Type:** String
- **Required:** No

[HTTPS and TLS listeners] The security policy that defines which protocols and ciphers are supported. For more information, see [Security policies](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/describe-ssl-policies.html) in the *Application Load Balancers Guide* and [Security policies](https://docs.aws.amazon.com/elasticloadbalancing/latest/network/describe-ssl-policies.html) in the *Network Load Balancers Guide*. [HTTPS listeners] Updating the security policy can result in interruptions if the load balancer is handling a high volume of traffic. To decrease the possibility of an interruption if your load balancer is handling a high volume of traffic, create an additional load balancer or request an LCU reservation.

## Enum Values

### content_type (ContentType)

| Value | DSL Identifier |
|-------|----------------|
| `text/plain` | `awscc.elasticloadbalancingv2.Listener.Action.FixedResponseConfig.ContentType.text/plain` |
| `text/css` | `awscc.elasticloadbalancingv2.Listener.Action.FixedResponseConfig.ContentType.text/css` |
| `text/html` | `awscc.elasticloadbalancingv2.Listener.Action.FixedResponseConfig.ContentType.text/html` |
| `application/javascript` | `awscc.elasticloadbalancingv2.Listener.Action.FixedResponseConfig.ContentType.application/javascript` |
| `application/json` | `awscc.elasticloadbalancingv2.Listener.Action.FixedResponseConfig.ContentType.application/json` |

Shorthand formats: `text/plain` or `ContentType.text/plain`

### mode (Mode)

| Value | DSL Identifier |
|-------|----------------|
| `off` | `awscc.elasticloadbalancingv2.Listener.MutualAuthentication.Mode.off` |
| `passthrough` | `awscc.elasticloadbalancingv2.Listener.MutualAuthentication.Mode.passthrough` |
| `verify` | `awscc.elasticloadbalancingv2.Listener.MutualAuthentication.Mode.verify` |

Shorthand formats: `off` or `Mode.off`

## Struct Definitions

### Action

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `authenticate_cognito_config` | [Struct(AuthenticateCognitoConfig)](#authenticatecognitoconfig) | No | [HTTPS listeners] Information for using Amazon Cognito to authenticate users. Specify only when ``Type`` is ``authenticate-cognito``. |
| `authenticate_oidc_config` | [Struct(AuthenticateOidcConfig)](#authenticateoidcconfig) | No | [HTTPS listeners] Information about an identity provider that is compliant with OpenID Connect (OIDC). Specify only when ``Type`` is ``authenticate-oidc``. |
| `fixed_response_config` | [Struct(FixedResponseConfig)](#fixedresponseconfig) | No | [Application Load Balancer] Information for creating an action that returns a custom HTTP response. Specify only when ``Type`` is ``fixed-response``. |
| `forward_config` | [Struct(ForwardConfig)](#forwardconfig) | No | Information for creating an action that distributes requests among multiple target groups. Specify only when ``Type`` is ``forward``. If you specify both ``ForwardConfig`` and ``TargetGroupArn``, you can specify only one target group using ``ForwardConfig`` and it must be the same target group specified in ``TargetGroupArn``. |
| `jwt_validation_config` | [Struct(JwtValidationConfig)](#jwtvalidationconfig) | No | [HTTPS listeners] Information for validating JWT access tokens in client requests. Specify only when ``Type`` is ``jwt-validation``. |
| `order` | Int | No | The order for the action. This value is required for rules with multiple actions. The action with the lowest value for order is performed first. |
| `redirect_config` | [Struct(RedirectConfig)](#redirectconfig) | No | [Application Load Balancer] Information for creating a redirect action. Specify only when ``Type`` is ``redirect``. |
| `target_group_arn` | Arn | No | The Amazon Resource Name (ARN) of the target group. Specify only when ``Type`` is ``forward`` and you want to route to a single target group. To route to multiple target groups, you must use ``ForwardConfig`` instead. |
| `type` | String | Yes | The type of action. |

### AuthenticateCognitoConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `authentication_request_extra_params` | `Map<String, String>` | No | The query parameters (up to 10) to include in the redirect request to the authorization endpoint. |
| `on_unauthenticated_request` | String | No | The behavior if the user is not authenticated. The following are possible values: + deny```` - Return an HTTP 401 Unauthorized error. + allow```` - Allow the request to be forwarded to the target. + authenticate```` - Redirect the request to the IdP authorization endpoint. This is the default value. |
| `scope` | String | No | The set of user claims to be requested from the IdP. The default is ``openid``. To verify which scope values your IdP supports and how to separate multiple values, see the documentation for your IdP. |
| `session_cookie_name` | String | No | The name of the cookie used to maintain session information. The default is AWSELBAuthSessionCookie. |
| `session_timeout` | String | No | The maximum duration of the authentication session, in seconds. The default is 604800 seconds (7 days). |
| `user_pool_arn` | Arn | Yes | The Amazon Resource Name (ARN) of the Amazon Cognito user pool. |
| `user_pool_client_id` | String | Yes | The ID of the Amazon Cognito user pool client. |
| `user_pool_domain` | String | Yes | The domain prefix or fully-qualified domain name of the Amazon Cognito user pool. |

### AuthenticateOidcConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `authentication_request_extra_params` | `Map<String, String>` | No | The query parameters (up to 10) to include in the redirect request to the authorization endpoint. |
| `authorization_endpoint` | String | Yes | The authorization endpoint of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path. |
| `client_id` | String | Yes | The OAuth 2.0 client identifier. |
| `client_secret` | String | No | The OAuth 2.0 client secret. This parameter is required if you are creating a rule. If you are modifying a rule, you can omit this parameter if you set ``UseExistingClientSecret`` to true. |
| `issuer` | String | Yes | The OIDC issuer identifier of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path. |
| `on_unauthenticated_request` | String | No | The behavior if the user is not authenticated. The following are possible values: + deny```` - Return an HTTP 401 Unauthorized error. + allow```` - Allow the request to be forwarded to the target. + authenticate```` - Redirect the request to the IdP authorization endpoint. This is the default value. |
| `scope` | String | No | The set of user claims to be requested from the IdP. The default is ``openid``. To verify which scope values your IdP supports and how to separate multiple values, see the documentation for your IdP. |
| `session_cookie_name` | String | No | The name of the cookie used to maintain session information. The default is AWSELBAuthSessionCookie. |
| `session_timeout` | String | No | The maximum duration of the authentication session, in seconds. The default is 604800 seconds (7 days). |
| `token_endpoint` | String | Yes | The token endpoint of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path. |
| `use_existing_client_secret` | Bool | No | Indicates whether to use the existing client secret when modifying a rule. If you are creating a rule, you can omit this parameter or set it to false. |
| `user_info_endpoint` | String | Yes | The user info endpoint of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path. |

### Certificate

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `certificate_arn` | Arn | No | The Amazon Resource Name (ARN) of the certificate. |

### FixedResponseConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content_type` | [Enum (ContentType)](#content_type-contenttype) | No | The content type. Valid Values: text/plain | text/css | text/html | application/javascript | application/json |
| `message_body` | String | No | The message. |
| `status_code` | String | Yes | The HTTP response code (2XX, 4XX, or 5XX). |

### ForwardConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `target_group_stickiness_config` | [Struct(TargetGroupStickinessConfig)](#targetgroupstickinessconfig) | No | Information about the target group stickiness for a rule. |
| `target_groups` | [List\<TargetGroupTuple\>](#targetgrouptuple) | No | Information about how traffic will be distributed between multiple target groups in a forward rule. |

### JwtValidationActionAdditionalClaim

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `format` | String | Yes | The format of the claim value. |
| `name` | String | Yes | The name of the claim. You can't specify ``exp``, ``iss``, ``nbf``, or ``iat`` because we validate them by default. |
| `values` | `List<String>` | Yes | The claim value. The maximum size of the list is 10. Each value can be up to 256 characters in length. If the format is ``space-separated-values``, the values can't include spaces. |

### JwtValidationConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `additional_claims` | [List\<JwtValidationActionAdditionalClaim\>](#jwtvalidationactionadditionalclaim) | No |  |
| `issuer` | String | Yes |  |
| `jwks_endpoint` | String | Yes |  |

### ListenerAttribute

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | String | No | The name of the attribute. The following attribute is supported by Network Load Balancers, and Gateway Load Balancers. + ``tcp.idle_timeout.seconds`` - The tcp idle timeout value, in seconds. The valid range is 60-6000 seconds. The default is 350 seconds. The following attributes are only supported by Application Load Balancers. + ``routing.http.request.x_amzn_mtls_clientcert_serial_number.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Serial-Number* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert_issuer.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Issuer* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert_subject.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Subject* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert_validity.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Validity* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert_leaf.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Leaf* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert* HTTP request header. + ``routing.http.request.x_amzn_tls_version.header_name`` - Enables you to modify the header name of the *X-Amzn-Tls-Version* HTTP request header. + ``routing.http.request.x_amzn_tls_cipher_suite.header_name`` - Enables you to modify the header name of the *X-Amzn-Tls-Cipher-Suite* HTTP request header. + ``routing.http.response.server.enabled`` - Enables you to allow or remove the HTTP response server header. + ``routing.http.response.strict_transport_security.header_value`` - Informs browsers that the site should only be accessed using HTTPS, and that any future attempts to access it using HTTP should automatically be converted to HTTPS. + ``routing.http.response.access_control_allow_origin.header_value`` - Specifies which origins are allowed to access the server. + ``routing.http.response.access_control_allow_methods.header_value`` - Returns which HTTP methods are allowed when accessing the server from a different origin. + ``routing.http.response.access_control_allow_headers.header_value`` - Specifies which headers can be used during the request. + ``routing.http.response.access_control_allow_credentials.header_value`` - Indicates whether the browser should include credentials such as cookies or authentication when making requests. + ``routing.http.response.access_control_expose_headers.header_value`` - Returns which headers the browser can expose to the requesting client. + ``routing.http.response.access_control_max_age.header_value`` - Specifies how long the results of a preflight request can be cached, in seconds. + ``routing.http.response.content_security_policy.header_value`` - Specifies restrictions enforced by the browser to help minimize the risk of certain types of security threats. + ``routing.http.response.x_content_type_options.header_value`` - Indicates whether the MIME types advertised in the *Content-Type* headers should be followed and not be changed. + ``routing.http.response.x_frame_options.header_value`` - Indicates whether the browser is allowed to render a page in a *frame*, *iframe*, *embed* or *object*. |
| `value` | String | No | The value of the attribute. |

### MutualAuthentication

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `advertise_trust_store_ca_names` | String | No | Indicates whether trust store CA certificate names are advertised. |
| `ignore_client_certificate_expiry` | Bool | No | Indicates whether expired client certificates are ignored. |
| `mode` | [Enum (Mode)](#mode-mode) | No | The client certificate handling method. Options are ``off``, ``passthrough`` or ``verify``. The default value on initial resource creation is ``off``. After mutual authentication is turned on, you must explicitly set the ``Mode`` to ``off`` to turn it off; removing the property from your template will not turn it off. |
| `trust_store_arn` | Arn | No | The Amazon Resource Name (ARN) of the trust store. |

### RedirectConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `host` | String | No | The hostname. This component is not percent-encoded. The hostname can contain #{host}. |
| `path` | String | No | The absolute path, starting with the leading "/". This component is not percent-encoded. The path can contain #{host}, #{path}, and #{port}. |
| `port` | String | No | The port. You can specify a value from 1 to 65535 or #{port}. |
| `protocol` | String | No | The protocol. You can specify HTTP, HTTPS, or #{protocol}. You can redirect HTTP to HTTP, HTTP to HTTPS, and HTTPS to HTTPS. You can't redirect HTTPS to HTTP. |
| `query` | String | No | The query parameters, URL-encoded when necessary, but not percent-encoded. Do not include the leading "?", as it is automatically added. You can specify any of the reserved keywords. |
| `status_code` | String | Yes | The HTTP redirect code. The redirect is either permanent (HTTP 301) or temporary (HTTP 302). |

### TargetGroupStickinessConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `duration_seconds` | Int | No | [Application Load Balancers] The time period, in seconds, during which requests from a client should be routed to the same target group. The range is 1-604800 seconds (7 days). You must specify this value when enabling target group stickiness. |
| `enabled` | Bool | No | Indicates whether target group stickiness is enabled. |

### TargetGroupTuple

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `target_group_arn` | Arn | No | The Amazon Resource Name (ARN) of the target group. |
| `weight` | Int | No | The weight. The range is 0 to 999. |

## Attribute Reference

### `listener_arn`

- **Type:** Arn



