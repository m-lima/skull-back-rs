CREATE TABLE IF NOT EXISTS "last_modified" (
  "table"  INTEGER  NOT NULL PRIMARY KEY,
  "millis" INTEGER
);

-- Skulls [0]
CREATE TRIGGER IF NOT EXISTS "skulls_last_modified_insert" AFTER INSERT ON skulls
BEGIN
  INSERT OR REPLACE INTO "last_modified" (
    "table",
    "millis"
  ) VALUES (
    0,
    strftime("%s", "now") || substr(strftime("%f", "now"),4)
  );
END;
CREATE TRIGGER IF NOT EXISTS "skulls_last_modified_update" AFTER UPDATE ON skulls
BEGIN
  INSERT OR REPLACE INTO "last_modified" (
    "table",
    "millis"
  ) VALUES (
    0,
    strftime("%s", "now") || substr(strftime("%f", "now"),4)
  );
END;
CREATE TRIGGER IF NOT EXISTS "skulls_last_modified_delete" AFTER DELETE ON skulls
BEGIN
  INSERT OR REPLACE INTO "last_modified" (
    "table",
    "millis"
  ) VALUES (
    0,
    strftime("%s", "now") || substr(strftime("%f", "now"),4)
  );
END;

-- Quicks [1]
CREATE TRIGGER IF NOT EXISTS "quicks_last_modified_insert" AFTER INSERT ON quicks
BEGIN
  INSERT OR REPLACE INTO "last_modified" (
    "table",
    "millis"
  ) VALUES (
    1,
    strftime("%s", "now") || substr(strftime("%f", "now"),4)
  );
END;
CREATE TRIGGER IF NOT EXISTS "quicks_last_modified_update" AFTER UPDATE ON quicks
BEGIN
  INSERT OR REPLACE INTO "last_modified" (
    "table",
    "millis"
  ) VALUES (
    1,
    strftime("%s", "now") || substr(strftime("%f", "now"),4)
  );
END;
CREATE TRIGGER IF NOT EXISTS "quicks_last_modified_delete" AFTER DELETE ON quicks
BEGIN
  INSERT OR REPLACE INTO "last_modified" (
    "table",
    "millis"
  ) VALUES (
    1,
    strftime("%s", "now") || substr(strftime("%f", "now"),4)
  );
END;

-- Occurrences [2]
CREATE TRIGGER IF NOT EXISTS "occurrences_last_modified_insert" AFTER INSERT ON occurrences
BEGIN
  INSERT OR REPLACE INTO "last_modified" (
    "table",
    "millis"
  ) VALUES (
    2,
    strftime("%s", "now") || substr(strftime("%f", "now"),4)
  );
END;
CREATE TRIGGER IF NOT EXISTS "occurrences_last_modified_update" AFTER UPDATE ON occurrences
BEGIN
  INSERT OR REPLACE INTO "last_modified" (
    "table",
    "millis"
  ) VALUES (
    2,
    strftime("%s", "now") || substr(strftime("%f", "now"),4)
  );
END;
CREATE TRIGGER IF NOT EXISTS "occurrences_last_modified_delete" AFTER DELETE ON occurrences
BEGIN
  INSERT OR REPLACE INTO "last_modified" (
    "table",
    "millis"
  ) VALUES (
    2,
    strftime("%s", "now") || substr(strftime("%f", "now"),4)
  );
END;
