# empty-values

Forbid empty values in mappings and sequences.

## Options

- `forbid-in-block-mappings`: (default: `true`) Forbid empty values in block mappings.
- `forbid-in-flow-mappings`: (default: `true`) Forbid empty values in flow mappings.
- `forbid-in-block-sequences`: (default: `true`) Forbid empty values in block sequences.

## Examples

### Correct

```yaml
key: value
- item
```

### Incorrect

```yaml
key:
- 
```
