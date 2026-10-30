---
title: "awscc.scheduler.ScheduleGroup"
---


CloudFormation Type: `AWS::Scheduler::ScheduleGroup`

Definition of AWS::Scheduler::ScheduleGroup Resource Type

## Argument Reference

### `name`

- **Type:** String(pattern, len: 1..=64)
- **Required:** No
- **Create-only:** Yes

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

The list of tags to associate with the schedule group.

## Enum Values

### state (ScheduleGroupState)

| Value | DSL Identifier |
|-------|----------------|
| `ACTIVE` | `aws.scheduler.ScheduleGroup.ScheduleGroupState.active` |
| `DELETING` | `aws.scheduler.ScheduleGroup.ScheduleGroupState.deleting` |

Shorthand formats: `active` or `ScheduleGroupState.active`

## Attribute Reference

### `arn`

- **Type:** Arn

The Amazon Resource Name (ARN) of the schedule group.

### `creation_date`

- **Type:** String(date-time)

The time at which the schedule group was created.

### `last_modification_date`

- **Type:** String(date-time)

The time at which the schedule group was last modified.

### `state`

- **Type:** [Enum (ScheduleGroupState)](#state-schedulegroupstate)

