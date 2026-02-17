# line-length

Checks that all lines are shorter than a given maximum length.

## Options

- `max`: (default: `80`) The maximum line length.
- `allow-non-breakable-words`: (default: `true`) Whether to allow non-breakable words (like long URLs) to exceed the line length.
- `allow-non-breakable-inline-mappings`: (default: `false`) Whether to allow non-breakable inline mappings to exceed the line length.

## Examples

### Correct

```yaml
key: a short value
```

### Incorrect

```yaml
key: a very, very, very, very, very, very, very, very, very, very, very, very, very, very, very, very long value
```
