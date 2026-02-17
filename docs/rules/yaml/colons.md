# colons

Enforce consistent spacing around colons.

## Options

- `max-spaces-before`: (default: `0`) Maximum number of spaces before colons.
- `min-spaces-after`: (default: `1`) Minimum number of spaces after colons.
- `max-spaces-after`: (default: `1`) Maximum number of spaces after colons.

## Examples

### Correct

```yaml
key: value
```

### Incorrect (too many spaces before)

```yaml
key : value
```

### Incorrect (too few spaces after)

```yaml
key:value
```

### Incorrect (too many spaces after)

```yaml
key:  value
```
