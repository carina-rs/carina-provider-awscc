//! web_acl_association schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::WAFv2::WebACLAssociation
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, ResourceSchema};

/// Returns the schema config for wafv2_web_acl_association (AWS::WAFv2::WebACLAssociation)
pub fn wafv2_web_acl_association_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::WAFv2::WebACLAssociation",
        resource_type_name: "wafv2.WebAclAssociation",
        primary_identifier: &[
            crate::schemas::config::PrimaryIdentifierAttribute {
                provider_name: "ResourceArn",
                dsl_name: "resource_arn",
            },
            crate::schemas::config::PrimaryIdentifierAttribute {
                provider_name: "WebACLArn",
                dsl_name: "web_acl_arn",
            },
        ],
        has_tags: false,
        schema: ResourceSchema::new("wafv2.WebAclAssociation")
            .with_description(
                "Associates WebACL to Application Load Balancer, CloudFront or API Gateway.",
            )
            .attribute(
                AttributeSchema::new("resource_arn", carina_aws_types::arn())
                    .required()
                    .create_only()
                    .with_provider_name("ResourceArn"),
            )
            .attribute(
                AttributeSchema::new("web_acl_arn", carina_aws_types::arn())
                    .required()
                    .create_only()
                    .with_provider_name("WebACLArn"),
            ),
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("wafv2.WebAclAssociation", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "wafv2:AssociateWebACL",
            "wafv2:GetWebACLForResource",
            "wafv2:GetWebACL",
            "wafv2:DisassociateWebACL",
            "wafv2:PutPermissionPolicy",
            "wafv2:GetPermissionPolicy",
            "elasticloadbalancing:SetWebACL",
            "apigateway:SetWebACL",
            "appsync:SetWebACL",
            "cognito-idp:AssociateWebACL",
            "cognito-idp:DisassociateWebACL",
            "cognito-idp:GetWebACLForResource",
            "apprunner:AssociateWebAcl",
            "apprunner:DisassociateWebAcl",
            "apprunner:DescribeWebAclForService",
            "ec2:AssociateVerifiedAccessInstanceWebAcl",
            "ec2:DisassociateVerifiedAccessInstanceWebAcl",
            "ec2:DescribeVerifiedAccessInstanceWebAclAssociations",
            "ec2:GetVerifiedAccessInstanceWebAcl",
            "amplify:AssociateWebACL",
            "amplify:GetWebACLForResource",
            "elasticloadbalancing:CreateWebACLAssociation",
            "elasticloadbalancing:DeleteWebACLAssociation",
            "elasticloadbalancing:GetLoadBalancerWebACL",
            "appsync:AssociateWebACL",
            "appsync:DisassociateWebACL",
            "appsync:GetWebACLForResource",
            "bedrock-agentcore:GatewayAssociateWebACL",
            "bedrock-agentcore:GatewayDisassociateWebACL",
            "bedrock-agentcore:GatewayGetWebACLForResource",
        ],
        carina_core::effect::PlanOp::Read => &[
            "wafv2:AssociateWebACL",
            "wafv2:GetWebACLForResource",
            "wafv2:GetWebACL",
            "wafv2:DisassociateWebACL",
            "elasticloadbalancing:SetWebACL",
            "apigateway:SetWebACL",
            "appsync:SetWebACL",
            "cognito-idp:AssociateWebACL",
            "cognito-idp:DisassociateWebACL",
            "cognito-idp:GetWebACLForResource",
            "apprunner:AssociateWebAcl",
            "apprunner:DisassociateWebAcl",
            "apprunner:DescribeWebAclForService",
            "ec2:AssociateVerifiedAccessInstanceWebAcl",
            "ec2:DisassociateVerifiedAccessInstanceWebAcl",
            "ec2:DescribeVerifiedAccessInstanceWebAclAssociations",
            "ec2:GetVerifiedAccessInstanceWebAcl",
            "amplify:GetWebACLForResource",
            "elasticloadbalancing:CreateWebACLAssociation",
            "elasticloadbalancing:DeleteWebACLAssociation",
            "elasticloadbalancing:GetLoadBalancerWebACL",
            "appsync:AssociateWebACL",
            "appsync:DisassociateWebACL",
            "appsync:GetWebACLForResource",
            "bedrock-agentcore:GatewayAssociateWebACL",
            "bedrock-agentcore:GatewayDisassociateWebACL",
            "bedrock-agentcore:GatewayGetWebACLForResource",
        ],
        carina_core::effect::PlanOp::Update => &[
            "wafv2:AssociateWebACL",
            "wafv2:GetWebACLForResource",
            "wafv2:GetWebACL",
            "wafv2:DisassociateWebACL",
            "elasticloadbalancing:SetWebACL",
            "apigateway:SetWebACL",
            "appsync:SetWebACL",
            "cognito-idp:AssociateWebACL",
            "cognito-idp:DisassociateWebACL",
            "cognito-idp:GetWebACLForResource",
            "apprunner:AssociateWebAcl",
            "apprunner:DisassociateWebAcl",
            "apprunner:DescribeWebAclForService",
            "ec2:AssociateVerifiedAccessInstanceWebAcl",
            "ec2:DisassociateVerifiedAccessInstanceWebAcl",
            "ec2:DescribeVerifiedAccessInstanceWebAclAssociations",
            "ec2:GetVerifiedAccessInstanceWebAcl",
            "elasticloadbalancing:CreateWebACLAssociation",
            "elasticloadbalancing:DeleteWebACLAssociation",
            "elasticloadbalancing:GetLoadBalancerWebACL",
            "appsync:AssociateWebACL",
            "appsync:DisassociateWebACL",
            "appsync:GetWebACLForResource",
            "bedrock-agentcore:GatewayAssociateWebACL",
            "bedrock-agentcore:GatewayDisassociateWebACL",
            "bedrock-agentcore:GatewayGetWebACLForResource",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "wafv2:AssociateWebACL",
            "wafv2:GetWebACLForResource",
            "wafv2:GetWebACL",
            "wafv2:DisassociateWebACL",
            "wafv2:PutPermissionPolicy",
            "elasticloadbalancing:SetWebACL",
            "apigateway:SetWebACL",
            "appsync:SetWebACL",
            "cognito-idp:AssociateWebACL",
            "cognito-idp:DisassociateWebACL",
            "cognito-idp:GetWebACLForResource",
            "apprunner:AssociateWebAcl",
            "apprunner:DisassociateWebAcl",
            "apprunner:DescribeWebAclForService",
            "ec2:AssociateVerifiedAccessInstanceWebAcl",
            "ec2:DisassociateVerifiedAccessInstanceWebAcl",
            "ec2:DescribeVerifiedAccessInstanceWebAclAssociations",
            "ec2:GetVerifiedAccessInstanceWebAcl",
            "amplify:DisassociateWebACL",
            "amplify:GetWebACLForResource",
            "elasticloadbalancing:CreateWebACLAssociation",
            "elasticloadbalancing:DeleteWebACLAssociation",
            "elasticloadbalancing:GetLoadBalancerWebACL",
            "appsync:AssociateWebACL",
            "appsync:DisassociateWebACL",
            "appsync:GetWebACLForResource",
            "bedrock-agentcore:GatewayAssociateWebACL",
            "bedrock-agentcore:GatewayDisassociateWebACL",
            "bedrock-agentcore:GatewayGetWebACLForResource",
        ],
    }
}
