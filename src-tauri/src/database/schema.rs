use rusqlite::Connection;

fn add_column_if_not_exists(
    conn: &Connection,
    table: &str,
    column: &str,
    col_type: &str,
) -> Result<(), rusqlite::Error> {
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM pragma_table_info(?) WHERE name = ?",
        [table, column],
        |r| r.get(0),
    )?;
    if !exists {
        conn.execute(
            &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, col_type),
            [],
        )?;
    }
    Ok(())
}

pub fn needs_score_migration(conn: &Connection) -> Result<bool, rusqlite::Error> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('PlayerScore') WHERE name = 'hash')",
        [], |row| row.get(0),
    )
}

pub fn create_tables(conn: &Connection) -> Result<bool, rusqlite::Error> {
    // Map 表 - 谱面基本信息
    conn.execute(
        "CREATE TABLE IF NOT EXISTS Map (
            key VARCHAR(16) PRIMARY KEY,
            songName TEXT,
            songAuthorName TEXT,
            levelAuthorName TEXT,
            rating FLOAT,
            coverURL TEXT,
            previewURL TEXT
        )",
        [],
    )?;

    // MapDiff 表 - 谱面各版本的难度信息
    conn.execute(
        "CREATE TABLE IF NOT EXISTS MapDiff (
            id INTEGER PRIMARY KEY,
            key TEXT,
            hash TEXT NOT NULL,
            isCurrent INTEGER NOT NULL DEFAULT 0,
            metadataKnown INTEGER NOT NULL DEFAULT 0,
            mode TEXT NOT NULL,
            difficulty TEXT NOT NULL,
            starsRating FLOAT,
            starsFetchedAt BIGINT,
            passRating FLOAT,
            accRating FLOAT,
            techRating FLOAT,
            UNIQUE (hash, mode, difficulty)
        )",
        [],
    )?;

    // 兼容已有数据库：为已存在的 MapDiff 表添加评级列（存在则跳过）
    add_column_if_not_exists(conn, "MapDiff", "starsRating", "FLOAT")?;
    add_column_if_not_exists(conn, "MapDiff", "starsFetchedAt", "BIGINT")?;
    add_column_if_not_exists(conn, "MapDiff", "passRating", "FLOAT")?;
    add_column_if_not_exists(conn, "MapDiff", "accRating", "FLOAT")?;
    add_column_if_not_exists(conn, "MapDiff", "techRating", "FLOAT")?;

    // PlayerInfo 表 - 玩家基本信息
    conn.execute(
        "CREATE TABLE IF NOT EXISTS PlayerInfo (
            id VARCHAR(40) PRIMARY KEY,
            name TEXT,
            avatarURL TEXT,
            liveURL TEXT,
            alias TEXT,
            updatedAt BIGINT,
            scoreSyncEpoch BIGINT
        )",
        [],
    )?;

    add_column_if_not_exists(conn, "PlayerInfo", "updatedAt", "BIGINT")?;
    let has_sync_epoch: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('PlayerInfo') WHERE name = 'scoreSyncEpoch')",
        [], |row| row.get(0),
    )?;

    // PlayerScore 表 - 玩家成绩记录
    conn.execute(
        "CREATE TABLE IF NOT EXISTS PlayerScore (
            playerId TEXT NOT NULL REFERENCES PlayerInfo(id),
            mapDiffId INTEGER NOT NULL REFERENCES MapDiff(id),
            scoreId INTEGER NOT NULL DEFAULT 0,
            accuracy REAL NOT NULL,
            epochTime INTEGER NOT NULL,
            PRIMARY KEY (playerId, mapDiffId)
        ) WITHOUT ROWID",
        [],
    )?;

    // Playlist 表 - 歌单元数据（source + identifier 组合主键）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS Playlist (
            source TEXT NOT NULL CHECK(source IN ('beatsaver', 'mapper', 'local')),
            identifier TEXT NOT NULL,
            name TEXT,
            description TEXT,
            playlistImage TEXT,
            ownerId TEXT,
            ownerName TEXT,
            totalMaps INTEGER,
            createdAt TEXT,
            songsChangedAt TEXT,
            downloadURL TEXT,
            localPath TEXT,
            lastBplistFetchedAt BIGINT,
            updatedAt BIGINT,
            PRIMARY KEY (source, identifier)
        )",
        [],
    )?;

    let migrated = needs_score_migration(conn)?;
    if migrated {
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(include_str!("normalize_scores.sql"))?;
        tx.commit()?;
    }
    // Accept databases produced by the earlier normalization without replay IDs.
    add_column_if_not_exists(conn, "PlayerScore", "scoreId", "INTEGER NOT NULL DEFAULT 0")?;
    if !has_sync_epoch {
        let tx = conn.unchecked_transaction()?;
        tx.execute("ALTER TABLE PlayerInfo ADD COLUMN scoreSyncEpoch BIGINT", [])?;
        // Earlier normalized databases discarded IDs. Re-fetch those players
        // once so their existing replay links can be restored as well.
        tx.execute("UPDATE PlayerInfo SET scoreSyncEpoch = (SELECT MAX(epochTime) FROM PlayerScore WHERE playerId = PlayerInfo.id)
            WHERE NOT EXISTS (SELECT 1 FROM PlayerScore WHERE playerId = PlayerInfo.id AND scoreId = 0)", [])?;
        tx.commit()?;
    }
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_mapdiff_key ON MapDiff(key);
         CREATE INDEX IF NOT EXISTS idx_playerscore_diff ON PlayerScore(mapDiffId);
         CREATE INDEX IF NOT EXISTS idx_playerscore_player_time ON PlayerScore(playerId, epochTime DESC);
         PRAGMA user_version = 2;"
    )?;
    Ok(migrated)
}
