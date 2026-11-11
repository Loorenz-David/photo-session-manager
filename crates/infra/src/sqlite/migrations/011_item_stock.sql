-- P65 (changelog 45): the stock a new item is created with on an integration (chosen in the item
-- form), and the stock frozen with a confirmed publish item. New tables only (`IF NOT EXISTS`: a
-- test that rewinds `user_version` re-runs this).
CREATE TABLE IF NOT EXISTS item_stock (
    item_id INTEGER NOT NULL REFERENCES items(id),
    integration TEXT NOT NULL,
    location_id TEXT NOT NULL,
    location_name TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    PRIMARY KEY (item_id, integration)
);
CREATE TABLE IF NOT EXISTS publish_item_stock (
    publish_item_id INTEGER PRIMARY KEY REFERENCES publish_items(id),
    location_id TEXT NOT NULL,
    location_name TEXT NOT NULL,
    quantity INTEGER NOT NULL
);
