---
title: "awscc.cloudfront.OriginAccessControl"
description: "AWSCC CloudFront OriginAccessControl resource reference"
---


CloudFormation Type: `AWS::CloudFront::OriginAccessControl`

Creates a new origin access control in CloudFront. After you create an origin access control, you can add it to an origin in a CloudFront distribution so that CloudFront sends authenticated (signed) requests to the origin.
 This makes it possible to block public access to the origin, allowing viewers (users) to access the origin's content only through CloudFront.
 For more information about using a CloudFront origin access control, see [Restricting access to an origin](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/private-content-restricting-access-to-origin.html) in the *Amazon CloudFront Developer Guide*.

## Argument Reference

### `origin_access_control_config`

- **Type:** [Struct(OriginAccessControlConfig)](#originaccesscontrolconfig)
- **Required:** Yes

The origin access control.

## Enum Values

### origin_access_control_origin_type (OriginAccessControlOriginType)

| Value | DSL Identifier |
|-------|----------------|
| `s3` | `aws.cloudfront.OriginAccessControl.OriginAccessControlConfig.OriginAccessControlOriginType.s3` |
| `mediastore` | `aws.cloudfront.OriginAccessControl.OriginAccessControlConfig.OriginAccessControlOriginType.mediastore` |
| `lambda` | `aws.cloudfront.OriginAccessControl.OriginAccessControlConfig.OriginAccessControlOriginType.lambda` |
| `mediapackagev2` | `aws.cloudfront.OriginAccessControl.OriginAccessControlConfig.OriginAccessControlOriginType.mediapackagev2` |

Shorthand formats: `s3` or `OriginAccessControlOriginType.s3`

### signing_behavior (SigningBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `always` | `aws.cloudfront.OriginAccessControl.OriginAccessControlConfig.SigningBehavior.always` |
| `never` | `aws.cloudfront.OriginAccessControl.OriginAccessControlConfig.SigningBehavior.never` |
| `no-override` | `aws.cloudfront.OriginAccessControl.OriginAccessControlConfig.SigningBehavior.no_override` |

Shorthand formats: `always` or `SigningBehavior.always`

### signing_protocol (SigningProtocol)

| Value | DSL Identifier |
|-------|----------------|
| `sigv4` | `aws.cloudfront.OriginAccessControl.OriginAccessControlConfig.SigningProtocol.sigv4` |

Shorthand formats: `sigv4` or `SigningProtocol.sigv4`

## Struct Definitions

### OriginAccessControlConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `description` | String | No | A description of the origin access control. |
| `name` | String | Yes | A name to identify the origin access control. You can specify up to 64 characters. |
| `origin_access_control_origin_type` | [Enum (OriginAccessControlOriginType)](#origin_access_control_origin_type-originaccesscontrolorigintype) | Yes | The type of origin that this origin access control is for. |
| `signing_behavior` | [Enum (SigningBehavior)](#signing_behavior-signingbehavior) | Yes | Specifies which requests CloudFront signs (adds authentication information to). Specify ``always`` for the most common use case. For more information, see [origin access control advanced settings](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/private-content-restricting-access-to-s3.html#oac-advanced-settings) in the *Amazon CloudFront Developer Guide*. This field can have one of the following values: + ``always`` – CloudFront signs all origin requests, overwriting the ``Authorization`` header from the viewer request if one exists. + ``never`` – CloudFront doesn't sign any origin requests. This value turns off origin access control for all origins in all distributions that use this origin access control. + ``no-override`` – If the viewer request doesn't contain the ``Authorization`` header, then CloudFront signs the origin request. If the viewer request contains the ``Authorization`` header, then CloudFront doesn't sign the origin request and instead passes along the ``Authorization`` header from the viewer request. *WARNING: To pass along the Authorization header from the viewer request, you must add the Authorization header to a cache policy for all cache behaviors that use origins associated with this origin access control.* |
| `signing_protocol` | [Enum (SigningProtocol)](#signing_protocol-signingprotocol) | Yes | The signing protocol of the origin access control, which determines how CloudFront signs (authenticates) requests. The only valid value is ``sigv4``. |

## Attribute Reference

### `id`

- **Type:** String



