# empty-lines

Checks for too many consecutive empty lines.

## Options

- `max`: (default: `2`) Maximum number of consecutive empty lines.
- `max-start`: (default: `0`) Maximum number of empty lines at the beginning of the file.
- `max-end`: (default: `0`) Maximum number of empty lines at the end of the file.

## Examples

### Correct

```yaml
key: value

other: value
```

### Incorrect

```yaml


key: value



other: value

```
