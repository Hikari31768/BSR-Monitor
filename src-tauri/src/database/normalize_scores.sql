-- Run inside a transaction. MapDiff also holds score-only difficulties whose
-- song metadata has not been downloaded yet; these must not count as cached maps.
CREATE TABLE MapDiff_new (
    id INTEGER PRIMARY KEY,
    key TEXT,
    hash TEXT NOT NULL,
    isCurrent INTEGER NOT NULL DEFAULT 0,
    metadataKnown INTEGER NOT NULL DEFAULT 0,
    mode TEXT NOT NULL,
    difficulty TEXT NOT NULL,
    starsRating REAL,
    starsFetchedAt INTEGER,
    passRating REAL,
    accRating REAL,
    techRating REAL,
    UNIQUE(hash, mode, difficulty)
);

INSERT INTO MapDiff_new
    (key, hash, isCurrent, metadataKnown, mode, difficulty,
     starsRating, starsFetchedAt, passRating, accRating, techRating)
SELECT key, LOWER(hash), isCurrent, 1, mode, difficulty,
       starsRating, starsFetchedAt, passRating, accRating, techRating
FROM (
    SELECT *, ROW_NUMBER() OVER (
        PARTITION BY LOWER(hash), mode, difficulty
        ORDER BY starsFetchedAt DESC, isCurrent DESC, key
    ) AS priority FROM MapDiff
) WHERE priority = 1;

INSERT INTO MapDiff_new (key, hash, mode, difficulty)
SELECT MAX(mapKey), LOWER(hash), mode, difficulty FROM PlayerScore
WHERE 1 GROUP BY LOWER(hash), mode, difficulty
ON CONFLICT(hash, mode, difficulty) DO UPDATE SET
    key = COALESCE(MapDiff_new.key, excluded.key);

CREATE TABLE PlayerScore_new (
    playerId TEXT NOT NULL REFERENCES PlayerInfo(id),
    mapDiffId INTEGER NOT NULL REFERENCES MapDiff_new(id),
    scoreId INTEGER NOT NULL DEFAULT 0,
    accuracy REAL NOT NULL,
    epochTime INTEGER NOT NULL,
    PRIMARY KEY(playerId, mapDiffId)
) WITHOUT ROWID;

INSERT INTO PlayerScore_new (playerId, mapDiffId, scoreId, accuracy, epochTime)
SELECT s.playerId, d.id, s.scoreId, s.accuracy, s.epochTime
FROM PlayerScore s JOIN MapDiff_new d
  ON d.hash = LOWER(s.hash) AND d.mode = s.mode AND d.difficulty = s.difficulty
WHERE 1
ON CONFLICT(playerId, mapDiffId) DO UPDATE SET
    scoreId = excluded.scoreId, accuracy = excluded.accuracy, epochTime = excluded.epochTime
WHERE excluded.epochTime > PlayerScore_new.epochTime
   OR (excluded.epochTime = PlayerScore_new.epochTime
       AND excluded.accuracy > PlayerScore_new.accuracy);

DROP TABLE PlayerScore;
DROP TABLE MapDiff;
ALTER TABLE MapDiff_new RENAME TO MapDiff;
ALTER TABLE PlayerScore_new RENAME TO PlayerScore;
CREATE INDEX idx_mapdiff_key ON MapDiff(key);
CREATE INDEX idx_playerscore_diff ON PlayerScore(mapDiffId);
CREATE INDEX idx_playerscore_player_time ON PlayerScore(playerId, epochTime DESC);
PRAGMA user_version = 2;
