-- Skulls
CREATE TABLE skulls (
  "id"         INTEGER NOT NULL PRIMARY KEY,
  "name"       TEXT    NOT NULL UNIQUE CHECK(LENGTH(TRIM(name)) > 0),
  "color"      INTEGER NOT NULL UNIQUE,
  "icon"       TEXT    NOT NULL UNIQUE CHECK(LENGTH(TRIM(icon)) > 0),
  "price"      REAL    NOT NULL        CHECK(price >= 0),
  "limit"      REAL
);

-- Quicks
CREATE TABLE quicks (
  "id"     INTEGER NOT NULL PRIMARY KEY,
  "skull"  INTEGER NOT NULL,
  "amount" REAL    NOT NULL              CHECK(amount >= 0),

  FOREIGN KEY(skull) REFERENCES skulls(id) ON DELETE CASCADE,

  UNIQUE(skull, amount)
);

-- Occurrences
CREATE TABLE occurrences (
  "id"     INTEGER NOT NULL PRIMARY KEY,
  "skull"  INTEGER NOT NULL,
  "amount" REAL    NOT NULL              CHECK(amount >= 0),
  "millis" INTEGER NOT NULL,

  FOREIGN KEY(skull) REFERENCES skulls(id) ON DELETE RESTRICT
);
