# Acceptance Tests

## Known Coverage Gaps

The following generated resource types intentionally do not have acceptance
fixtures yet:

- `organizations.Organization`: the pooled `carina-test-00X` accounts are
  already members of the shared organization, and organization management is not
  safe to exercise from individual pool accounts.
- `organizations.Account`: creating accounts would add new members to the shared
  organization, and account closure lingers for 90 days before final deletion.
- `sso.Instance`: IAM Identity Center instances live in the organization
  management account, are limited to one instance per organization, and are not
  creatable from the pool accounts.
- `sso.PermissionSet`: permission sets require the organization IAM Identity
  Center instance and identity store that live in the management account.
- `sso.Assignment`: assignments require the organization IAM Identity Center
  instance and identity store that live in the management account.
- `identitystore.Group`: identity store resources live with the organization IAM
  Identity Center instance in the management account, not in pool accounts.
- `identitystore.GroupMembership`: group memberships depend on the management
  account identity store and are not creatable in pool accounts.

## Known Deep-Cleanup Gaps

The following resource types have acceptance fixtures but no corresponding pass
in `deep_cleanup_account()`. Until fixed, they can leave orphaned resources in
the pooled test accounts. `shell-tests/deep_cleanup_fixture_coverage.sh`
enforces this list and is where sweep mappings, parent-deletion exceptions, and
known gaps are maintained.

- `ec2.FlowLog`: tracked by #403.
- `ec2.EgressOnlyInternetGateway`: tracked by #403.
- `ec2.Ipam`: tracked by #406.
- `ec2.IpamPool`: tracked by #406.
- `kms.Key`: tracked by #407.
- `cloudfront.Distribution`: tracked by #408.
- `cloudfront.OriginAccessControl`: tracked by #408.
