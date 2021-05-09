CREATE TABLE folders (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL DEFAULT "",
  parent_id TEXT NOT NULL DEFAULT "",
  UNIQUE(title, parent_id)
);
CREATE TABLE notes (
  id TEXT PRIMARY KEY,
  parent_id TEXT NOT NULL DEFAULT "",
  title TEXT NOT NULL DEFAULT "",
  body TEXT NOT NULL DEFAULT "",
  UNIQUE(title, parent_id)
);
