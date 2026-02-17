# key-ordering

Checks that keys in mappings are in alphabetical order.

## Options

- `ignored-keys`: (default: `[]`) A list of keys to ignore when checking for alphabetical order.

## Examples

### Correct

```yaml
a: 1
b: 2
c: 3
```

### Incorrect

```yaml
c: 3
a: 1
b: 2
```
