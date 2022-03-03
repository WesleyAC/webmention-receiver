BEGIN;
PRAGMA user_version = 1;

CREATE TABLE webmentions (
	id           BLOB PRIMARY KEY, -- UUID4
	domain       TEXT NOT NULL,
	source       TEXT NOT NULL,
	target       TEXT NOT NULL,
	date_added   INT NOT NULL,     -- epoch time in milliseconds
	date_updated INT NOT NULL,     -- epoch time in milliseconds
	UNIQUE(domain, source, target)
) STRICT;

COMMIT;
