

# YamlDescender (yaml_descender.rs)
(and perhaps soon, json descender)

A rust object that descends through yaml trees based on a 'path' syntax

Code here is used in both [register_tool](https://github.com/AndrewOfC/register_tool) and [ucompleter](https://github.com/AndrewOfC/ucompleter).  In an effort to avoid duplication
and mantain consistency this repo was created.

# Concepts
## Paths

| Yaml Type | Separator                                                                               |
|-----------|-----------------------------------------------------------------------------------------|
| Hash      | '.' for dereferencing fields in a hash                                                  |
| Array     | [i] for bash orientations(default) <br>@i for zsh orientations<br>(where i is an index) |

### Example

```yaml

root:
  record:
    field: value

array:
  - record: value1
  - record: value2
```
root.record.field = value

array[0].record = value1

array[1].record = value2