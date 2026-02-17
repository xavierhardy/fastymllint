# octal-values

Checks for forbidden octal values.

## Options

- `forbid-implicit-octal`: (default: `true`) Forbid implicit octal values (e.g., `0123`).
- `forbid-explicit-octal`: (default: `true`) Forbid explicit octal values (e.g., `0o123`).

## Examples

### Correct

```yaml
number: 123
```

### Incorrect

```yaml
# With forbid-implicit-octal: true
number: 0123

# With forbid-explicit-octal: true
number: 0o123
```
