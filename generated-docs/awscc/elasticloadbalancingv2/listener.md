---
title: "awscc.elasticloadbalancingv2.Listener"
---


CloudFormation Type: `AWS::ElasticLoadBalancingV2::Listener`

Specifies a listener for an Application Load Balancer, Network Load Balancer, or Gateway Load Balancer.

## Example

```crn
awscc.elasticloadbalancingv2.Listener {
  load_balancer_arn = 'arn:aws:elasticloadbalancing:ap-northeast-1:123456789012:loadbalancer/app/registry-alb/abc123'
  port              = 443
  protocol          = aws.elasticloadbalancingv2.Listener.Protocol.https

  certificate {
    certificate_arn = 'arn:aws:acm:ap-northeast-1:123456789012:certificate/aaaa-bbbb'
  }

  default_action {
    type             = aws.elasticloadbalancingv2.Listener.Action.Type.forward
    target_group_arn = 'arn:aws:elasticloadbalancing:ap-northeast-1:123456789012:targetgroup/registry-tg/def456'
  }
}
```

## Argument Reference

### `alpn_policy`

- **Type:** List\<[AlpnPolicy](#alpn_policy-alpnpolicy)\> (items: unbounded..=1, ordered)
- **Required:** No

[TLS listener] The name of the Application-Layer Protocol Negotiation (ALPN) policy.

### `certificates`

- **Type:** List\<[Struct(Certificate)](#certificates-1)\> (items: unbounded..=1, unique items, ordered)
- **Required:** No

The default SSL server certificate for a secure listener. You must provide exactly one certificate if the listener protocol is HTTPS or TLS. For an HTTPS listener, update requires some interruptions. For a TLS listener, update requires no interruption. To create a certificate list for a secure listener, use [AWS::ElasticLoadBalancingV2::ListenerCertificate](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-elasticloadbalancingv2-listenercertificate.html).

### `default_actions`

- **Type:** List\<[Struct(Action)](#default_actions-1)\> (unique items, ordered)
- **Required:** Yes

The actions for the default rule. You cannot define a condition for a default rule. To create additional rules for an Application Load Balancer, use [AWS::ElasticLoadBalancingV2::ListenerRule](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-elasticloadbalancingv2-listenerrule.html).

### `listener_attributes`

- **Type:** List\<[Struct(ListenerAttribute)](#listener_attributes-1)\> (unique items, unordered)
- **Required:** No

The listener attributes. Attributes that you do not modify retain their current values.

### `load_balancer_arn`

- **Type:** `aws.Arn (pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):[^:]+:.*$/ (Full), custom validator)`
- **Required:** Yes
- **Create-only:** Yes

The Amazon Resource Name (ARN) of the load balancer.

### `mutual_authentication`

- **Type:** [Struct(MutualAuthentication)](#mutual_authentication-1)
- **Required:** No

The mutual authentication configuration information.

### `port`

- **Type:** `Int (range: 1..=65535)`
- **Required:** No

The port on which the load balancer is listening. You can't specify a port for a Gateway Load Balancer.

### `protocol`

- **Type:** [Protocol](#protocol-protocol)
- **Required:** No

The protocol for connections from clients to the load balancer. For Application Load Balancers, the supported protocols are HTTP and HTTPS. For Network Load Balancers, the supported protocols are TCP, TLS, UDP, TCP_UDP, QUIC, and TCP_QUIC. You can’t specify the UDP, TCP_UDP, QUIC, or TCP_QUIC protocol if dual-stack mode is enabled. You can't specify a protocol for a Gateway Load Balancer.

### `ssl_policy`

- **Type:** `String`
- **Required:** No

[HTTPS and TLS listeners] The security policy that defines which protocols and ciphers are supported. For more information, see [Security policies](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/describe-ssl-policies.html) in the *Application Load Balancers Guide* and [Security policies](https://docs.aws.amazon.com/elasticloadbalancing/latest/network/describe-ssl-policies.html) in the *Network Load Balancers Guide*. [HTTPS listeners] Updating the security policy can result in interruptions if the load balancer is handling a high volume of traffic. To decrease the possibility of an interruption if your load balancer is handling a high volume of traffic, create an additional load balancer or request an LCU reservation.

## Enum Values

### alpn_policy (AlpnPolicy)

| Value | DSL Identifier |
|-------|----------------|
| `HTTP1Only` | `aws.elasticloadbalancingv2.Listener.AlpnPolicy.http1_only` |
| `HTTP2Only` | `aws.elasticloadbalancingv2.Listener.AlpnPolicy.http2_only` |
| `HTTP2Optional` | `aws.elasticloadbalancingv2.Listener.AlpnPolicy.http2_optional` |
| `HTTP2Preferred` | `aws.elasticloadbalancingv2.Listener.AlpnPolicy.http2_preferred` |
| `None` | `aws.elasticloadbalancingv2.Listener.AlpnPolicy.none` |

Shorthand formats: `http1_only` or `AlpnPolicy.http1_only`

### default_actions.authenticate_cognito_config.on_unauthenticated_request (OnUnauthenticatedRequest)

| Value | DSL Identifier |
|-------|----------------|
| `deny` | `aws.elasticloadbalancingv2.Listener.Action.AuthenticateCognitoConfig.OnUnauthenticatedRequest.deny` |
| `allow` | `aws.elasticloadbalancingv2.Listener.Action.AuthenticateCognitoConfig.OnUnauthenticatedRequest.allow` |
| `authenticate` | `aws.elasticloadbalancingv2.Listener.Action.AuthenticateCognitoConfig.OnUnauthenticatedRequest.authenticate` |

Shorthand formats: `deny` or `OnUnauthenticatedRequest.deny`

### default_actions.authenticate_oidc_config.on_unauthenticated_request (OnUnauthenticatedRequest)

| Value | DSL Identifier |
|-------|----------------|
| `deny` | `aws.elasticloadbalancingv2.Listener.Action.AuthenticateOidcConfig.OnUnauthenticatedRequest.deny` |
| `allow` | `aws.elasticloadbalancingv2.Listener.Action.AuthenticateOidcConfig.OnUnauthenticatedRequest.allow` |
| `authenticate` | `aws.elasticloadbalancingv2.Listener.Action.AuthenticateOidcConfig.OnUnauthenticatedRequest.authenticate` |

Shorthand formats: `deny` or `OnUnauthenticatedRequest.deny`

### default_actions.fixed_response_config.content_type (ContentType)

| Value | DSL Identifier |
|-------|----------------|
| `text/plain` | `aws.elasticloadbalancingv2.Listener.Action.FixedResponseConfig.ContentType.text_plain` |
| `text/css` | `aws.elasticloadbalancingv2.Listener.Action.FixedResponseConfig.ContentType.text_css` |
| `text/html` | `aws.elasticloadbalancingv2.Listener.Action.FixedResponseConfig.ContentType.text_html` |
| `application/javascript` | `aws.elasticloadbalancingv2.Listener.Action.FixedResponseConfig.ContentType.application_javascript` |
| `application/json` | `aws.elasticloadbalancingv2.Listener.Action.FixedResponseConfig.ContentType.application_json` |

Shorthand formats: `text_plain` or `ContentType.text_plain`

### default_actions.jwt_validation_config.additional_claims.format (Format)

| Value | DSL Identifier |
|-------|----------------|
| `single-string` | `aws.elasticloadbalancingv2.Listener.Action.JwtValidationConfig.JwtValidationActionAdditionalClaim.Format.single_string` |
| `string-array` | `aws.elasticloadbalancingv2.Listener.Action.JwtValidationConfig.JwtValidationActionAdditionalClaim.Format.string_array` |
| `space-separated-values` | `aws.elasticloadbalancingv2.Listener.Action.JwtValidationConfig.JwtValidationActionAdditionalClaim.Format.space_separated_values` |

Shorthand formats: `single_string` or `Format.single_string`

### default_actions.redirect_config.status_code (StatusCode)

| Value | DSL Identifier |
|-------|----------------|
| `HTTP_301` | `aws.elasticloadbalancingv2.Listener.Action.RedirectConfig.StatusCode.http_301` |
| `HTTP_302` | `aws.elasticloadbalancingv2.Listener.Action.RedirectConfig.StatusCode.http_302` |

Shorthand formats: `http_301` or `StatusCode.http_301`

### default_actions.type (Type)

| Value | DSL Identifier |
|-------|----------------|
| `forward` | `aws.elasticloadbalancingv2.Listener.Action.Type.forward` |
| `authenticate-oidc` | `aws.elasticloadbalancingv2.Listener.Action.Type.authenticate_oidc` |
| `authenticate-cognito` | `aws.elasticloadbalancingv2.Listener.Action.Type.authenticate_cognito` |
| `redirect` | `aws.elasticloadbalancingv2.Listener.Action.Type.redirect` |
| `fixed-response` | `aws.elasticloadbalancingv2.Listener.Action.Type.fixed_response` |
| `jwt-validation` | `aws.elasticloadbalancingv2.Listener.Action.Type.jwt_validation` |

Shorthand formats: `forward` or `Type.forward`

### mutual_authentication.advertise_trust_store_ca_names (AdvertiseTrustStoreCaNames)

| Value | DSL Identifier |
|-------|----------------|
| `on` | `aws.elasticloadbalancingv2.Listener.MutualAuthentication.AdvertiseTrustStoreCaNames.on` |
| `off` | `aws.elasticloadbalancingv2.Listener.MutualAuthentication.AdvertiseTrustStoreCaNames.off` |

Shorthand formats: `on` or `AdvertiseTrustStoreCaNames.on`

### mutual_authentication.mode (Mode)

| Value | DSL Identifier |
|-------|----------------|
| `off` | `aws.elasticloadbalancingv2.Listener.MutualAuthentication.Mode.off` |
| `passthrough` | `aws.elasticloadbalancingv2.Listener.MutualAuthentication.Mode.passthrough` |
| `verify` | `aws.elasticloadbalancingv2.Listener.MutualAuthentication.Mode.verify` |

Shorthand formats: `off` or `Mode.off`

### protocol (Protocol)

| Value | DSL Identifier |
|-------|----------------|
| `HTTP` | `aws.elasticloadbalancingv2.Listener.Protocol.http` |
| `HTTPS` | `aws.elasticloadbalancingv2.Listener.Protocol.https` |
| `TCP` | `aws.elasticloadbalancingv2.Listener.Protocol.tcp` |
| `TLS` | `aws.elasticloadbalancingv2.Listener.Protocol.tls` |
| `UDP` | `aws.elasticloadbalancingv2.Listener.Protocol.udp` |
| `TCP_UDP` | `aws.elasticloadbalancingv2.Listener.Protocol.tcp_udp` |
| `QUIC` | `aws.elasticloadbalancingv2.Listener.Protocol.quic` |
| `TCP_QUIC` | `aws.elasticloadbalancingv2.Listener.Protocol.tcp_quic` |

Shorthand formats: `http` or `Protocol.http`

## Struct Definitions

### certificates

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `certificate_arn` | `aws.Arn (pattern: /^arn:(?:aws\|aws-cn\|aws-us-gov\|aws-iso\|aws-iso-b\|aws-iso-e\|aws-iso-f\|aws-eusc):[^:]+:.*$/ (Full), custom validator)` | No | The Amazon Resource Name (ARN) of the certificate. |

### default_actions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `authenticate_cognito_config` | [Struct(AuthenticateCognitoConfig)](#default_actionsauthenticate_cognito_config) | No | [HTTPS listeners] Information for using Amazon Cognito to authenticate users. Specify only when ``Type`` is ``authenticate-cognito``. |
| `authenticate_oidc_config` | [Struct(AuthenticateOidcConfig)](#default_actionsauthenticate_oidc_config) | No | [HTTPS listeners] Information about an identity provider that is compliant with OpenID Connect (OIDC). Specify only when ``Type`` is ``authenticate-oidc``. |
| `fixed_response_config` | [Struct(FixedResponseConfig)](#default_actionsfixed_response_config) | No | [Application Load Balancer] Information for creating an action that returns a custom HTTP response. Specify only when ``Type`` is ``fixed-response``. |
| `forward_config` | [Struct(ForwardConfig)](#default_actionsforward_config) | No | Information for creating an action that distributes requests among multiple target groups. Specify only when ``Type`` is ``forward``. If you specify both ``ForwardConfig`` and ``TargetGroupArn``, you can specify only one target group using ``ForwardConfig`` and it must be the same target group specified in ``TargetGroupArn``. |
| `jwt_validation_config` | [Struct(JwtValidationConfig)](#default_actionsjwt_validation_config) | No | [HTTPS listeners] Information for validating JWT access tokens in client requests. Specify only when ``Type`` is ``jwt-validation``. |
| `order` | `Int (range: 1..=50000)` | No | The order for the action. This value is required for rules with multiple actions. The action with the lowest value for order is performed first. |
| `redirect_config` | [Struct(RedirectConfig)](#default_actionsredirect_config) | No | [Application Load Balancer] Information for creating a redirect action. Specify only when ``Type`` is ``redirect``. |
| `target_group_arn` | `aws.Arn (pattern: /^arn:(?:aws\|aws-cn\|aws-us-gov\|aws-iso\|aws-iso-b\|aws-iso-e\|aws-iso-f\|aws-eusc):[^:]+:.*$/ (Full), custom validator)` | No | The Amazon Resource Name (ARN) of the target group. Specify only when ``Type`` is ``forward`` and you want to route to a single target group. To route to multiple target groups, you must use ``ForwardConfig`` instead. |
| `type` | [Type](#default_actionstype-type) | Yes | The type of action. |

### default_actions.authenticate_cognito_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `authentication_request_extra_params` | Map\<`String`, `String`\> | No | The query parameters (up to 10) to include in the redirect request to the authorization endpoint. |
| `on_unauthenticated_request` | [OnUnauthenticatedRequest](#default_actionsauthenticate_cognito_configon_unauthenticated_request-onunauthenticatedrequest) | No | The behavior if the user is not authenticated. The following are possible values: + deny```` - Return an HTTP 401 Unauthorized error. + allow```` - Allow the request to be forwarded to the target. + authenticate```` - Redirect the request to the IdP authorization endpoint. This is the default value. |
| `scope` | `String` | No | The set of user claims to be requested from the IdP. The default is ``openid``. To verify which scope values your IdP supports and how to separate multiple values, see the documentation for your IdP. |
| `session_cookie_name` | `String` | No | The name of the cookie used to maintain session information. The default is AWSELBAuthSessionCookie. |
| `session_timeout` | `String (pattern: /^[1-9][0-9]*\z/ (Full))` | No | The maximum duration of the authentication session, in seconds. The default is 604800 seconds (7 days). |
| `user_pool_arn` | `aws.Arn (pattern: /^arn:(?:aws\|aws-cn\|aws-us-gov\|aws-iso\|aws-iso-b\|aws-iso-e\|aws-iso-f\|aws-eusc):[^:]+:.*$/ (Full), custom validator)` | Yes | The Amazon Resource Name (ARN) of the Amazon Cognito user pool. |
| `user_pool_client_id` | `String` | Yes | The ID of the Amazon Cognito user pool client. |
| `user_pool_domain` | `String` | Yes | The domain prefix or fully-qualified domain name of the Amazon Cognito user pool. |

### default_actions.authenticate_oidc_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `authentication_request_extra_params` | Map\<`String`, `String`\> | No | The query parameters (up to 10) to include in the redirect request to the authorization endpoint. |
| `authorization_endpoint` | `HttpsUrl (custom validator)` | Yes | The authorization endpoint of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path. |
| `client_id` | `String` | Yes | The OAuth 2.0 client identifier. |
| `client_secret` | `String` | No | The OAuth 2.0 client secret. This parameter is required if you are creating a rule. If you are modifying a rule, you can omit this parameter if you set ``UseExistingClientSecret`` to true. |
| `issuer` | `HttpsUrl (custom validator)` | Yes | The OIDC issuer identifier of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path. |
| `on_unauthenticated_request` | [OnUnauthenticatedRequest](#default_actionsauthenticate_oidc_configon_unauthenticated_request-onunauthenticatedrequest) | No | The behavior if the user is not authenticated. The following are possible values: + deny```` - Return an HTTP 401 Unauthorized error. + allow```` - Allow the request to be forwarded to the target. + authenticate```` - Redirect the request to the IdP authorization endpoint. This is the default value. |
| `scope` | `String` | No | The set of user claims to be requested from the IdP. The default is ``openid``. To verify which scope values your IdP supports and how to separate multiple values, see the documentation for your IdP. |
| `session_cookie_name` | `String` | No | The name of the cookie used to maintain session information. The default is AWSELBAuthSessionCookie. |
| `session_timeout` | `String (pattern: /^[1-9][0-9]*\z/ (Full))` | No | The maximum duration of the authentication session, in seconds. The default is 604800 seconds (7 days). |
| `token_endpoint` | `HttpsUrl (custom validator)` | Yes | The token endpoint of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path. |
| `use_existing_client_secret` | `Bool` | No | Indicates whether to use the existing client secret when modifying a rule. If you are creating a rule, you can omit this parameter or set it to false. |
| `user_info_endpoint` | `HttpsUrl (custom validator)` | Yes | The user info endpoint of the IdP. This must be a full URL, including the HTTPS protocol, the domain, and the path. |

**Field groups:**

- At least one of: `client_secret`, `use_existing_client_secret`.

### default_actions.fixed_response_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content_type` | [ContentType](#default_actionsfixed_response_configcontent_type-contenttype) | No | The content type. Valid Values: text/plain \| text/css \| text/html \| application/javascript \| application/json |
| `message_body` | `String` | No | The message. |
| `status_code` | `HttpResponseStatusCode (custom validator)` | Yes | The HTTP response code (2XX, 4XX, or 5XX). |

### default_actions.forward_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `target_group_stickiness_config` | [Struct(TargetGroupStickinessConfig)](#default_actionsforward_configtarget_group_stickiness_config) | No | Information about the target group stickiness for a rule. |
| `target_groups` | List\<[Struct(TargetGroupTuple)](#default_actionsforward_configtarget_groups)\> (items: unbounded..=5, unique items, ordered) | No | Information about how traffic will be distributed between multiple target groups in a forward rule. |

### default_actions.forward_config.target_group_stickiness_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `duration_seconds` | `Int (range: 1..=604800)` | No | [Application Load Balancers] The time period, in seconds, during which requests from a client should be routed to the same target group. The range is 1-604800 seconds (7 days). You must specify this value when enabling target group stickiness. |
| `enabled` | `Bool` | No | Indicates whether target group stickiness is enabled. |

### default_actions.forward_config.target_groups

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `target_group_arn` | `aws.Arn (pattern: /^arn:(?:aws\|aws-cn\|aws-us-gov\|aws-iso\|aws-iso-b\|aws-iso-e\|aws-iso-f\|aws-eusc):[^:]+:.*$/ (Full), custom validator)` | No | The Amazon Resource Name (ARN) of the target group. |
| `weight` | `Int (range: 0..=999)` | No | The weight. The range is 0 to 999. |

### default_actions.jwt_validation_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `additional_claims` | List\<[Struct(JwtValidationActionAdditionalClaim)](#default_actionsjwt_validation_configadditional_claims)\> (items: unbounded..=10, unique items, ordered) | No |  |
| `issuer` | `String (len: unbounded..=256)` | Yes |  |
| `jwks_endpoint` | `HttpsUrl (custom validator)` | Yes |  |

### default_actions.jwt_validation_config.additional_claims

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `format` | [Format](#default_actionsjwt_validation_configadditional_claimsformat-format) | Yes | The format of the claim value. |
| `name` | `String` | Yes | The name of the claim. You can't specify ``exp``, ``iss``, ``nbf``, or ``iat`` because we validate them by default. |
| `values` | List\<`String (len: unbounded..=256)`\> (items: unbounded..=10, unique items, ordered) | Yes | The claim value. The maximum size of the list is 10. Each value can be up to 256 characters in length. If the format is ``space-separated-values``, the values can't include spaces. |

### default_actions.redirect_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `host` | `RedirectHost (custom validator)` | No | The hostname. This component is not percent-encoded. The hostname can contain #{host}. |
| `path` | `RedirectPath (custom validator)` | No | The absolute path, starting with the leading "/". This component is not percent-encoded. The path can contain #{host}, #{path}, and #{port}. |
| `port` | `RedirectPort (custom validator)` | No | The port. You can specify a value from 1 to 65535 or #{port}. |
| `protocol` | `RedirectProtocol (custom validator)` | No | The protocol. You can specify HTTP, HTTPS, or #{protocol}. You can redirect HTTP to HTTP, HTTP to HTTPS, and HTTPS to HTTPS. You can't redirect HTTPS to HTTP. |
| `query` | `RedirectQuery (custom validator)` | No | The query parameters, URL-encoded when necessary, but not percent-encoded. Do not include the leading "?", as it is automatically added. You can specify any of the reserved keywords. |
| `status_code` | [StatusCode](#default_actionsredirect_configstatus_code-statuscode) | Yes | The HTTP redirect code. The redirect is either permanent (HTTP 301) or temporary (HTTP 302). |

### listener_attributes

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | `String (pattern: /^[a-zA-Z0-9._]+\z/ (Full), len: unbounded..=256)` | No | The name of the attribute. The following attribute is supported by Network Load Balancers, and Gateway Load Balancers. + ``tcp.idle_timeout.seconds`` - The tcp idle timeout value, in seconds. The valid range is 60-6000 seconds. The default is 350 seconds. The following attributes are only supported by Application Load Balancers. + ``routing.http.request.x_amzn_mtls_clientcert_serial_number.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Serial-Number* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert_issuer.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Issuer* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert_subject.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Subject* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert_validity.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Validity* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert_leaf.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert-Leaf* HTTP request header. + ``routing.http.request.x_amzn_mtls_clientcert.header_name`` - Enables you to modify the header name of the *X-Amzn-Mtls-Clientcert* HTTP request header. + ``routing.http.request.x_amzn_tls_version.header_name`` - Enables you to modify the header name of the *X-Amzn-Tls-Version* HTTP request header. + ``routing.http.request.x_amzn_tls_cipher_suite.header_name`` - Enables you to modify the header name of the *X-Amzn-Tls-Cipher-Suite* HTTP request header. + ``routing.http.response.server.enabled`` - Enables you to allow or remove the HTTP response server header. + ``routing.http.response.strict_transport_security.header_value`` - Informs browsers that the site should only be accessed using HTTPS, and that any future attempts to access it using HTTP should automatically be converted to HTTPS. + ``routing.http.response.access_control_allow_origin.header_value`` - Specifies which origins are allowed to access the server. + ``routing.http.response.access_control_allow_methods.header_value`` - Returns which HTTP methods are allowed when accessing the server from a different origin. + ``routing.http.response.access_control_allow_headers.header_value`` - Specifies which headers can be used during the request. + ``routing.http.response.access_control_allow_credentials.header_value`` - Indicates whether the browser should include credentials such as cookies or authentication when making requests. + ``routing.http.response.access_control_expose_headers.header_value`` - Returns which headers the browser can expose to the requesting client. + ``routing.http.response.access_control_max_age.header_value`` - Specifies how long the results of a preflight request can be cached, in seconds. + ``routing.http.response.content_security_policy.header_value`` - Specifies restrictions enforced by the browser to help minimize the risk of certain types of security threats. + ``routing.http.response.x_content_type_options.header_value`` - Indicates whether the MIME types advertised in the *Content-Type* headers should be followed and not be changed. + ``routing.http.response.x_frame_options.header_value`` - Indicates whether the browser is allowed to render a page in a *frame*, *iframe*, *embed* or *object*. |
| `value` | `String` | No | The value of the attribute. |

### mutual_authentication

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `advertise_trust_store_ca_names` | [AdvertiseTrustStoreCaNames](#mutual_authenticationadvertise_trust_store_ca_names-advertisetruststorecanames) | No | Indicates whether trust store CA certificate names are advertised. |
| `ignore_client_certificate_expiry` | `Bool` | No | Indicates whether expired client certificates are ignored. |
| `mode` | [Mode](#mutual_authenticationmode-mode) | No | The client certificate handling method. Options are ``off``, ``passthrough`` or ``verify``. The default value on initial resource creation is ``off``. After mutual authentication is turned on, you must explicitly set the ``Mode`` to ``off`` to turn it off; removing the property from your template will not turn it off. |
| `trust_store_arn` | `aws.Arn (pattern: /^arn:(?:aws\|aws-cn\|aws-us-gov\|aws-iso\|aws-iso-b\|aws-iso-e\|aws-iso-f\|aws-eusc):[^:]+:.*$/ (Full), custom validator)` | No | The Amazon Resource Name (ARN) of the trust store. |

## Attribute Reference

### `listener_arn`

- **Type:** `aws.Arn (pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):[^:]+:.*$/ (Full), custom validator)`



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/Action/properties/Order` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_Action.html) | ` The order for the action. This value is required for rules with multiple actions. The action with the lowest value for order is performed first. `<br>` Valid Range: Minimum value of 1. Maximum value of 50000. ` | 2026-11-07 |
| `/definitions/Action/properties/Type` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-elasticloadbalancingv2-listener-action.html#cfn-elasticloadbalancingv2-listener-action-type) | ` The type of action. `<br>` Allowed values: forward \| authenticate-oidc \| authenticate-cognito \| redirect \| fixed-response \| jwt-validation ` | 2026-11-07 |
| `/definitions/AuthenticateCognitoConfig/properties/OnUnauthenticatedRequest` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_AuthenticateCognitoActionConfig.html) | ` The behavior if the user is not authenticated. The following are possible values: `<br>` deny - Return an HTTP 401 Unauthorized error. `<br>` allow - Allow the request to be forwarded to the target. `<br>` authenticate - Redirect the request to the IdP authorization endpoint. This is the default value. `<br>` Valid Values: deny \| allow \| authenticate ` | 2026-11-07 |
| `/definitions/AuthenticateCognitoConfig/properties/SessionTimeout` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/listener-authenticate-users.html) | ` By default, the SessionTimeout field is set to 7 days. If you want shorter sessions, you can configure a session timeout as short as 1 second. For more information, see Session timeout. ` | 2026-11-07 |
| `/definitions/AuthenticateOidcConfig/properties/OnUnauthenticatedRequest` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_AuthenticateOidcActionConfig.html) | ` The behavior if the user is not authenticated. The following are possible values: `<br>` deny - Return an HTTP 401 Unauthorized error. `<br>` allow - Allow the request to be forwarded to the target. `<br>` authenticate - Redirect the request to the IdP authorization endpoint. This is the default value. `<br>` Valid Values: deny \| allow \| authenticate ` | 2026-11-07 |
| `/definitions/AuthenticateOidcConfig/properties/SessionTimeout` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/listener-authenticate-users.html) | ` By default, the SessionTimeout field is set to 7 days. If you want shorter sessions, you can configure a session timeout as short as 1 second. For more information, see Session timeout. ` | 2026-11-07 |
| `/definitions/ForwardConfig/properties/TargetGroups` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/load-balancer-limits.html) | ` Target Groups per Action per Application Load Balancer 5 No ` | 2026-11-07 |
| `/definitions/JwtValidationActionAdditionalClaim/properties/Format` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_JwtValidationActionAdditionalClaim.html) | ` The format of the claim value. `<br>` Valid Values: single-string \| string-array \| space-separated-values ` | 2026-11-07 |
| `/definitions/JwtValidationActionAdditionalClaim/properties/Values` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_JwtValidationActionAdditionalClaim.html) | ` The claim value. The maximum size of the list is 10. Each value can be up to 256 characters in length. If the format is space-separated-values, the values can't include spaces. ` | 2026-11-07 |
| `/definitions/JwtValidationConfig/properties/AdditionalClaims` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_JwtValidationActionConfig.html) | ` Additional claims to validate. The maximum size of the list is 10. We validate the exp, iss, nbf, and iat claims by default. ` | 2026-11-07 |
| `/definitions/JwtValidationConfig/properties/Issuer` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_JwtValidationActionConfig.html) | ` The issuer of the JWT. The maximum length is 256 characters. ` | 2026-11-07 |
| `/definitions/ListenerAttribute/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_ListenerAttribute.html) | ` The name of the attribute. `<br>` The following attribute is supported by Network Load Balancers, and Gateway Load Balancers. `<br>` tcp.idle_timeout.seconds - The tcp idle timeout value, in seconds. The valid range is 60-6000 seconds. The default is 350 seconds. `<br>` The following attribute is only supported by Gateway Load Balancers: `<br>` send_tcp_reset.on_idle_timeout.enabled – Specifies whether the Gateway Load Balancer sends a TCP Reset to the sender of traffic when a TCP flow's idle timeout expires. This attribute also applies to non-SYN TCP packets received for flows that are not in the flow table. The value is true or false. The default is false. `<br>` The following attributes are only supported by Application Load Balancers. `<br>` routing.http.request.x_amzn_mtls_clientcert_serial_number.header_name - Enables you to modify the header name of the X-Amzn-Mtls-Clientcert-Serial-Number HTTP request header. `<br>` routing.http.request.x_amzn_mtls_clientcert_issuer.header_name - Enables you to modify the header name of the X-Amzn-Mtls-Clientcert-Issuer HTTP request header. `<br>` routing.http.request.x_amzn_mtls_clientcert_subject.header_name - Enables you to modify the header name of the X-Amzn-Mtls-Clientcert-Subject HTTP request header. `<br>` routing.http.request.x_amzn_mtls_clientcert_validity.header_name - Enables you to modify the header name of the X-Amzn-Mtls-Clientcert-Validity HTTP request header. `<br>` routing.http.request.x_amzn_mtls_clientcert_leaf.header_name - Enables you to modify the header name of the X-Amzn-Mtls-Clientcert-Leaf HTTP request header. `<br>` routing.http.request.x_amzn_mtls_clientcert.header_name - Enables you to modify the header name of the X-Amzn-Mtls-Clientcert HTTP request header. `<br>` routing.http.request.x_amzn_tls_version.header_name - Enables you to modify the header name of the X-Amzn-Tls-Version HTTP request header. `<br>` routing.http.request.x_amzn_tls_cipher_suite.header_name - Enables you to modify the header name of the X-Amzn-Tls-Cipher-Suite HTTP request header. `<br>` routing.http.response.server.enabled - Enables you to allow or remove the HTTP response server header. `<br>` routing.http.response.strict_transport_security.header_value - Informs browsers that the site should only be accessed using HTTPS, and that any future attempts to access it using HTTP should automatically be converted to HTTPS. `<br>` routing.http.response.access_control_allow_origin.header_value - Specifies which origins are allowed to access the server. `<br>` routing.http.response.access_control_allow_methods.header_value - Returns which HTTP methods are allowed when accessing the server from a different origin. `<br>` routing.http.response.access_control_allow_headers.header_value - Specifies which headers can be used during the request. `<br>` routing.http.response.access_control_allow_credentials.header_value - Indicates whether the browser should include credentials such as cookies or authentication when making requests. `<br>` routing.http.response.access_control_expose_headers.header_value - Returns which headers the browser can expose to the requesting client. `<br>` routing.http.response.access_control_max_age.header_value - Specifies how long the results of a preflight request can be cached, in seconds. `<br>` routing.http.response.content_security_policy.header_value - Specifies restrictions enforced by the browser to help minimize the risk of certain types of security threats. `<br>` routing.http.response.x_content_type_options.header_value - Indicates whether the MIME types advertised in the Content-Type headers should be followed and not be changed. `<br>` routing.http.response.x_frame_options.header_value - Indicates whether the browser is allowed to render a page in a frame, iframe, embed or object. `<br>` Length Constraints: Maximum length of 256. `<br>` Pattern: ^[a-zA-Z0-9._]+$ ` | 2026-11-07 |
| `/definitions/MutualAuthentication/properties/AdvertiseTrustStoreCaNames` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_MutualAuthenticationAttributes.html) | ` Indicates whether trust store CA certificate names are advertised. `<br>` Valid Values: on \| off ` | 2026-11-07 |
| `/definitions/RedirectConfig/properties/StatusCode` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-elasticloadbalancingv2-listener-redirectconfig.html#cfn-elasticloadbalancingv2-listener-redirectconfig-statuscode) | ` The HTTP redirect code. The redirect is either permanent (HTTP 301) or temporary (HTTP 302). `<br>` Allowed values: HTTP_301 \| HTTP_302 ` | 2026-11-07 |
| `/definitions/TargetGroupStickinessConfig/properties/DurationSeconds` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_TargetGroupStickinessConfig.html) | ` [Application Load Balancers] The time period, in seconds, during which requests from a client should be routed to the same target group. The range is 1-604800 seconds (7 days). You must specify this value when enabling target group stickiness. ` | 2026-11-07 |
| `/definitions/TargetGroupTuple/properties/Weight` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_TargetGroupTuple.html) | ` The weight. The range is 0 to 999. ` | 2026-11-07 |
| `/properties/AlpnPolicy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_CreateListener.html) | ` [TLS listeners] The name of the Application-Layer Protocol Negotiation (ALPN) policy. You can specify one policy name. The following are the possible values: `<br>` HTTP1Only `<br>` HTTP2Only `<br>` HTTP2Optional `<br>` HTTP2Preferred `<br>` None `<br>` For more information, see ALPN policies in the Network Load Balancers Guide. ` | 2026-11-07 |
| `/properties/Certificates` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_CreateListener.html) | ` [HTTPS and TLS listeners] The default certificate for the listener. You must provide exactly one certificate. Set CertificateArn to the certificate ARN but do not set IsDefault. `<br>` To create a certificate list for the listener, use AddListenerCertificates. ` | 2026-11-07 |
| `/properties/Port` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_CreateListener.html) | ` The port on which the load balancer is listening. You can't specify a port for a Gateway Load Balancer. `<br>` Valid Range: Minimum value of 1. Maximum value of 65535. ` | 2026-11-07 |

