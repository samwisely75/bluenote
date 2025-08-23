# Bluenote Composition Language (BCL)

Bluenote Composition Language (BCL) is an IMAGINARY, domain-specific language designed for querying and manipulating data within the blueline. The language syntax is a blend of HTTP, JSON Query Language (JQL), XML Query Language (XQL), Cascade Style Sheet (CSS) Selector, Regular Expression (Regex), and SQL. It is designed for real hard-core black-and-white console jankies who want to play with waves of text data in the terminal like Matrix operators in the 21st century.

## Example

```shell
GET /_cat/shards
| FIXED                                  # Convert fixed-length to dataset
| SELECT .node, .index, .shard, .prirep  # Limit the scope
| SORT .node, .index, .prirep, .shard    # Sort dataset
| TSV                                    # Output in TSV format
```

```shell
POST @profile1:/employees/_search        # Fetch employee data in JSON
WITH BODY AS JSON                        # with passing query defined in YAML
  query:                                 # and translated into JSON
    bool:
      must:
        - match:
            age: 30
        - term:
            employ_status: "employed"
  WITH HEADER Authorization = "ApiKey: ${API_KEY}" # Using special api key stored in the e/v
| FOR hit IN hits.hits[]                           # Looping
|| POST @profile2:/expo/register
   WITH BODY IN FORM
        fname: hit.first_name
        lname: hit.last_name
        email: hit.email
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
