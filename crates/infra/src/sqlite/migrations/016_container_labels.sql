-- Container name and description (owner 2026-11-16): an optional label for the shot angle and a note on how to take it.
ALTER TABLE template_containers ADD COLUMN name TEXT;
ALTER TABLE template_containers ADD COLUMN description TEXT;
