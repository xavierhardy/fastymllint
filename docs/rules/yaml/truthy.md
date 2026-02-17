# truthy

Enforce specific truthy values.

## Options

- `allowed-values`: (default: `["true", "false"]`) List of allowed truthy values.
- `check-keys`: (default: `true`) Whether to also check keys for truthy values.

## Examples

### Correct

```yaml
boolean: true
status: false
```

### Incorrect

```yaml
boolean: yes
status: off
```
