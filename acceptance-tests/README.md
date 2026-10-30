# Acceptance Tests

## Persistent run registry and cleanup

Stateful acceptance tests register each fixture under
`${CARINA_ACCEPTANCE_HOME:-$HOME/.cache/carina-acceptance}/carina-provider-awscc/runs/<run-id>/`.
Each entry keeps its injected `main.crn`, local Carina state, and metadata for
the fixture, AWS account, and creating host. Each run also records its creating
host, PID, and process start time so PID reuse is not mistaken for a live run.
Temporary log directories may be removed, but a registry entry is removed only
after `carina state show --json` reports absent state or an empty resource list.

Use `./acceptance-tests/run-tests.sh cleanup` to destroy every registered
entry. `--accounts START-END` restricts cleanup to those accounts, and any
remaining arguments filter the stored fixture path by substring. For example:

```console
./acceptance-tests/run-tests.sh cleanup --accounts 3-3 ec2_vpc
```

Cleanup uses the account recorded in each entry. Unusable entries remain
registered and produce a non-zero exit: without fixture filters they are
validation failures (exit status 1), while unusable metadata that cannot be
matched against a fixture filter is skipped (exit status 3). Failed or
incomplete destroys also produce exit status 1 and remain registered, with
their paths printed for investigation. Runs whose creating process is still
alive on this host are reported as in progress and skipped. Runs created on
another host are also skipped and are never destroyed automatically. Destroy
or validation failures take precedence over skipped entries. If entries exist
but none match the requested accounts or fixture filters, cleanup reports `No
registry entries match`; an empty registry reports `No registered entries`.

Custom suites that are run directly also use this registry. Set `AWS_PROFILE`
to the account profile so the suite uses and records the same account for a
later cleanup.

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
