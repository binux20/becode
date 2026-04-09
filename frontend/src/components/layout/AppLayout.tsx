import { motion } from 'framer-motion';
import { Header } from './Header';
import { Sidebar } from './Sidebar';
import { StatusBar } from './StatusBar';
import { ChatPanel } from '../chat/ChatPanel';
import { useUIStore } from '../../store/uiStore';

export function AppLayout() {
  const { sidebarOpen, sidebarWidth } = useUIStore();

  return (
    <div className="h-screen flex flex-col bg-bee-dark">
      {/* Header */}
      <Header />

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar */}
        <motion.div
          initial={false}
          animate={{
            width: sidebarOpen ? sidebarWidth : 0,
            opacity: sidebarOpen ? 1 : 0,
          }}
          transition={{ duration: 0.2, ease: 'easeInOut' }}
          className="h-full overflow-hidden border-r border-panel-border"
        >
          <Sidebar />
        </motion.div>

        {/* Chat Area */}
        <div className="flex-1 flex flex-col min-w-0">
          <ChatPanel />
        </div>
      </div>

      {/* Status Bar */}
      <StatusBar />
    </div>
  );
}
