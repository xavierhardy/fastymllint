# commas

Enforce consistent spacing around commas.

## Options

- `max-spaces-before`: (default: `0`) Maximum number of spaces before commas.
- `min-spaces-after`: (default: `1`) Minimum number of spaces after commas.
- `max-spaces-after`: (default: `1`) Maximum number of spaces after commas.

## Examples

### Correct

```yaml
flow: [item1, item2, item3]
```

### Incorrect (too many spaces before)

```yaml
flow: [item1 , item2]
```

### Incorrect (too few spaces after)

```yaml
flow: [item1,item2]
```

### Incorrect (too many spaces after)

```yaml
flow: [item1,  item2]
```
