import { useState } from "react";
import TestPage from "./TestPage";

function App() {
  const [count, setCount] = useState(0);
  const [showTest, setShowTest] = useState(false);

  if (showTest) {
    return (
      <div>
        <TestPage />
        <div style={{ padding: "20px" }}>
          <button onClick={() => setShowTest(false)}>返回主页</button>
        </div>
      </div>
    );
  }

  return (
    <div className="container">
      <h1>BSR Monitor</h1>
      <p>BeatSaber 玩家成绩监控应用</p>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
      </div>
      <div className="card" style={{ marginTop: "20px" }}>
        <button onClick={() => setShowTest(true)}>命令层测试页面</button>
      </div>
    </div>
  );
}

export default App;