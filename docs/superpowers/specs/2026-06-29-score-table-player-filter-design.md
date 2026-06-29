# 打谱查询页玩家成绩筛选设计

**日期**：2026-06-29  
**范围**：`src/pages/ScoreTable.tsx`、`src/components/ScoreTableHeader.tsx`  
**状态**：已实现

---

## 1. 背景与目标

在「打谱查询」页面的歌单模式下，用户希望针对每个已选玩家快速过滤出：

1. 该玩家 **有过成绩** 的谱面（任意难度）。
2. 该玩家 **没有成绩** 的谱面（所有难度均无成绩）。

这样可以在多人对比场景下，快速定位每个人「打过的图」或「还没打的图」。

---

## 2. 设计原则

- **仅歌单模式生效**：单谱面模式只有一张图，过滤无意义，不显示筛选按钮。
- **按玩家独立**：不同玩家可以同时设置不同筛选条件，彼此之间互不影响。
- **同一人互斥**：一个玩家同一时刻只能启用「有成绩」或「无成绩」之一；切换到另一个会自动替换。
- ** toggle 行为**：点击已激活的筛选按钮会关闭筛选。
- **不持久化**：筛选状态只保留在当前页面会话内，切换歌单、翻页时保留；关闭页面或退出应用后重置。
- **不修改后端**：过滤逻辑完全在前端完成，不新增 Rust 命令或数据库变更。

---

## 3. UI 设计

### 3.1 位置与布局

在歌单模式下，成绩表格表头的每个玩家列中，于玩家头像 **左侧** 新增两个 20×20 的小按钮，与右侧已有的「删除」「刷新」按钮呈镜像布局。

```
┌─────────────────────────────────────────┐
│  [✓]  [头像]  [×]                       │
│  [−]  [名字]  [↻]                       │
└─────────────────────────────────────────┘
```

- 左上：`✓`（有成绩）
- 左下：`−`（无成绩）
- 右上：`×`（移除玩家，已有）
- 右下：`↻`（刷新成绩，已有）

### 3.2 图标与 tooltip

| 图标 | 含义 | Tooltip |
|------|------|---------|
| ✓ | 有成绩 | 筛选此人有过成绩的谱面（任何难度） |
| − | 无成绩 | 筛选此人没有成绩的谱面（所有难度） |

### 3.3 激活状态

采用 **头像角标** 表示当前筛选生效：

- 激活「有成绩」：头像右上角显示 **绿色** 小圆点。
- 激活「无成绩」：头像右上角显示 **灰色** 小圆点。
- 未激活：无角标。

由于同一人两种状态互斥，不会同时出现两个角标。

### 3.4 显示条件与可见性

- 仅在 `viewMode === 'playlist'` 时渲染这两个按钮。单谱面模式下不渲染。
- 未启用筛选时，按钮行为与现有「删除」「刷新」按钮保持一致：默认隐藏，鼠标悬停在玩家列上时显示。
- 该玩家启用任一筛选后，两个按钮 **常显**，方便用户随时切换或关闭筛选。

---

## 4. 筛选规则

### 4.1 作用粒度

以 **谱面（map）** 为最小过滤单位，而不是难度行。

- **有成绩**：该玩家在当前谱面的任意难度中存在成绩（`PlayerScoreCell` 非空）。旧版本成绩也算有成绩。
- **无成绩**：该玩家在当前谱面的所有难度中均无成绩。

### 4.2 多玩家组合

各玩家之间的筛选条件使用 **AND** 逻辑组合。

示例：
- 玩家 A 选「有成绩」、玩家 B 选「无成绩」。
- 结果：只保留「A 任意难度有成绩 **且** B 所有难度无成绩」的谱面。

### 4.3 状态模型

每个玩家拥有三种独立状态：

```ts
type PlayerFilterState = 'none' | 'has_scores' | 'no_scores';

const [playerFilters, setPlayerFilters] = useState<Record<string, PlayerFilterState>>({});
```

- `none`：未启用筛选（默认）。
- `has_scores`：启用「有成绩」筛选。
- `no_scores`：启用「无成绩」筛选。

状态切换规则：

| 当前状态 | 点击按钮 | 新状态 |
|----------|----------|--------|
| none | 有成绩 | has_scores |
| none | 无成绩 | no_scores |
| has_scores | 有成绩 | none |
| has_scores | 无成绩 | no_scores |
| no_scores | 有成绩 | has_scores |
| no_scores | 无成绩 | none |

---

## 5. 数据流与实现

### 5.1 状态管理

- `ScoreTable.tsx` 中新增 `playerFilters` 状态。
- 通过 props 传递给 `ScoreTableHeader`：
  - `playerFilters: Record<string, PlayerFilterState>`
  - `onTogglePlayerFilter: (playerId: string, type: 'has_scores' | 'no_scores') => void`
- `ScoreTableHeader` 负责渲染按钮并响应点击。

### 5.2 过滤逻辑

在 `ScoreTable.tsx` 的 `displayData` 计算中，对歌单模式的 `playlistData.maps` 应用过滤：

```ts
const displayData = useMemo(() => {
  if (viewMode === null) return [];
  const data = viewMode === 'single'
    ? (currentMapData ? [currentMapData] : [])
    : (playlistData?.maps || []);

  if (viewMode !== 'playlist') return data;

  return data.filter(mapData => {
    return Object.entries(playerFilters).every(([playerId, filter]) => {
      if (filter === 'none') return true;
      const hasScore = mapData.scores.some(row =>
        row.scores.some(s => s.player_id === playerId)
      );
      return filter === 'has_scores' ? hasScore : !hasScore;
    });
  });
}, [viewMode, currentMapData, playlistData, playerFilters]);
```

### 5.3 空结果提示

当组合筛选后没有符合条件的谱面时，表格空状态显示：

> 没有符合当前筛选条件的谱面

---

## 6. 边界情况

| 场景 | 行为 |
|------|------|
| 移除玩家 | 自动清理该玩家的筛选状态。 |
| 添加玩家 | 新玩家默认 `none`，不继承任何筛选。 |
| 切换歌单 | 筛选状态保留（当前页面会话内）。 |
| 翻页 | 筛选状态保留，对当前页数据继续生效。 |
| 结果为空 | 显示「没有符合当前筛选条件的谱面」。 |
| 单谱面模式 | 不显示筛选按钮，也不进行过滤。 |
| 关闭页面/退出应用 | 筛选状态丢失，下次打开为默认状态。 |

---

## 7. 不做的内容

- 不持久化到 `UserPrefs` 或数据库。
- 不新增 Rust 命令或后端 API。
- 不过滤到难度行级别（只到谱面级别）。
- 不支持 OR 逻辑组合多个玩家的筛选条件。

---

## 8. 相关文件

- `src/pages/ScoreTable.tsx`
- `src/components/ScoreTableHeader.tsx`
- `src/types/index.ts`（可能需要导出 `PlayerFilterState` 类型）

---

## 9. 验收标准

- [x] 歌单模式下每个玩家列左侧显示 ✓ 和 − 两个按钮。
- [x] 单谱面模式下不显示筛选按钮。
- [x] 点击 ✓ 后只显示该玩家有成绩的谱面，头像出现绿色角标。
- [x] 点击 − 后只显示该玩家无成绩的谱面，头像出现灰色角标。
- [x] 同一玩家两种筛选状态互斥，再次点击已激活按钮可关闭筛选。
- [x] 多玩家同时筛选时结果取 AND 交集。
- [x] 移除玩家后该玩家的筛选状态被清理。
- [x] 筛选结果为空时显示正确提示。
- [x] 切换歌单或翻页后筛选状态在当前页面会话内保留。

---

## 10. 实现记录

### 本次操作

1. 在 `src/types/index.ts` 新增 `PlayerFilterState` 类型（`'none' | 'has_scores' | 'no_scores'`）。
2. 在 `src/components/ScoreTableHeader.tsx`：
   - 扩展 props，接收 `viewMode`、`playerFilters`、`onTogglePlayerFilter`。
   - 仅在 `viewMode === 'playlist'` 时渲染左侧 ✓/− 筛选按钮。
   - 按钮默认随 `group-hover` 显示，激活状态常显。
   - 在玩家头像右上角根据当前筛选状态渲染绿色（有成绩）或灰色（无成绩）角标。
3. 在 `src/pages/ScoreTable.tsx`：
   - 新增 `playerFilters` 状态，仅前端内存维护，不持久化。
   - 实现 `handleTogglePlayerFilter`，同一玩家两种状态互斥，再次点击已激活按钮会关闭。
   - 在 `handleRemovePlayer` 中同步清理对应玩家的筛选状态。
   - 在 `displayData` 计算中以谱面为粒度应用 AND 组合过滤。
   - 空状态区分「筛选结果为空」与「歌单无数据」。
   - 将新状态与回调通过 props 传递给 `ScoreTableHeader`。
4. 更新本文档状态与验收标准。

### 验证结果

- `npx tsc --noEmit` 通过，无类型错误。
- `npm run build` 通过，前端构建成功。

### 流程归纳

状态定义 → 表头 UI 与交互 → 页面状态与过滤逻辑 → 空状态与边界处理 → 类型/构建验证 → 设计文档归档。
