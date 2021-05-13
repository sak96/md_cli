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
    +----------------------------------------------+
    |:                                             |
    +----------------------------------------------+
```

## PopUpView

```ascii
    +----------------------------------------------+
    |                                              |
    |                                              |
    |          +------------------------+          |
    |          | <Message or Question>  |          |
    |          |                        |          |
    |          | <Optional Input>       |          |
    |          |                        |          |
    |          |            [Ok][Cancel]|          |
    |          +------------------------+          |
    |                                              |
    |                                              |
    +----------------------------------------------+
```

## Content

| Item   | content              | on enter               |
| ------ | -------------------- | ---------------------- |
| Folder | sub Folder + notes   | make it current folder |
| Note   | body display as text | run edit command       |
| None   | help message         | n/a                    |

## Key Maps

| keys               | action              |
| ------------------ | ------------------- |
| j,k,up,down        | movement in list    |
| l, enter, right    | select item in list |
| h, backspace, left | go back             |
| a                  | add folder/note     |
| d                  | delete folder/note  |
| m                  | move note           |
| c                  | copy note           |
| :                  | interpreter         |
