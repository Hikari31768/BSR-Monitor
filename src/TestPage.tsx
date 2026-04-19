import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function TestPage() {
  const [result, setResult] = useState<string>("");
  const [playerId, setPlayerId] = useState<string>("");
  const [playlistId, setPlaylistId] = useState<string>("1");
  const [mapKey, setMapKey] = useState<string>("1f49");
  const [leaderboardId, setLeaderboardId] = useState<string>("4b302xx91");
  const [loading, setLoading] = useState<boolean>(false);

  const showResult = (data: unknown) => {
    setResult(JSON.stringify(data, null, 2));
    setLoading(false);
  };

  const showError = (e: unknown) => {
    setResult(`Error: ${e}`);
    setLoading(false);
  };

  // 玩家管理
  const testGetPlayerList = async () => {
    setLoading(true);
    try {
      const players = await invoke("get_player_list");
      showResult(players);
    } catch (e) { showError(e); }
  };

  const testAddPlayer = async () => {
    if (!playerId) { setResult("请输入玩家 ID"); return; }
    setLoading(true);
    try {
      const player = await invoke("add_player", { id: playerId, liveUrl: null, alias: null });
      showResult(player);
    } catch (e) { showError(e); }
  };

  const testGetPlayer = async () => {
    if (!playerId) { setResult("请输入玩家 ID"); return; }
    setLoading(true);
    try {
      const player = await invoke("get_player", { id: playerId });
      showResult(player);
    } catch (e) { showError(e); }
  };

  const testDeletePlayer = async () => {
    if (!playerId) { setResult("请输入玩家 ID"); return; }
    setLoading(true);
    try {
      await invoke("delete_player", { id: playerId });
      showResult("删除成功");
    } catch (e) { showError(e); }
  };

  const testUpdatePlayerScores = async () => {
    if (!playerId) { setResult("请输入玩家 ID"); return; }
    setLoading(true);
    try {
      const count = await invoke("update_player_scores", { playerId });
      showResult(`更新了 ${count} 条成绩`);
    } catch (e) { showError(e); }
  };

  const testGetPlayerScores = async () => {
    if (!playerId) { setResult("请输入玩家 ID"); return; }
    setLoading(true);
    try {
      const scores = await invoke("get_player_scores", { playerId });
      showResult(scores);
    } catch (e) { showError(e); }
  };

  const testGetPlayerScoreCount = async () => {
    if (!playerId) { setResult("请输入玩家 ID"); return; }
    setLoading(true);
    try {
      const count = await invoke("get_player_score_count", { playerId });
      showResult(`成绩数量: ${count}`);
    } catch (e) { showError(e); }
  };

  // 谱面管理
  const testGetMapByKey = async () => {
    setLoading(true);
    try {
      const map = await invoke("get_map_by_key", { key: mapKey });
      showResult(map);
    } catch (e) { showError(e); }
  };

  const testGetMapByHash = async () => {
    setLoading(true);
    try {
      const map = await invoke("get_map_by_hash", { hash: "0e9507ff8018c554b5eb589348a5524dce965f7d" });
      showResult(map);
    } catch (e) { showError(e); }
  };

  // 歌单管理
  const testGetPlaylistList = async () => {
    setLoading(true);
    try {
      const playlists = await invoke("get_playlist_list");
      showResult(playlists);
    } catch (e) { showError(e); }
  };

  const testAddPlaylist = async () => {
    setLoading(true);
    try {
      const playlist = await invoke("add_playlist", { playlistId: parseInt(playlistId) });
      showResult(playlist);
    } catch (e) { showError(e); }
  };

  const testGetPlaylist = async () => {
    setLoading(true);
    try {
      const playlist = await invoke("get_playlist", { playlistId: parseInt(playlistId) });
      showResult(playlist);
    } catch (e) { showError(e); }
  };

  const testDeletePlaylist = async () => {
    setLoading(true);
    try {
      await invoke("delete_playlist", { playlistId: parseInt(playlistId) });
      showResult("删除成功");
    } catch (e) { showError(e); }
  };

  const testGetPlaylistMaps = async () => {
    setLoading(true);
    try {
      const maps = await invoke("get_playlist_maps", { playlistId: parseInt(playlistId) });
      showResult(maps);
    } catch (e) { showError(e); }
  };

  // 排行榜
  const testGetLeaderboard = async () => {
    setLoading(true);
    try {
      const leaderboard = await invoke("get_leaderboard", { leaderboardId });
      showResult(leaderboard);
    } catch (e) { showError(e); }
  };

  return (
    <div style={{ padding: "20px", fontFamily: "system-ui, sans-serif" }}>
      <h2>命令层测试页面</h2>

      {/* 玩家管理 */}
      <section style={{ marginBottom: "20px" }}>
        <h3>玩家管理</h3>
        <div style={{ marginBottom: "10px" }}>
          <input
            type="text"
            placeholder="玩家 ID (如: 76561198035353849)"
            value={playerId}
            onChange={(e) => setPlayerId(e.target.value)}
            style={{ width: "250px", padding: "5px", marginRight: "10px" }}
          />
        </div>
        <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
          <button onClick={testAddPlayer} disabled={loading}>添加玩家</button>
          <button onClick={testGetPlayer} disabled={loading}>获取玩家</button>
          <button onClick={testGetPlayerList} disabled={loading}>获取玩家列表</button>
          <button onClick={testDeletePlayer} disabled={loading}>删除玩家</button>
        </div>
        <div style={{ display: "flex", gap: "10px", marginTop: "10px", flexWrap: "wrap" }}>
          <button onClick={testUpdatePlayerScores} disabled={loading}>更新成绩</button>
          <button onClick={testGetPlayerScores} disabled={loading}>获取成绩列表</button>
          <button onClick={testGetPlayerScoreCount} disabled={loading}>获取成绩数量</button>
        </div>
      </section>

      {/* 谱面管理 */}
      <section style={{ marginBottom: "20px" }}>
        <h3>谱面管理</h3>
        <div style={{ marginBottom: "10px" }}>
          <input
            type="text"
            placeholder="谱面 Key"
            value={mapKey}
            onChange={(e) => setMapKey(e.target.value)}
            style={{ width: "150px", padding: "5px", marginRight: "10px" }}
          />
        </div>
        <div style={{ display: "flex", gap: "10px" }}>
          <button onClick={testGetMapByKey} disabled={loading}>通过 Key 获取谱面</button>
          <button onClick={testGetMapByHash} disabled={loading}>通过 Hash 获取谱面</button>
        </div>
      </section>

      {/* 歌单管理 */}
      <section style={{ marginBottom: "20px" }}>
        <h3>歌单管理</h3>
        <div style={{ marginBottom: "10px" }}>
          <input
            type="text"
            placeholder="歌单 ID"
            value={playlistId}
            onChange={(e) => setPlaylistId(e.target.value)}
            style={{ width: "100px", padding: "5px", marginRight: "10px" }}
          />
        </div>
        <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
          <button onClick={testAddPlaylist} disabled={loading}>添加歌单</button>
          <button onClick={testGetPlaylist} disabled={loading}>获取歌单</button>
          <button onClick={testGetPlaylistList} disabled={loading}>获取歌单列表</button>
          <button onClick={testDeletePlaylist} disabled={loading}>删除歌单</button>
          <button onClick={testGetPlaylistMaps} disabled={loading}>获取歌单谱面</button>
        </div>
      </section>

      {/* 排行榜 */}
      <section style={{ marginBottom: "20px" }}>
        <h3>排行榜</h3>
        <div style={{ marginBottom: "10px" }}>
          <input
            type="text"
            placeholder="Leaderboard ID"
            value={leaderboardId}
            onChange={(e) => setLeaderboardId(e.target.value)}
            style={{ width: "150px", padding: "5px", marginRight: "10px" }}
          />
        </div>
        <button onClick={testGetLeaderboard} disabled={loading}>获取排行榜</button>
      </section>

      {/* 结果展示 */}
      <section>
        <h3>结果</h3>
        <pre style={{
          background: "#1e1e1e",
          color: "#d4d4d4",
          padding: "15px",
          borderRadius: "8px",
          overflow: "auto",
          maxHeight: "400px",
          whiteSpace: "pre-wrap",
          fontSize: "14px"
        }}>
          {loading ? "加载中..." : (result || "点击上方按钮测试命令")}
        </pre>
      </section>
    </div>
  );
}

export default TestPage;