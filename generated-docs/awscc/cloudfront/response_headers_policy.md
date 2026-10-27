---
title: "awscc.cloudfront.ResponseHeadersPolicy"
description: "AWSCC CloudFront ResponseHeadersPolicy resource reference"
---


CloudFormation Type: `AWS::CloudFront::ResponseHeadersPolicy`

A response headers policy.
 A response headers policy contains information about a set of HTTP response headers.
 After you create a response headers policy, you can use its ID to attach it to one or more cache behaviors in a CloudFront distribution. When it's attached to a cache behavior, the response headers policy affects the HTTP headers that CloudFront includes in HTTP responses to requests that match the cache behavior. CloudFront adds or removes response headers according to the configuration of the response headers policy.
 For more information, see [Adding or removing HTTP headers in CloudFront responses](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/modifying-response-headers.html) in the *Amazon CloudFront Developer Guide*.

## Example

```crn
let response_headers_policy = awscc.cloudfront.ResponseHeadersPolicy {
  response_headers_policy_config = {
    name                    = 'carina-registry-security-headers'
    comment                 = 'Security headers for the registry read plane'

    security_headers_config = {
      content_security_policy = {
        content_security_policy = "default-src 'self'"
        override                = true
      }

      content_type_options    = {
        override = true
      }
    }
  }
}

awscc.cloudfront.Distribution {
  distribution_config = {
    comment                = 'Distribution using the response headers policy'
    enabled                = false
    price_class            = price_class_100
    http_version           = http2
    ipv6_enabled           = false

    default_cache_behavior = {
      target_origin_id           = 'example-origin'
      viewer_protocol_policy     = allow_all
      allowed_methods            = [get, head]
      cached_methods             = [get, head]
      response_headers_policy_id = response_headers_policy.id

      forwarded_values           = {
        query_string = false
        cookies      = {
          forward = none
        }
      }
    }

    origin {
      id          = 'example-origin'
      domain_name = 'example.com'

      custom_origin_config = {
        http_port              = 80
        https_port             = 443
        origin_protocol_policy = http_only
        origin_ssl_protocols   = [tlsv1_2]
      }
    }

    restrictions           = {
      geo_restriction = {
        restriction_type = none
      }
    }

    viewer_certificate     = {
      cloud_front_default_certificate = true
    }
  }
}
```

## Argument Reference

### `response_headers_policy_config`

- **Type:** [Struct(ResponseHeadersPolicyConfig)](#responseheaderspolicyconfig)
- **Required:** Yes

A response headers policy configuration.

## Enum Values

### items (Items)

| Value | DSL Identifier |
|-------|----------------|
| `GET` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.CorsConfig.AccessControlAllowMethods.Items.get` |
| `DELETE` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.CorsConfig.AccessControlAllowMethods.Items.delete` |
| `HEAD` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.CorsConfig.AccessControlAllowMethods.Items.head` |
| `OPTIONS` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.CorsConfig.AccessControlAllowMethods.Items.options` |
| `PATCH` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.CorsConfig.AccessControlAllowMethods.Items.patch` |
| `POST` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.CorsConfig.AccessControlAllowMethods.Items.post` |
| `PUT` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.CorsConfig.AccessControlAllowMethods.Items.put` |
| `ALL` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.CorsConfig.AccessControlAllowMethods.Items.all` |

Shorthand formats: `get` or `Items.get`

### frame_option (FrameOption)

| Value | DSL Identifier |
|-------|----------------|
| `DENY` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.SecurityHeadersConfig.FrameOptions.FrameOption.deny` |
| `SAMEORIGIN` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.SecurityHeadersConfig.FrameOptions.FrameOption.sameorigin` |

Shorthand formats: `deny` or `FrameOption.deny`

### referrer_policy (ReferrerPolicy)

| Value | DSL Identifier |
|-------|----------------|
| `no-referrer` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.SecurityHeadersConfig.ReferrerPolicy.ReferrerPolicy.no_referrer` |
| `no-referrer-when-downgrade` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.SecurityHeadersConfig.ReferrerPolicy.ReferrerPolicy.no_referrer_when_downgrade` |
| `origin` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.SecurityHeadersConfig.ReferrerPolicy.ReferrerPolicy.origin` |
| `origin-when-cross-origin` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.SecurityHeadersConfig.ReferrerPolicy.ReferrerPolicy.origin_when_cross_origin` |
| `same-origin` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.SecurityHeadersConfig.ReferrerPolicy.ReferrerPolicy.same_origin` |
| `strict-origin` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.SecurityHeadersConfig.ReferrerPolicy.ReferrerPolicy.strict_origin` |
| `strict-origin-when-cross-origin` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.SecurityHeadersConfig.ReferrerPolicy.ReferrerPolicy.strict_origin_when_cross_origin` |
| `unsafe-url` | `aws.cloudfront.ResponseHeadersPolicy.ResponseHeadersPolicyConfig.SecurityHeadersConfig.ReferrerPolicy.ReferrerPolicy.unsafe_url` |

Shorthand formats: `no_referrer` or `ReferrerPolicy.no_referrer`

## Struct Definitions

### AccessControlAllowHeaders

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `items` | `List<String>` | Yes | The list of HTTP header names. You can specify ``*`` to allow all headers. |

### AccessControlAllowMethods

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `items` | List\<[Enum (Items)](#items-items)\> | Yes | The list of HTTP methods. Valid values are: + ``GET`` + ``DELETE`` + ``HEAD`` + ``OPTIONS`` + ``PATCH`` + ``POST`` + ``PUT`` + ``ALL`` ``ALL`` is a special value that includes all of the listed HTTP methods. |

### AccessControlAllowOrigins

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `items` | `List<String>` | Yes | The list of origins (domain names). You can specify ``*`` to allow all origins. |

### AccessControlExposeHeaders

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `items` | `List<String>` | Yes | The list of HTTP headers. You can specify ``*`` to expose all headers. |

### ContentSecurityPolicy

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content_security_policy` | String | Yes | The policy directives and their values that CloudFront includes as values for the ``Content-Security-Policy`` HTTP response header. For more information about the ``Content-Security-Policy`` HTTP response header, see [Content-Security-Policy](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy) in the MDN Web Docs. |
| `override` | Bool | Yes | A Boolean that determines whether CloudFront overrides the ``Content-Security-Policy`` HTTP response header received from the origin with the one specified in this response headers policy. |

### ContentTypeOptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `override` | Bool | Yes | A Boolean that determines whether CloudFront overrides the ``X-Content-Type-Options`` HTTP response header received from the origin with the one specified in this response headers policy. |

### CorsConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `access_control_allow_credentials` | Bool | Yes | A Boolean that CloudFront uses as the value for the ``Access-Control-Allow-Credentials`` HTTP response header. For more information about the ``Access-Control-Allow-Credentials`` HTTP response header, see [Access-Control-Allow-Credentials](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials) in the MDN Web Docs. |
| `access_control_allow_headers` | [Struct(AccessControlAllowHeaders)](#accesscontrolallowheaders) | Yes | A list of HTTP header names that CloudFront includes as values for the ``Access-Control-Allow-Headers`` HTTP response header. For more information about the ``Access-Control-Allow-Headers`` HTTP response header, see [Access-Control-Allow-Headers](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers) in the MDN Web Docs. |
| `access_control_allow_methods` | [Struct(AccessControlAllowMethods)](#accesscontrolallowmethods) | Yes | A list of HTTP methods that CloudFront includes as values for the ``Access-Control-Allow-Methods`` HTTP response header. For more information about the ``Access-Control-Allow-Methods`` HTTP response header, see [Access-Control-Allow-Methods](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Methods) in the MDN Web Docs. |
| `access_control_allow_origins` | [Struct(AccessControlAllowOrigins)](#accesscontrolalloworigins) | Yes | A list of origins (domain names) that CloudFront can use as the value for the ``Access-Control-Allow-Origin`` HTTP response header. For more information about the ``Access-Control-Allow-Origin`` HTTP response header, see [Access-Control-Allow-Origin](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin) in the MDN Web Docs. |
| `access_control_expose_headers` | [Struct(AccessControlExposeHeaders)](#accesscontrolexposeheaders) | No | A list of HTTP headers that CloudFront includes as values for the ``Access-Control-Expose-Headers`` HTTP response header. For more information about the ``Access-Control-Expose-Headers`` HTTP response header, see [Access-Control-Expose-Headers](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Expose-Headers) in the MDN Web Docs. |
| `access_control_max_age_sec` | Int | No | A number that CloudFront uses as the value for the ``Access-Control-Max-Age`` HTTP response header. For more information about the ``Access-Control-Max-Age`` HTTP response header, see [Access-Control-Max-Age](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Max-Age) in the MDN Web Docs. |
| `origin_override` | Bool | Yes | A Boolean that determines whether CloudFront overrides HTTP response headers received from the origin with the ones specified in this response headers policy. |

### CustomHeader

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `header` | String | Yes | The HTTP response header name. |
| `override` | Bool | Yes | A Boolean that determines whether CloudFront overrides a response header with the same name received from the origin with the header specified here. |
| `value` | String | Yes | The value for the HTTP response header. |

### CustomHeadersConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `items` | [List\<CustomHeader\>](#customheader) | Yes | The list of HTTP response headers and their values. |

### FrameOptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `frame_option` | [Enum (FrameOption)](#frame_option-frameoption) | Yes | The value of the ``X-Frame-Options`` HTTP response header. Valid values are ``DENY`` and ``SAMEORIGIN``. For more information about these values, see [X-Frame-Options](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Frame-Options) in the MDN Web Docs. |
| `override` | Bool | Yes | A Boolean that determines whether CloudFront overrides the ``X-Frame-Options`` HTTP response header received from the origin with the one specified in this response headers policy. |

### ReferrerPolicy

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `override` | Bool | Yes | A Boolean that determines whether CloudFront overrides the ``Referrer-Policy`` HTTP response header received from the origin with the one specified in this response headers policy. |
| `referrer_policy` | [Enum (ReferrerPolicy)](#referrer_policy-referrerpolicy) | Yes | Determines whether CloudFront includes the ``Referrer-Policy`` HTTP response header and the header's value. For more information about the ``Referrer-Policy`` HTTP response header, see [Referrer-Policy](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referrer-Policy) in the MDN Web Docs. |

### RemoveHeader

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `header` | String | Yes | The HTTP header name. |

### RemoveHeadersConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `items` | [List\<RemoveHeader\>](#removeheader) | Yes | The list of HTTP header names. |

### ResponseHeadersPolicyConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `comment` | String | No | A comment to describe the response headers policy. The comment cannot be longer than 128 characters. |
| `cors_config` | [Struct(CorsConfig)](#corsconfig) | No | A configuration for a set of HTTP response headers that are used for cross-origin resource sharing (CORS). |
| `custom_headers_config` | [Struct(CustomHeadersConfig)](#customheadersconfig) | No | A configuration for a set of custom HTTP response headers. |
| `name` | String | Yes | A name to identify the response headers policy. The name must be unique for response headers policies in this AWS-account. |
| `remove_headers_config` | [Struct(RemoveHeadersConfig)](#removeheadersconfig) | No | A configuration for a set of HTTP headers to remove from the HTTP response. |
| `security_headers_config` | [Struct(SecurityHeadersConfig)](#securityheadersconfig) | No | A configuration for a set of security-related HTTP response headers. |
| `server_timing_headers_config` | [Struct(ServerTimingHeadersConfig)](#servertimingheadersconfig) | No | A configuration for enabling the ``Server-Timing`` header in HTTP responses sent from CloudFront. |

### SecurityHeadersConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content_security_policy` | [Struct(ContentSecurityPolicy)](#contentsecuritypolicy) | No | The policy directives and their values that CloudFront includes as values for the ``Content-Security-Policy`` HTTP response header. For more information about the ``Content-Security-Policy`` HTTP response header, see [Content-Security-Policy](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy) in the MDN Web Docs. |
| `content_type_options` | [Struct(ContentTypeOptions)](#contenttypeoptions) | No | Determines whether CloudFront includes the ``X-Content-Type-Options`` HTTP response header with its value set to ``nosniff``. For more information about the ``X-Content-Type-Options`` HTTP response header, see [X-Content-Type-Options](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options) in the MDN Web Docs. |
| `frame_options` | [Struct(FrameOptions)](#frameoptions) | No | Determines whether CloudFront includes the ``X-Frame-Options`` HTTP response header and the header's value. For more information about the ``X-Frame-Options`` HTTP response header, see [X-Frame-Options](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Frame-Options) in the MDN Web Docs. |
| `referrer_policy` | [Struct(ReferrerPolicy)](#referrerpolicy) | No | Determines whether CloudFront includes the ``Referrer-Policy`` HTTP response header and the header's value. For more information about the ``Referrer-Policy`` HTTP response header, see [Referrer-Policy](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referrer-Policy) in the MDN Web Docs. |
| `strict_transport_security` | [Struct(StrictTransportSecurity)](#stricttransportsecurity) | No | Determines whether CloudFront includes the ``Strict-Transport-Security`` HTTP response header and the header's value. For more information about the ``Strict-Transport-Security`` HTTP response header, see [Security headers](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/understanding-response-headers-policies.html#understanding-response-headers-policies-security) in the *Amazon CloudFront Developer Guide* and [Strict-Transport-Security](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Strict-Transport-Security) in the MDN Web Docs. |
| `xss_protection` | [Struct(XSSProtection)](#xssprotection) | No | Determines whether CloudFront includes the ``X-XSS-Protection`` HTTP response header and the header's value. For more information about the ``X-XSS-Protection`` HTTP response header, see [X-XSS-Protection](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-XSS-Protection) in the MDN Web Docs. |

### ServerTimingHeadersConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | Bool | Yes | A Boolean that determines whether CloudFront adds the ``Server-Timing`` header to HTTP responses that it sends in response to requests that match a cache behavior that's associated with this response headers policy. |
| `sampling_rate` | Float(0..=100) | No | A number 0?100 (inclusive) that specifies the percentage of responses that you want CloudFront to add the ``Server-Timing`` header to. When you set the sampling rate to 100, CloudFront adds the ``Server-Timing`` header to the HTTP response for every request that matches the cache behavior that this response headers policy is attached to. When you set it to 50, CloudFront adds the header to 50% of the responses for requests that match the cache behavior. You can set the sampling rate to any number 0?100 with up to four decimal places. |

### StrictTransportSecurity

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `access_control_max_age_sec` | Int | Yes | A number that CloudFront uses as the value for the ``max-age`` directive in the ``Strict-Transport-Security`` HTTP response header. |
| `include_subdomains` | Bool | No | A Boolean that determines whether CloudFront includes the ``includeSubDomains`` directive in the ``Strict-Transport-Security`` HTTP response header. |
| `override` | Bool | Yes | A Boolean that determines whether CloudFront overrides the ``Strict-Transport-Security`` HTTP response header received from the origin with the one specified in this response headers policy. |
| `preload` | Bool | No | A Boolean that determines whether CloudFront includes the ``preload`` directive in the ``Strict-Transport-Security`` HTTP response header. |

### XSSProtection

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `mode_block` | Bool | No | A Boolean that determines whether CloudFront includes the ``mode=block`` directive in the ``X-XSS-Protection`` header. For more information about this directive, see [X-XSS-Protection](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-XSS-Protection) in the MDN Web Docs. |
| `override` | Bool | Yes | A Boolean that determines whether CloudFront overrides the ``X-XSS-Protection`` HTTP response header received from the origin with the one specified in this response headers policy. |
| `protection` | Bool | Yes | A Boolean that determines the value of the ``X-XSS-Protection`` HTTP response header. When this setting is ``true``, the value of the ``X-XSS-Protection`` header is ``1``. When this setting is ``false``, the value of the ``X-XSS-Protection`` header is ``0``. For more information about these settings, see [X-XSS-Protection](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-XSS-Protection) in the MDN Web Docs. |
| `report_uri` | String | No | A reporting URI, which CloudFront uses as the value of the ``report`` directive in the ``X-XSS-Protection`` header. You cannot specify a ``ReportUri`` when ``ModeBlock`` is ``true``. For more information about using a reporting URL, see [X-XSS-Protection](https://docs.aws.amazon.com/https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-XSS-Protection) in the MDN Web Docs. |

## Attribute Reference

### `id`

- **Type:** String



### `last_modified_time`

- **Type:** String



