# float-values

Checks for forbidden float values.

## Options

- `forbid-inf`: (default: `false`) Forbid `inf` values.
- `forbid-nan`: (default: `false`) Forbid `nan` values.
- `forbid-scientific-notation`: (default: `false`) Forbid scientific notation.
- `require-numeral-before-decimal`: (default: `false`) Require a numeral before a decimal point.

## Examples

### Correct

```yaml
float: 1.0
```

### Incorrect

```yaml
# With forbid-inf: true
float: .inf

# With forbid-nan: true
float: .nan

# With forbid-scientific-notation: true
float: 1e-5

# With require-numeral-before-decimal: true
float: .5
```
