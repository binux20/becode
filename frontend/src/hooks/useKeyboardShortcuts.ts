import { useEffect } from 'react';
import { useUIStore } from '../store/uiStore';
import { useChatStore } from '../store/chatStore';

export function useKeyboardShortcuts() {
  const {
    toggleSidebar,
    toggleSettings,
    toggleCommandPalette,
    settingsOpen,
    commandPaletteOpen,
  } = useUIStore();

  const { clearMessages } = useChatStore();

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't trigger shortcuts when typing in inputs (except specific ones)
      const target = e.target as HTMLElement;
      const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA';

      // Escape - close modals
      if (e.key === 'Escape') {
        if (commandPaletteOpen) {
          useUIStore.getState().closeCommandPalette();
          return;
        }
        if (settingsOpen) {
          useUIStore.getState().closeSettings();
          return;
        }
      }

      // Only process shortcuts with modifiers
      if (!e.ctrlKey && !e.metaKey) return;

      // Ctrl+K - Command Palette (works everywhere)
      if (e.key === 'k') {
        e.preventDefault();
        toggleCommandPalette();
        return;
      }

      // Skip other shortcuts when in input
      if (isInput && e.key !== 'Enter' && e.key !== 'l') return;

      // Ctrl+B - Toggle Sidebar
      if (e.key === 'b') {
        e.preventDefault();
        toggleSidebar();
        return;
      }

      // Ctrl+, - Open Settings
      if (e.key === ',') {
        e.preventDefault();
        toggleSettings();
        return;
      }

      // Ctrl+L - Clear Chat
      if (e.key === 'l') {
        e.preventDefault();
        clearMessages();
        return;
      }

      // Ctrl+Shift+C - Compact Context
      if (e.key === 'c' && e.shiftKey) {
        e.preventDefault();
        // Trigger compact via event or direct call
        // This would normally be handled by MessageInput
        return;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [toggleSidebar, toggleSettings, toggleCommandPalette, clearMessages, settingsOpen, commandPaletteOpen]);
}
