import { motion } from 'framer-motion';
import { Settings, FolderOpen, Command } from 'lucide-react';
import { useSettingsStore } from '../../store/settingsStore';
import { useUIStore } from '../../store/uiStore';
import { useChatStore } from '../../store/chatStore';
import { BeeLogo } from '../bee/BeeLogo';

export function Header() {
  const { currentProvider, currentModel, providers, models, setProvider, setModel, loadModels } = useSettingsStore();
  const { openSettings, toggleCommandPalette, toggleSidebar, sidebarOpen } = useUIStore();
  const { isThinking } = useChatStore();

  const handleProviderChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const provider = e.target.value;
    setProvider(provider);
    loadModels(provider);
  };

  return (
    <header className="h-14 flex items-center justify-between px-4 bg-bee-darker border-b border-panel-border">
      {/* Left section */}
      <div className="flex items-center gap-4">
        {/* Sidebar toggle */}
        <button
          onClick={toggleSidebar}
          className="p-2 rounded-lg hover:bg-bee-light transition-colors"
          title="Toggle Sidebar (Ctrl+B)"
        >
          <motion.div
            animate={{ rotate: sidebarOpen ? 0 : 180 }}
            transition={{ duration: 0.2 }}
          >
            <FolderOpen size={20} className="text-gray-400" />
          </motion.div>
        </button>

        {/* Logo */}
        <div className="flex items-center gap-2">
          <BeeLogo size={32} animated={isThinking} />
          <h1 className="text-xl font-bold">
            <span className="gradient-text">Bee</span>
            <span className="text-white">Code</span>
          </h1>
        </div>

        {/* Thinking indicator */}
        {isThinking && (
          <motion.div
            initial={{ opacity: 0, x: -10 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -10 }}
            className="flex items-center gap-2 text-bee-yellow"
          >
            <div className="typing-indicator">
              <span></span>
              <span></span>
              <span></span>
            </div>
            <span className="text-sm">Thinking...</span>
          </motion.div>
        )}
      </div>

      {/* Center section - Provider & Model selectors */}
      <div className="flex items-center gap-3">
        {/* Provider selector */}
        <select
          value={currentProvider}
          onChange={handleProviderChange}
          className="bg-bee-light border border-panel-border rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:border-bee-yellow transition-colors"
        >
          {providers.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>

        {/* Model selector */}
        <select
          value={currentModel}
          onChange={(e) => setModel(e.target.value)}
          className="bg-bee-light border border-panel-border rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:border-bee-yellow transition-colors min-w-[200px]"
        >
          {models.map((m) => (
            <option key={m.id} value={m.id}>
              {m.name}
            </option>
          ))}
        </select>
      </div>

      {/* Right section */}
      <div className="flex items-center gap-2">
        {/* Command Palette button */}
        <button
          onClick={toggleCommandPalette}
          className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-bee-light hover:bg-opacity-80 transition-colors text-gray-400 hover:text-white"
          title="Command Palette (Ctrl+K)"
        >
          <Command size={16} />
          <span className="text-sm">Ctrl+K</span>
        </button>

        {/* Settings button */}
        <button
          onClick={openSettings}
          className="p-2 rounded-lg hover:bg-bee-light transition-colors text-gray-400 hover:text-bee-yellow"
          title="Settings (Ctrl+,)"
        >
          <Settings size={20} />
        </button>
      </div>
    </header>
  );
}
