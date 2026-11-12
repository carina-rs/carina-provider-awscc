---
title: "awscc.s3.BucketPolicy"
---


CloudFormation Type: `AWS::S3::BucketPolicy`

Applies an Amazon S3 bucket policy to an Amazon S3 bucket. If you are using an identity other than the root user of the AWS-account that owns the bucket, the calling identity must have the ``PutBucketPolicy`` permissions on the specified bucket and belong to the bucket owner's account in order to use this operation.
 If you don't have ``PutBucketPolicy`` permissions, Amazon S3 returns a ``403 Access Denied`` error. If you have the correct permissions, but you're not using an identity that belongs to the bucket owner's account, Amazon S3 returns a ``405 Method Not Allowed`` error.
   As a security precaution, the root user of the AWS-account that owns a bucket can always use this operation, even if the policy explicitly denies the root user the ability to perform this action. 
  When using the ``AWS::S3::BucketPolicy`` resource, you can create, update, and delete bucket policies for S3 buckets located in Regions that are different from the stack's Region. However, the CloudFormation stacks should be deployed in the US East (N. Virginia) or ``us-east-1`` Region. This cross-region bucket policy modification functionality is supported for backward compatibility with existing workflows.
  If the [DeletionPolicy attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-deletionpolicy.html) is not specified or set to ``Delete``, the bucket policy will be removed when the stack is deleted. If set to ``Retain``, the bucket policy will be preserved even after the stack is deleted.
  For example, a CloudFormation stack in ``us-east-1`` can use the ``AWS::S3::BucketPolicy`` resource to manage the bucket policy for an S3 bucket in ``us-west-2``. The retention or removal of the bucket policy during the stack deletion is determined by the ``DeletionPolicy`` attribute specified in the stack template.
 For more information, see [Bucket policy examples](https://docs.aws.amazon.com/AmazonS3/latest/userguide/example-bucket-policies.html).
 The following operations are related to ``PutBucketPolicy``:
  +   [CreateBucket](https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateBucket.html) 
  +   [DeleteBucket](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucket.html)

## Example

```crn
let example_bucket = awscc.s3.Bucket {
  bucket_name = 'carina-example-policy-bucket'

  tags = {
    Environment = 'example'
  }
}

let bucket_reader = awscc.iam.Role {
  assume_role_policy_document = {
    version = aws.iam.PolicyDocument.Version.2012_10_17
    statement {
      effect = aws.iam.PolicyDocument.Statement.Effect.allow

      principal = {
        service = 'lambda.amazonaws.com'
      }

      action = 'sts:AssumeRole'
    }
  }
}

awscc.s3.BucketPolicy {
  bucket = example_bucket.bucket_name

  policy_document = {
    version = aws.iam.PolicyDocument.Version.2012_10_17
    statement {
      sid    = 'AllowReaderRoleToListBucket'
      effect = aws.iam.PolicyDocument.Statement.Effect.allow

      principal = {
        aws = bucket_reader.arn
      }

      action   = 's3:ListBucket'
      resource = example_bucket.arn
    }
  }
}
```

## Argument Reference

### `bucket`

- **Type:** `String`
- **Required:** Yes
- **Create-only:** Yes

The name of the Amazon S3 bucket to which the policy applies.

### `policy_document`

- **Type:** [Struct(PolicyDocument)](#policy_document-1)
- **Required:** Yes

A policy document containing permissions to add to the specified bucket. In IAM, you must provide policy documents in JSON format. However, in CloudFormation you can provide the policy in JSON or YAML format because CloudFormation converts YAML to JSON before submitting it to IAM. For more information, see the AWS::IAM::Policy [PolicyDocument](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-iam-policy.html#cfn-iam-policy-policydocument) resource description in this guide and [Access Policy Language Overview](https://docs.aws.amazon.com/AmazonS3/latest/dev/access-policy-language-overview.html) in the *Amazon S3 User Guide*.

## Enum Values

### policy_document.version (Version)

| Value | DSL Identifier |
|-------|----------------|
| `2012-10-17` | `aws.iam.PolicyDocument.Version.2012_10_17` |
| `2008-10-17` | `aws.iam.PolicyDocument.Version.2008_10_17` |

Use the fully qualified DSL identifiers above; bare numeric values parse as integers.

### statement.condition.\<key\> (ConditionOperator)

| Value | DSL Identifier |
|-------|----------------|
| `string_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals` |
| `string_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals_if_exists` |
| `for_all_values_string_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals` |
| `for_all_values_string_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals_if_exists` |
| `for_any_value_string_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals` |
| `for_any_value_string_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals_if_exists` |
| `string_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals` |
| `string_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals_if_exists` |
| `for_all_values_string_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals` |
| `for_all_values_string_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals_if_exists` |
| `for_any_value_string_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals` |
| `for_any_value_string_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals_if_exists` |
| `string_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals_ignore_case` |
| `string_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals_ignore_case_if_exists` |
| `for_all_values_string_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals_ignore_case` |
| `for_all_values_string_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals_ignore_case_if_exists` |
| `for_any_value_string_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals_ignore_case` |
| `for_any_value_string_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals_ignore_case_if_exists` |
| `string_not_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals_ignore_case` |
| `string_not_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals_ignore_case_if_exists` |
| `for_all_values_string_not_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals_ignore_case` |
| `for_all_values_string_not_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals_ignore_case_if_exists` |
| `for_any_value_string_not_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals_ignore_case` |
| `for_any_value_string_not_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals_ignore_case_if_exists` |
| `string_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_like` |
| `string_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_like_if_exists` |
| `for_all_values_string_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_like` |
| `for_all_values_string_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_like_if_exists` |
| `for_any_value_string_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_like` |
| `for_any_value_string_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_like_if_exists` |
| `string_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_like` |
| `string_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_like_if_exists` |
| `for_all_values_string_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_like` |
| `for_all_values_string_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_like_if_exists` |
| `for_any_value_string_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_like` |
| `for_any_value_string_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_like_if_exists` |
| `numeric_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_equals` |
| `numeric_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_equals_if_exists` |
| `for_all_values_numeric_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_equals` |
| `for_all_values_numeric_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_equals_if_exists` |
| `for_any_value_numeric_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_equals` |
| `for_any_value_numeric_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_equals_if_exists` |
| `numeric_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_not_equals` |
| `numeric_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_not_equals_if_exists` |
| `for_all_values_numeric_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_not_equals` |
| `for_all_values_numeric_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_not_equals_if_exists` |
| `for_any_value_numeric_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_not_equals` |
| `for_any_value_numeric_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_not_equals_if_exists` |
| `numeric_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than` |
| `numeric_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than_if_exists` |
| `for_all_values_numeric_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than` |
| `for_all_values_numeric_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than_if_exists` |
| `for_any_value_numeric_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than` |
| `for_any_value_numeric_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than_if_exists` |
| `numeric_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than_equals` |
| `numeric_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than_equals_if_exists` |
| `for_all_values_numeric_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than_equals` |
| `for_all_values_numeric_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than_equals_if_exists` |
| `for_any_value_numeric_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than_equals` |
| `for_any_value_numeric_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than_equals_if_exists` |
| `numeric_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than` |
| `numeric_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than_if_exists` |
| `for_all_values_numeric_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than` |
| `for_all_values_numeric_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than_if_exists` |
| `for_any_value_numeric_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than` |
| `for_any_value_numeric_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than_if_exists` |
| `numeric_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than_equals` |
| `numeric_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than_equals_if_exists` |
| `for_all_values_numeric_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than_equals` |
| `for_all_values_numeric_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than_equals_if_exists` |
| `for_any_value_numeric_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than_equals` |
| `for_any_value_numeric_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than_equals_if_exists` |
| `date_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_equals` |
| `date_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_equals_if_exists` |
| `for_all_values_date_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_equals` |
| `for_all_values_date_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_equals_if_exists` |
| `for_any_value_date_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_equals` |
| `for_any_value_date_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_equals_if_exists` |
| `date_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_not_equals` |
| `date_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_not_equals_if_exists` |
| `for_all_values_date_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_not_equals` |
| `for_all_values_date_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_not_equals_if_exists` |
| `for_any_value_date_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_not_equals` |
| `for_any_value_date_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_not_equals_if_exists` |
| `date_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than` |
| `date_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than_if_exists` |
| `for_all_values_date_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than` |
| `for_all_values_date_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than_if_exists` |
| `for_any_value_date_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than` |
| `for_any_value_date_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than_if_exists` |
| `date_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than_equals` |
| `date_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than_equals_if_exists` |
| `for_all_values_date_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than_equals` |
| `for_all_values_date_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than_equals_if_exists` |
| `for_any_value_date_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than_equals` |
| `for_any_value_date_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than_equals_if_exists` |
| `date_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than` |
| `date_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than_if_exists` |
| `for_all_values_date_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than` |
| `for_all_values_date_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than_if_exists` |
| `for_any_value_date_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than` |
| `for_any_value_date_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than_if_exists` |
| `date_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than_equals` |
| `date_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than_equals_if_exists` |
| `for_all_values_date_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than_equals` |
| `for_all_values_date_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than_equals_if_exists` |
| `for_any_value_date_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than_equals` |
| `for_any_value_date_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than_equals_if_exists` |
| `bool` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.bool` |
| `bool_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.bool_if_exists` |
| `for_all_values_bool` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_bool` |
| `for_all_values_bool_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_bool_if_exists` |
| `for_any_value_bool` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_bool` |
| `for_any_value_bool_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_bool_if_exists` |
| `binary_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.binary_equals` |
| `binary_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.binary_equals_if_exists` |
| `for_all_values_binary_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_binary_equals` |
| `for_all_values_binary_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_binary_equals_if_exists` |
| `for_any_value_binary_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_binary_equals` |
| `for_any_value_binary_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_binary_equals_if_exists` |
| `ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.ip_address` |
| `ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.ip_address_if_exists` |
| `for_all_values_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_ip_address` |
| `for_all_values_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_ip_address_if_exists` |
| `for_any_value_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_ip_address` |
| `for_any_value_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_ip_address_if_exists` |
| `not_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.not_ip_address` |
| `not_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.not_ip_address_if_exists` |
| `for_all_values_not_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_not_ip_address` |
| `for_all_values_not_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_not_ip_address_if_exists` |
| `for_any_value_not_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_not_ip_address` |
| `for_any_value_not_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_not_ip_address_if_exists` |
| `arn_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_equals` |
| `arn_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_equals_if_exists` |
| `for_all_values_arn_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_equals` |
| `for_all_values_arn_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_equals_if_exists` |
| `for_any_value_arn_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_equals` |
| `for_any_value_arn_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_equals_if_exists` |
| `arn_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_equals` |
| `arn_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_equals_if_exists` |
| `for_all_values_arn_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_equals` |
| `for_all_values_arn_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_equals_if_exists` |
| `for_any_value_arn_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_equals` |
| `for_any_value_arn_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_equals_if_exists` |
| `arn_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_like` |
| `arn_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_like_if_exists` |
| `for_all_values_arn_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_like` |
| `for_all_values_arn_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_like_if_exists` |
| `for_any_value_arn_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_like` |
| `for_any_value_arn_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_like_if_exists` |
| `arn_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_like` |
| `arn_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_like_if_exists` |
| `for_all_values_arn_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_like` |
| `for_all_values_arn_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_like_if_exists` |
| `for_any_value_arn_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_like` |
| `for_any_value_arn_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_like_if_exists` |
| `null` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.null` |
| `null_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.null_if_exists` |
| `for_all_values_null` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_null` |
| `for_all_values_null_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_null_if_exists` |
| `for_any_value_null` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_null` |
| `for_any_value_null_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_null_if_exists` |

Shorthand formats: `string_equals` or `ConditionOperator.string_equals`

### statement.effect (Effect)

| Value | DSL Identifier |
|-------|----------------|
| `Allow` | `aws.iam.PolicyDocument.Statement.Effect.allow` |
| `Deny` | `aws.iam.PolicyDocument.Statement.Effect.deny` |

Shorthand formats: `allow` or `Effect.allow`

## Struct Definitions

### policy_document

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | [Version](#policy_documentversion-version) | No |  |
| `id` | `String` | No |  |
| `statement` | [Union(policy\_document.statement)](#policy_documentstatement) | Yes |  |

### principal

Used by: `union_of_principal_or_string.member_0`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `service` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `aws` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `federated` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `canonical_user` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |

**Field groups:**

- At least one of: `service`, `aws`, `federated`, `canonical_user`.

### statement

Used by: `policy_document.statement.member_0`, `policy_document.statement.member_1`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sid` | `String` | No |  |
| `effect` | [Effect](#statementeffect-effect) | Yes |  |
| `action` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `not_action` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `resource` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `not_resource` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `principal` | [Union(union\_of\_principal\_or\_string)](#union_of_principal_or_string) | No |  |
| `not_principal` | [Union(union\_of\_principal\_or\_string)](#union_of_principal_or_string) | No |  |
| `condition` | Map\<[ConditionOperator](#statementconditionkey-conditionoperator), Map\<`String`, [Union(statement.condition.\<value\>.\<value\>)](#statementconditionvaluevalue)\>\> | No |  |

**Field groups:**

- Exactly one of: `action`, `not_action`.
- At most one of: `resource`, `not_resource`.
- At most one of: `principal`, `not_principal`.

## Union Definitions

### policy_document.statement

| Member | Type |
|--------|------|
| `member_0` | List\<[Struct(Statement)](#statement)\> (items: 1..unbounded, ordered) |
| `member_1` | [Struct(Statement)](#statement) |

### statement.condition.\<value\>.\<value\>

| Member | Type |
|--------|------|
| `member_0` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) |
| `member_1` | `Int` |
| `member_2` | `Float` |
| `member_3` | `Bool` |
| `member_4` | List\<[Union(union\_of\_string\_or\_int\_or\_float\_or\_bool)](#union_of_string_or_int_or_float_or_bool)\> (items: 1..unbounded, unordered) |

### union_of_principal_or_string

Used by: `statement.not_principal`, `statement.principal`

| Member | Type |
|--------|------|
| `member_0` | [Struct(Principal)](#principal) |
| `member_1` | `String (pattern: /^\*\z/ (Full))` |

### union_of_string_or_int_or_float_or_bool

Used by: `statement.condition.<value>.<value>.member_4`

| Member | Type |
|--------|------|
| `member_0` | `String` |
| `member_1` | `Int` |
| `member_2` | `Float` |
| `member_3` | `Bool` |

### union_of_string_or_list_of_string

Used by: `principal.aws`, `principal.canonical_user`, `principal.federated`, `principal.service`, `statement.action`, `statement.not_action`, `statement.not_resource`, `statement.resource`, `statement.condition.<value>.<value>.member_0`

| Member | Type |
|--------|------|
| `member_0` | `String` |
| `member_1` | List\<`String`\> (items: 1..unbounded, unordered) |

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html) | ` <version_block> = "Version" : ("2008-10-17" \| "2012-10-17") `<br>` <effect_block> = "Effect" : ("Allow" \| "Deny") ` | 2026-11-07 |
| `/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_statement.html) | ` The Statement element can contain a single statement or an array of individual statements. ` | 2026-11-07 |
| `/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_condition.html) | ` You can create conditions that compare strings, dates, numbers, and more using typical Boolean comparisons such as equals, greater than, and less than. ` | 2026-11-07 |

