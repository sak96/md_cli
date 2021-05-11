# Database

Internal models used are explained here.

## Folder

Also called book.

| name                      | value             | ref       |
|---------------------------|-------------------|-----------|
| id                        | uuid of item      |           |
| title                     | name of item      |           |
| parent_id                 | uuid of parent_id | Folder.id |
| uniq [ parent_id + title] | same item         |           |

## Note

| name                      | value             | ref       |
|---------------------------|-------------------|-----------|
| id                        | uuid of item      |           |
| parent_id                 | uuid of parent_id | Folder.id |
| title                     | name of item      |           |
| body                      | content of item   |           |
| uniq [ parent_id + title] | same item         |           |

