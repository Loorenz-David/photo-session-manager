-- Price labels (book 09C §4; master_plan_labels.md §4 Storage): whether the saved Pricing key may read
-- labels (unknown | allowed | denied). A fact about the key, not label state: nothing else about
-- labels is ever stored (M-L1).
ALTER TABLE conversation_sync ADD COLUMN labels_access TEXT NOT NULL DEFAULT 'unknown';
