import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import Layout from './components/Layout';
import Players from './pages/Players';
import Playlists from './pages/Playlists';
import MapSearch from './pages/MapSearch';
import ScoreLookup from './pages/ScoreLookup';
import TestPage from './pages/TestPage';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Navigate to="/players" replace />} />
          <Route path="players" element={<Players />} />
          <Route path="playlists" element={<Playlists />} />
          <Route path="maps" element={<MapSearch />} />
          <Route path="scores" element={<ScoreLookup />} />
          <Route path="test" element={<TestPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;