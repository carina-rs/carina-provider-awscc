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

## Deep-Cleanup Coverage

All resource types with acceptance fixtures are covered by a cleanup pass or a
validated parent-deletion exception. There are currently no known deep-cleanup
gaps. `shell-tests/deep_cleanup_fixture_coverage.sh` enforces this coverage and
is where sweep mappings and parent-deletion exceptions are maintained. When a
fixture type temporarily lacks cleanup, record it in that test's known-gaps list
with a tracking issue. Known-gap entries are self-retiring: once the expected
sweep command appears, the test fails until the entry is removed.
