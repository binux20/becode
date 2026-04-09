import { useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { AppLayout } from './components/layout/AppLayout';
import { SettingsModal } from './components/settings/SettingsModal';
import { CommandPalette } from './components/common/CommandPalette';
import { useSettingsStore } from './store/settingsStore';
import { useUIStore } from './store/uiStore';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';

function App() {
  const { loadConfig, loadProviders } = useSettingsStore();
  const { settingsOpen, commandPaletteOpen, closeSettings, closeCommandPalette } = useUIStore();

  // Initialize app
  useEffect(() => {
    loadConfig();
    loadProviders();
  }, [loadConfig, loadProviders]);

  // Setup keyboard shortcuts
  useKeyboardShortcuts();

  return (
    <div className="h-screen w-screen overflow-hidden bg-bee-dark">
      {/* Main Layout */}
      <AppLayout />

      {/* Settings Modal */}
      <AnimatePresence>
        {settingsOpen && (
          <SettingsModal onClose={closeSettings} />
        )}
      </AnimatePresence>

      {/* Command Palette */}
      <AnimatePresence>
        {commandPaletteOpen && (
          <CommandPalette onClose={closeCommandPalette} />
        )}
      </AnimatePresence>
    </div>
  );
}

export default App;
