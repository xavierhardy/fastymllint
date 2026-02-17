# document-end

Enforce the presence of a document end marker (`...`).

## Options

- `present`: (default: `false` for yamllint, but `true` if rule is enabled in megalinter) When `true`, requires the document to end with `...`.

## Examples

### Correct (with `present: true`)

```yaml
key: value
...
```

### Incorrect (with `present: true`)

```yaml
key: value
```
