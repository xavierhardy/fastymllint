# brackets

Enforce consistent spacing inside brackets.

## Options

- `min-spaces-inside`: (default: `0`) Minimum number of spaces inside brackets.
- `max-spaces-inside`: (default: `0`) Maximum number of spaces inside brackets.
- `min-spaces-inside-empty`: (default: `null`) Minimum number of spaces inside empty brackets. If not set, `min-spaces-inside` is used.
- `max-spaces-inside-empty`: (default: `null`) Maximum number of spaces inside empty brackets. If not set, `max-spaces-inside` is used.
- `forbid`: (default: `false`) Forbid flow sequences (e.g., `[item1, item2]`).
- `forbid-non-empty`: (default: `false`) Forbid non-empty flow sequences.

## Examples

### Correct

```yaml
# With default settings (0 spaces)
sequence: [item1, item2]
empty: []
```

### Correct (with 1 space)

```yaml
# With min-spaces-inside: 1, max-spaces-inside: 1
sequence: [ item1, item2 ]
```

### Incorrect

```yaml
# With min-spaces-inside: 1
sequence: [item1, item2]
```
