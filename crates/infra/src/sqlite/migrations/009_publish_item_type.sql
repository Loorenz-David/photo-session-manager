ALTER TABLE publish_items ADD COLUMN frozen_item_type TEXT NOT NULL DEFAULT '';
ALTER TABLE item_links ADD COLUMN item_type_base TEXT;
