-- Skulls
CREATE TABLE IF NOT EXISTS "skulls" (
  "id"         INTEGER NOT NULL PRIMARY KEY,
  "name"       TEXT    NOT NULL UNIQUE,
  "color"      TEXT    NOT NULL UNIQUE,
  "icon"       TEXT    NOT NULL UNIQUE,
  "unit_price" REAL    NOT NULL,
  "limit"      REAL
);

-- Quicks
CREATE TABLE IF NOT EXISTS "quicks" (
  "id"     INTEGER NOT NULL PRIMARY KEY,
  "skull"  INTEGER NOT NULL,
  "amount" REAL    NOT NULL,

  FOREIGN KEY("skull") REFERENCES "skulls"("id") ON DELETE CASCADE,

  UNIQUE("skull", "amount")
);

-- Occurrences
CREATE TABLE IF NOT EXISTS "occurrences" (
  "id"     INTEGER NOT NULL PRIMARY KEY,
  "skull"  INTEGER NOT NULL,
  "amount" REAL    NOT NULL,
  "millis" INTEGER NOT NULL,

  FOREIGN KEY("skull") REFERENCES "skulls"("id") ON DELETE RESTRICT
);
