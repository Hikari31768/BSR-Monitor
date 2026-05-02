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

pub fn create_tables(conn: &Connection) -> Result<(), rusqlite::Error> {
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
            key VARCHAR(16) NOT NULL,
            hash CHAR(40) NOT NULL,
            isCurrent BOOLEAN NOT NULL,
            mode VARCHAR(16) NOT NULL,
            difficulty VARCHAR(16) NOT NULL,
            starsRating FLOAT,
            starsFetchedAt BIGINT,
            passRating FLOAT,
            accRating FLOAT,
            techRating FLOAT,
            PRIMARY KEY (key, hash, mode, difficulty),
            FOREIGN KEY (key) REFERENCES Map(key)
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
            alias TEXT
        )",
        [],
    )?;

    // PlayerScore 表 - 玩家成绩记录
    conn.execute(
        "CREATE TABLE IF NOT EXISTS PlayerScore (
            playerId VARCHAR(40) NOT NULL,
            scoreId BIGINT NOT NULL,
            leaderboardId VARCHAR(64) NOT NULL,
            mapKey VARCHAR(16),
            hash CHAR(40) NOT NULL,
            mode VARCHAR(16) NOT NULL,
            difficulty VARCHAR(16) NOT NULL,
            accuracy FLOAT NOT NULL,
            epochTime BIGINT NOT NULL,
            PRIMARY KEY (playerId, hash, mode, difficulty),
            FOREIGN KEY (playerId) REFERENCES PlayerInfo(id)
        )",
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

    // 创建索引以提升查询性能
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_mapdiff_key ON MapDiff(key)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_playerscore_player_hash ON PlayerScore(playerId, hash)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_playerscore_hash ON PlayerScore(hash)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_playerscore_time ON PlayerScore(epochTime)",
        [],
    )?;

    Ok(())
}
