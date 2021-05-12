# Tui

Terminal UI is explained here.

## AppView

```ascii
    +--Folder Name--+--Selected Item---------------+
    | Folder 1/     |                              |
    | Folder 2/     |  <content of selected item>  |
    | Folder 3/     |                              |
    | Note 1        |                              |
    | Note 2        |                              |
    | Note 3        |                              |
    |               |                              |
    |               |                              |
    |               |                              |
    |               |                              |
    |               |                              |
    +---------------+------------------------------+
```

## Content

| Item   | content              | on enter               |
| ------ | -------------------- | ---------------------- |
| Folder | sub Folder + notes   | make it current folder |
| Note   | body display as text | run edit command       |
