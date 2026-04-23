/**
 * Centralized Playwright selectors for BSR Monitor E2E tests
 */

export const Sidebar = {
  navScoreTable: 'a',                       // nav link - 打谱查询
  navPlayers: 'a',                          // nav link - 玩家管理
  navPlaylists: 'a',                        // nav link - 歌单管理
  textScoreTable: '打谱查询',
  textPlayers: '玩家管理',
  textPlaylists: '歌单管理',
};

export const PlayersPage = {
  heading: '玩家管理',
  btnAddPlayer: '添加玩家',
  // Add player modal
  modalTitleAdd: '添加玩家',
  inputPlayerId: /玩家.*ID|Steam.*ID|player.*id/i,
  inputLiveUrl: /live.*url|直播|live/i,
  inputAlias: /备注|alias/i,
  btnModalAdd: '添加',
  btnModalCancel: '取消',
  // Player card actions
  btnUpdateScores: '更新成绩',
  btnDeletePlayer: '删除',
  btnConfirmDelete: '确认删除',
  // Status
  loadingText: '加载中',
  emptyStateText: /暂无玩家|还没有玩家/,
};

export const PlaylistsPage = {
  heading: '歌单管理',
  btnAddPlaylist: '添加歌单',
  // Add playlist modal
  modalTitleAdd: '添加歌单',
  inputPlaylistId: /歌单.*ID|playlist.*id/i,
  btnModalAdd: '添加',
  btnModalCancel: '取消',
  // Playlist card actions
  btnDeletePlaylist: '删除',
  btnViewMaps: /查看.*谱面|谱面列表/,
  btnConfirmDelete: '确认删除',
  // Status
  loadingText: '加载中',
  emptyStateText: /暂无歌单|还没有歌单/,
};

export const ScoreTablePage = {
  // Mode toggle
  btnSingleMode: '单谱面',
  btnPlaylistMode: '歌单',
  // Single mode
  inputMapKey: /输入.*key|输入.*hash|谱面/i,
  btnSearch: '查询',
  // Playlist mode
  selectPlaylist: /选择.*歌单|歌单选择/i,
  btnPrevPage: '上一页',
  btnNextPage: '下一页',
  // Player selector
  btnAddPlayer: '添加玩家',
  // Table
  scoreTable: 'table',
  // Status
  loadingText: '加载中',
  emptyStateText: /暂无|未查询/,
};

export const Modal = {
  overlay: '[role="dialog"]',
  title: 'h2, .modal-title, [class*="modal"] h2',
  btnCancel: '取消',
  btnConfirm: '确认',
};

export const Common = {
  loadingSpinner: /loading|spinner/i,
  emptyState: '[class*="empty"]',
};
