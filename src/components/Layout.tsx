import { Outlet } from 'react-router-dom';
import Sidebar from './Sidebar';
import { AudioProvider } from '../contexts/AudioContext';

export default function Layout() {
  return (
    <AudioProvider>
      <div className="h-screen bg-[#0d0d12] flex overflow-hidden">
        <Sidebar />
        <main className="flex-1 p-8 overflow-hidden">
          <div className="max-w-6xl mx-auto h-full">
            <Outlet />
          </div>
        </main>
      </div>
    </AudioProvider>
  );
}