CREATE TABLE IF NOT EXISTS shooting_templates (
  id INTEGER PRIMARY KEY,
  item_type TEXT NOT NULL,
  type_key TEXT NOT NULL UNIQUE,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS template_containers (
  id INTEGER PRIMARY KEY,
  template_id INTEGER NOT NULL REFERENCES shooting_templates(id),
  final_no INTEGER NOT NULL,
  shoot_no INTEGER NOT NULL,
  example_file TEXT,
  split INTEGER NOT NULL DEFAULT 0,
  shape TEXT,
  zoom INTEGER NOT NULL DEFAULT 0
);
