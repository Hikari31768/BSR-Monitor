import { Outlet } from 'react-router-dom';
import Sidebar from './Sidebar';
import BackgroundTasks from './BackgroundTasks';
import { AudioProvider } from '../contexts/AudioContext';

export default function Layout() {
  return (
    <AudioProvider>
      <div className="h-screen bg-bs-bg-primary flex overflow-hidden">
        <Sidebar />
        <main className="flex-1 p-8 overflow-y-auto score-table-scroll">
          <Outlet />
        </main>
        <BackgroundTasks />
      </div>
    </AudioProvider>
  );
}