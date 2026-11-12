---
title: "awscc.wafv2.WebAclAssociation"
---


CloudFormation Type: `AWS::WAFv2::WebACLAssociation`

Associates WebACL to Application Load Balancer, CloudFront or API Gateway.

## Example

```crn
let vpc = awscc.ec2.Vpc {
  cidr_block           = '10.0.0.0/16'
  enable_dns_support   = true
  enable_dns_hostnames = true

  tags = {
    Environment = 'example'
  }
}

let subnet_a = awscc.ec2.Subnet {
  vpc_id            = vpc.vpc_id
  cidr_block        = '10.0.1.0/24'
  availability_zone = 'ap-northeast-1a'

  tags = {
    Environment = 'example'
  }
}

let subnet_c = awscc.ec2.Subnet {
  vpc_id            = vpc.vpc_id
  cidr_block        = '10.0.2.0/24'
  availability_zone = 'ap-northeast-1c'

  tags = {
    Environment = 'example'
  }
}

let load_balancer_security_group = awscc.ec2.SecurityGroup {
  vpc_id            = vpc.vpc_id
  group_description = 'Allow HTTP traffic to the example load balancer'

  security_group_ingress {
    ip_protocol = aws.ec2.SecurityGroup.Ingress.IpProtocol.tcp
    from_port   = 80
    to_port     = 80
    cidr_ip     = '10.0.0.0/16'
  }

  tags = {
    Environment = 'example'
  }
}

let web_acl = awscc.wafv2.WebAcl {
  name        = 'carina-example-associated-web-acl'
  description = 'Regional protection for the example load balancer'
  scope       = aws.wafv2.WebAcl.Scope.regional

  default_action = {
    allow = {}
  }

  visibility_config = {
    cloud_watch_metrics_enabled = true
    metric_name                 = 'carina-example-associated-web-acl'
    sampled_requests_enabled    = true
  }

  tags = {
    Environment = 'example'
  }
}

let load_balancer = awscc.elasticloadbalancingv2.LoadBalancer {
  name            = 'carina-example-alb'
  type            = aws.elasticloadbalancingv2.LoadBalancer.Type.application
  scheme          = aws.elasticloadbalancingv2.LoadBalancer.Scheme.internal
  subnets         = [subnet_a.subnet_id, subnet_c.subnet_id]
  security_groups = [load_balancer_security_group.group_id]

  tags = {
    Environment = 'example'
  }
}

awscc.wafv2.WebAclAssociation {
  resource_arn = load_balancer.load_balancer_arn
  web_acl_arn  = web_acl.arn
}
```

## Argument Reference

### `resource_arn`

- **Type:** `aws.Arn (identity constraints: pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):[^:]+:.*$/ (Full), member constraints: len: 20..=2048, custom validator)`
- **Required:** Yes
- **Create-only:** Yes

### `web_acl_arn`

- **Type:** `aws.Arn (identity constraints: pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):[^:]+:.*$/ (Full), member constraints: len: 20..=2048, custom validator)`
- **Required:** Yes
- **Create-only:** Yes

