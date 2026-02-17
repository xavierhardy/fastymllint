# braces

Enforce consistent spacing inside braces.

## Options

- `min-spaces-inside`: (default: `0`) Minimum number of spaces inside braces.
- `max-spaces-inside`: (default: `0`) Maximum number of spaces inside braces.
- `min-spaces-inside-empty`: (default: `null`) Minimum number of spaces inside empty braces. If not set, `min-spaces-inside` is used.
- `max-spaces-inside-empty`: (default: `null`) Maximum number of spaces inside empty braces. If not set, `max-spaces-inside` is used.
- `forbid`: (default: `false`) Forbid flow mappings (e.g., `{key: value}`).
- `forbid-non-empty`: (default: `false`) Forbid non-empty flow mappings.

## Examples

### Correct

```yaml
# With default settings (0 spaces)
mapping: {key: value}
empty: {}
```

### Correct (with 1 space)

```yaml
# With min-spaces-inside: 1, max-spaces-inside: 1
mapping: { key: value }
```

### Incorrect

```yaml
# With min-spaces-inside: 1
mapping: {key: value}
```
