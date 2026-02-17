# indentation

Checks for consistent indentation.

## Options

- `spaces`: (default: `consistent`) Number of spaces for indentation. If `consistent`, the indentation is detected from the first indentation level.
- `indent-sequences`: (default: `true`) Whether to indent block sequences.
- `check-multi-line-strings`: (default: `false`) Whether to check indentation of multi-line strings.

## Examples

### Correct

```yaml
key:
  - item1
  - item2
```

### Incorrect

```yaml
key:
  - item1
   - item2
```
