# Bluenote Composition Language (BCL)

Bluenote Composition Language (BCL) is an IMAGINARY, domain-specific language designed for querying and manipulating data within the blueline. The language syntax is a blend of HTTP, JSON Query Language (JQL), XML Query Language (XQL), Cascade Style Sheet (CSS) Selector, Regular Expression (Regex), and SQL. It is designed for real hard-core black-and-white console jankies who want to play with waves of text data in the terminal like Matrix operators in the 21st century.

## Example

```shell
GET /_cat/shards
| FIXED                                  # Convert fixed-length to dataset
| SELECT .node, .index, .shard, .prirep  # Limit the scope
| SORT .node, .index, .prirep, .shard    # Sort dataset
| SAVE ~/shards.csv IN CSV           # Save dataset to a file in CSV format
```

```shell
GET @profile1:/employees/_search         # Fetch employee data in JSON
WITH BODY IN YAML                        # with passing query conditions
  query:
    bool:
      must:
        - match:
            age: 30
        - term:
            employ_status: "employed"
| SELECT hits.hits[] | ._source
| POST @profile2:/expo/register
  WITH BODY IN FORM
      name: .name
      email: .email
```

```shell
POST /_query
WITH BODY IN YAML
  query:
    bool:
      must:
        - match:
            title: "Blueline"
        - range:
            @timestamp:
              gte: "2023-01-01"
              lte: "${now/d}"
  size: 10
  sort:
    - @timestamp: desc
| JSON
```

## Syntax

## Statement

## Functions
