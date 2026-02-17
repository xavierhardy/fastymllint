# key-duplicates

Checks for duplicate keys in mappings.

## Options

- `allowed-keys`: (default: `[]`) A list of keys that are allowed to be duplicated.

## Examples

### Correct

```yaml
key1: value1
key2: value2
```

### Incorrect

```yaml
key1: value1
key1: value2
```
