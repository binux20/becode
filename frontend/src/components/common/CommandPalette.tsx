import { useState, useEffect, useRef } from 'react';
import { motion } from 'framer-motion';
import { Search, Command, Settings, Trash2, Save, FolderOpen, Bot, Zap } from 'lucide-react';
import { useUIStore } from '../../store/uiStore';
import { useChatStore } from '../../store/chatStore';
import { useSettingsStore } from '../../store/settingsStore';

interface CommandItem {
  id: string;
  label: string;
  description: string;
  icon: React.ReactNode;
  shortcut?: string;
  action: () => void;
}

interface Props {
  onClose: () => void;
}

export function CommandPalette({ onClose }: Props) {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const { openSettings, toggleSidebar } = useUIStore();
  const { clearMessages } = useChatStore();
  const { providers, setProvider, config, updateSubAgents } = useSettingsStore();

  // Define commands
  const commands: CommandItem[] = [
    {
      id: 'settings',
      label: 'Open Settings',
      description: 'Configure providers, API keys, and preferences',
      icon: <Settings size={16} />,
      shortcut: 'Ctrl+,',
      action: () => {
        openSettings();
        onClose();
      },
    },
    {
      id: 'clear',
      label: 'Clear Chat',
      description: 'Clear all messages in the current chat',
      icon: <Trash2 size={16} />,
      shortcut: 'Ctrl+L',
      action: () => {
        clearMessages();
        onClose();
      },
    },
    {
      id: 'sidebar',
      label: 'Toggle Sidebar',
      description: 'Show or hide the sidebar',
      icon: <FolderOpen size={16} />,
      shortcut: 'Ctrl+B',
      action: () => {
        toggleSidebar();
        onClose();
      },
    },
    {
      id: 'agents-toggle',
      label: config?.subAgents?.enabled ? 'Disable Sub-Agents' : 'Enable Sub-Agents',
      description: 'Toggle AI sub-agents for complex tasks',
      icon: <Bot size={16} />,
      action: () => {
        updateSubAgents({ enabled: !config?.subAgents?.enabled });
        onClose();
      },
    },
    {
      id: 'compact',
      label: 'Compact Context',
      description: 'Summarize old messages to save tokens',
      icon: <Zap size={16} />,
      shortcut: 'Ctrl+Shift+C',
      action: () => {
        // Trigger compact via message input
        onClose();
      },
    },
    // Add provider switching commands
    ...providers.map((p) => ({
      id: `provider-${p.id}`,
      label: `Switch to ${p.name}`,
      description: p.description,
      icon: <Command size={16} />,
      action: () => {
        setProvider(p.id);
        onClose();
      },
    })),
  ];

  // Filter commands
  const filteredCommands = commands.filter(
    (cmd) =>
      cmd.label.toLowerCase().includes(query.toLowerCase()) ||
      cmd.description.toLowerCase().includes(query.toLowerCase())
  );

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Reset selection when query changes
  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex((i) => Math.min(i + 1, filteredCommands.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((i) => Math.max(i - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          filteredCommands[selectedIndex].action();
        }
        break;
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 z-50 flex items-start justify-center pt-[20vh] bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <motion.div
        initial={{ opacity: 0, scale: 0.95, y: -20 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: -20 }}
        transition={{ duration: 0.15 }}
        className="w-full max-w-lg bg-bee-darker border border-panel-border rounded-xl shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search input */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-panel-border">
          <Search size={18} className="text-gray-500" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a command or search..."
            className="flex-1 bg-transparent text-white placeholder-gray-500 outline-none"
          />
          <kbd className="px-2 py-1 text-xs bg-bee-light rounded text-gray-400">ESC</kbd>
        </div>

        {/* Commands list */}
        <div className="max-h-80 overflow-y-auto py-2">
          {filteredCommands.length === 0 ? (
            <div className="px-4 py-8 text-center text-gray-500">
              No commands found
            </div>
          ) : (
            filteredCommands.map((cmd, index) => (
              <div
                key={cmd.id}
                onClick={cmd.action}
                onMouseEnter={() => setSelectedIndex(index)}
                className={`flex items-center gap-3 px-4 py-3 cursor-pointer transition-colors ${
                  index === selectedIndex
                    ? 'bg-bee-yellow/10 text-bee-yellow'
                    : 'text-gray-300 hover:bg-bee-light/50'
                }`}
              >
                <span className={index === selectedIndex ? 'text-bee-yellow' : 'text-gray-500'}>
                  {cmd.icon}
                </span>
                <div className="flex-1 min-w-0">
                  <div className="font-medium truncate">{cmd.label}</div>
                  <div className="text-xs text-gray-500 truncate">{cmd.description}</div>
                </div>
                {cmd.shortcut && (
                  <kbd className="px-2 py-1 text-xs bg-bee-light rounded text-gray-400">
                    {cmd.shortcut}
                  </kbd>
                )}
              </div>
            ))
          )}
        </div>
      </motion.div>
    </motion.div>
  );
}
