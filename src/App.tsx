import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import Players from './pages/Players';
import Playlists from './pages/Playlists';
import ScoreTable from './pages/ScoreTable';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<ScoreTable />} />
          <Route path="players" element={<Players />} />
          <Route path="playlists" element={<Playlists />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;