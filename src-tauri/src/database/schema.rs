use rusqlite::Connection;

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
            PRIMARY KEY (key, hash, mode, difficulty),
            FOREIGN KEY (key) REFERENCES Map(key)
        )",
        [],
    )?;

    // 兼容已有数据库：为已存在的 MapDiff 表添加星级列
    conn.execute(
        "ALTER TABLE MapDiff ADD COLUMN starsRating FLOAT",
        [],
    )?;
    conn.execute(
        "ALTER TABLE MapDiff ADD COLUMN starsFetchedAt BIGINT",
        [],
    )?;

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

    // Playlist 表 - 歌单元数据
    conn.execute(
        "CREATE TABLE IF NOT EXISTS Playlist (
            playlistId INTEGER PRIMARY KEY,
            name TEXT,
            description TEXT,
            playlistImage TEXT,
            ownerId INTEGER,
            ownerName TEXT,
            totalMaps INTEGER,
            createdAt TEXT,
            songsChangedAt TEXT,
            downloadURL TEXT
        )",
        [],
    )?;

    // PlaylistMap 表 - 歌单与谱面的关联关系
    conn.execute(
        "CREATE TABLE IF NOT EXISTS PlaylistMap (
            playlistId INTEGER NOT NULL,
            mapKey VARCHAR(16) NOT NULL,
            mapOrder FLOAT NOT NULL,
            PRIMARY KEY (playlistId, mapKey),
            FOREIGN KEY (playlistId) REFERENCES Playlist(playlistId),
            FOREIGN KEY (mapKey) REFERENCES Map(key)
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
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_playlistmap_playlist ON PlaylistMap(playlistId)",
        [],
    )?;

    Ok(())
}