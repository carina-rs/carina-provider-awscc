---
title: "awscc.cloudfront.Function"
---


CloudFormation Type: `AWS::CloudFront::Function`

Creates a CF function.
 To create a function, you provide the function code and some configuration information about the function. The response contains an Amazon Resource Name (ARN) that uniquely identifies the function, and the function?s stage.
 By default, when you create a function, it?s in the ``DEVELOPMENT`` stage. In this stage, you can [test the function](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/test-function.html) in the CF console (or with ``TestFunction`` in the CF API).
 When you?re ready to use your function with a CF distribution, publish the function to the ``LIVE`` stage. You can do this in the CF console, with ``PublishFunction`` in the CF API, or by updating the ``AWS::CloudFront::Function`` resource with the ``AutoPublish`` property set to ``true``. When the function is published to the ``LIVE`` stage, you can attach it to a distribution?s cache behavior, using the function?s ARN.
 To automatically publish the function to the ``LIVE`` stage when it?s created, set the ``AutoPublish`` property to ``true``.

## Example

```crn
let viewer_request_function = awscc.cloudfront.Function {
  name = 'carina-registry-viewer-request'

  function_code = <<'JS'
function handler(event) {
  var request = event.request;
  if (request.uri === '/') {
    request.uri = '/providers/';
  }
  return request;
}
JS

  function_config = {
    runtime = cloudfront_js_2_0
    comment = 'Rewrite the registry root request'
  }

  auto_publish = true
}

awscc.cloudfront.Distribution {
  distribution_config = {
    comment      = 'Distribution using the viewer-request function'
    enabled      = false
    price_class  = price_class_100
    http_version = http2
    ipv6_enabled = false

    default_cache_behavior = {
      target_origin_id       = 'example-origin'
      viewer_protocol_policy = allow_all
      allowed_methods        = [get, head]
      cached_methods         = [get, head]

      function_association {
        event_type   = viewer_request
        function_arn = viewer_request_function.function_arn
      }

      forwarded_values = {
        query_string = false
        cookies = {
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

    restrictions = {
      geo_restriction = {
        restriction_type = none
      }
    }

    viewer_certificate = {
      cloud_front_default_certificate = true
    }
  }
}
```

## Argument Reference

### `auto_publish`

- **Type:** Bool
- **Required:** No
- **Write-only:** Yes

A flag that determines whether to automatically publish the function to the ``LIVE`` stage when it?s created. To automatically publish to the ``LIVE`` stage, set this property to ``true``.

### `function_code`

- **Type:** String
- **Required:** Yes

The function code. For more information about writing a CloudFront function, see [Writing function code for CloudFront Functions](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/writing-function-code.html) in the *Amazon CloudFront Developer Guide*.

### `function_config`

- **Type:** [Struct(FunctionConfig)](#function_config-1)
- **Required:** Yes

Contains configuration information about a CloudFront function.

### `name`

- **Type:** String
- **Required:** Yes
- **Create-only:** Yes

A name to identify the function.

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

A complex type that contains zero or more ``Tag`` elements.

## Enum Values

### function_config.runtime (Runtime)

| Value | DSL Identifier |
|-------|----------------|
| `cloudfront-js-1.0` | `aws.cloudfront.Function.FunctionConfig.Runtime.cloudfront_js_1_0` |
| `cloudfront-js-2.0` | `aws.cloudfront.Function.FunctionConfig.Runtime.cloudfront_js_2_0` |

Shorthand formats: `cloudfront_js_1_0` or `Runtime.cloudfront_js_1_0`

## Struct Definitions

### function_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `comment` | String | Yes | A comment to describe the function. The comment cannot be longer than 128 characters. |
| `key_value_store_associations` | List\<[Struct(KeyValueStoreAssociation)](#function_configkey_value_store_associations)\> | No | The configuration for the key value store associations. |
| `runtime` | [Enum (Runtime)](#function_configruntime-runtime) | Yes | The function's runtime environment version. |

### function_config.key_value_store_associations

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key_value_store_arn` | Arn | Yes | The Amazon Resource Name (ARN) of the key value store association. |

### function_metadata

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `function_arn` | Arn | No | The Amazon Resource Name (ARN) of the function. The ARN uniquely identifies the function. Provider-populated; cannot be set. |

## Attribute Reference

### `function_arn`

- **Type:** Arn



### `function_metadata`

- **Type:** [Struct(FunctionMetadata)](#function_metadata)

Contains metadata about a CloudFront function.

### `stage`

- **Type:** String



