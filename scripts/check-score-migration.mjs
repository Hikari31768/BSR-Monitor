// node scripts/check-score-migration.mjs <existing database>
// The source is read-only; migration runs only on a consistent scratch backup.
import { DatabaseSync, backup } from 'node:sqlite';
import { mkdir, readFile, stat, writeFile } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';

if (!process.argv[2]) throw new Error('Provide an existing database path');
await mkdir('.build-cache', { recursive: true });
const scratch = path.resolve(`.build-cache/migration-${process.pid}.db`);
const source = new DatabaseSync(path.resolve(process.argv[2]), { readOnly: true });
await backup(source, scratch);
source.close();
const db = new DatabaseSync(scratch);
const beforeBytes = (await stat(scratch)).size;
const before = db.prepare(`
    SELECT playerId, scoreId, LOWER(hash) AS hash, mode, difficulty, accuracy, epochTime
    FROM (
      SELECT *, ROW_NUMBER() OVER (
        PARTITION BY playerId, LOWER(hash), mode, difficulty
        ORDER BY epochTime DESC, accuracy DESC
      ) AS priority FROM PlayerScore
    ) WHERE priority = 1
    ORDER BY playerId, hash, mode, difficulty
`).all();
const metadataBefore = db.prepare(`
    SELECT key, LOWER(hash) AS hash, isCurrent, mode, difficulty,
           starsRating, starsFetchedAt, passRating, accRating, techRating
    FROM MapDiff ORDER BY key, hash, mode, difficulty
`).all();
db.exec('PRAGMA foreign_keys = ON; BEGIN;');
try {
    db.exec(await readFile('src-tauri/src/database/normalize_scores.sql', 'utf8'));
    db.exec('COMMIT;');
} catch (error) {
    db.exec('ROLLBACK;');
    throw error;
}
const after = db.prepare(`
    SELECT s.playerId, s.scoreId, d.hash, d.mode, d.difficulty, s.accuracy, s.epochTime
    FROM PlayerScore s JOIN MapDiff d ON s.mapDiffId = d.id
    ORDER BY s.playerId, d.hash, d.mode, d.difficulty
`).all();
assert.deepEqual(after, before, 'Score identity, accuracy or timestamp changed');
const metadataAfter = db.prepare(`
    SELECT key, hash, isCurrent, mode, difficulty,
           starsRating, starsFetchedAt, passRating, accRating, techRating
    FROM MapDiff WHERE metadataKnown = 1 ORDER BY key, hash, mode, difficulty
`).all();
assert.deepEqual(metadataAfter, metadataBefore, 'Existing map metadata changed');
assert.deepEqual(db.prepare('PRAGMA foreign_key_check').all(), []);
assert.equal(db.prepare('PRAGMA integrity_check').get().integrity_check, 'ok');
assert.deepEqual(db.prepare("SELECT name FROM pragma_table_info('PlayerScore')").all().map(r => r.name),
    ['playerId', 'mapDiffId', 'scoreId', 'accuracy', 'epochTime']);
db.exec('VACUUM;');
const results = {
    scoresPreserved: after.length,
    metadataPreserved: metadataAfter.length,
    beforeBytes,
    afterBytes: (await stat(scratch)).size,
    scratch,
};
db.close();
await writeFile('.build-cache/migration-results.json', JSON.stringify(results, null, 2));
console.log(JSON.stringify(results, null, 2));
