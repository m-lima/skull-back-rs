CREATE TABLE last_modified (
  table  INTEGER NOT NULL PRIMARY KEY,
  millis INTEGER NOT NULL
);

INSERT INTO last_modified (
  table,
  millis
) VALUES (
  0,
  strftime("%s", "now") || substr(strftime("%f", "now"), 4)
), (
  1,
  strftime("%s", "now") || substr(strftime("%f", "now"), 4)
), (
  2,
  strftime("%s", "now") || substr(strftime("%f", "now"), 4)
);

-- Skulls [0]
CREATE TRIGGER skulls_last_modified_insert AFTER INSERT ON skulls
BEGIN
  UPDATE last_modified
  SET
    millis = strftime("%s", "now") || substr(strftime("%f", "now"), 4)
  WHERE table = 0;
END;
CREATE TRIGGER skulls_last_modified_update AFTER UPDATE ON skulls
BEGIN
  UPDATE last_modified
  SET
    millis = strftime("%s", "now") || substr(strftime("%f", "now"), 4)
  WHERE table = 0;
END;
CREATE TRIGGER skulls_last_modified_delete AFTER DELETE ON skulls
BEGIN
  UPDATE last_modified
  SET
    millis = strftime("%s", "now") || substr(strftime("%f", "now"), 4)
  WHERE table = 0;
END;

-- Quicks [1]
CREATE TRIGGER quicks_last_modified_insert AFTER INSERT ON quicks
BEGIN
  UPDATE last_modified
  SET
    millis = strftime("%s", "now") || substr(strftime("%f", "now"), 4)
  WHERE table = 1;
END;
CREATE TRIGGER quicks_last_modified_update AFTER UPDATE ON quicks
BEGIN
  UPDATE last_modified
  SET
    millis = strftime("%s", "now") || substr(strftime("%f", "now"), 4)
  WHERE table = 1;
END;
CREATE TRIGGER quicks_last_modified_delete AFTER DELETE ON quicks
BEGIN
  UPDATE last_modified
  SET
    millis = strftime("%s", "now") || substr(strftime("%f", "now"), 4)
  WHERE table = 1;
END;

-- Occurrences [2]
CREATE TRIGGER occurrences_last_modified_insert AFTER INSERT ON occurrences
BEGIN
  UPDATE last_modified
  SET
    millis = strftime("%s", "now") || substr(strftime("%f", "now"), 4)
  WHERE table = 2;
END;
CREATE TRIGGER occurrences_last_modified_update AFTER UPDATE ON occurrences
BEGIN
  UPDATE last_modified
  SET
    millis = strftime("%s", "now") || substr(strftime("%f", "now"), 4)
  WHERE table = 2;
END;
CREATE TRIGGER occurrences_last_modified_delete AFTER DELETE ON occurrences
BEGIN
  UPDATE last_modified
  SET
    millis = strftime("%s", "now") || substr(strftime("%f", "now"), 4)
  WHERE table = 2;
END;
