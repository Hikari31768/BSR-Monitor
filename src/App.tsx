import { useState } from "react";

function App() {
  const [count, setCount] = useState(0);

  return (
    <div className="container">
      <h1>BSR Monitor</h1>
      <p>BeatSaber 玩家成绩监控应用</p>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
      </div>
    </div>
  );
}

export default App;