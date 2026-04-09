import { motion } from 'framer-motion';
import { Bot, Zap, AlertCircle } from 'lucide-react';
import { useChatStore } from '../../store/chatStore';
import { useSettingsStore } from '../../store/settingsStore';

export function StatusBar() {
  const { status, messages, error } = useChatStore();
  const { currentProvider, currentModel, config } = useSettingsStore();

  const getStatusColor = () => {
    switch (status) {
      case 'thinking':
      case 'compacting':
        return 'text-bee-yellow';
      case 'error':
        return 'text-red-400';
      case 'cancelled':
        return 'text-orange-400';
      default:
        return 'text-green-400';
    }
  };

  const getStatusText = () => {
    switch (status) {
      case 'thinking':
        return 'Thinking...';
      case 'compacting':
        return 'Compacting context...';
      case 'error':
        return error || 'Error';
      case 'cancelled':
        return 'Cancelled';
      default:
        return 'Ready';
    }
  };

  // Estimate tokens (rough approximation)
  const estimatedTokens = messages.reduce((acc, msg) => {
    return acc + Math.ceil(msg.content.length / 4);
  }, 0);

  return (
    <footer className="h-8 flex items-center justify-between px-4 bg-bee-darker border-t border-panel-border text-xs text-gray-500">
      {/* Left section */}
      <div className="flex items-center gap-4">
        {/* Status */}
        <div className="flex items-center gap-2">
          <motion.div
            animate={status === 'thinking' ? { scale: [1, 1.2, 1] } : {}}
            transition={{ repeat: Infinity, duration: 1 }}
          >
            {status === 'error' ? (
              <AlertCircle size={14} className="text-red-400" />
            ) : (
              <Zap size={14} className={getStatusColor()} />
            )}
          </motion.div>
          <span className={getStatusColor()}>{getStatusText()}</span>
        </div>

        {/* Messages count */}
        <span>Messages: {messages.length}</span>

        {/* Estimated tokens */}
        <span>~{estimatedTokens.toLocaleString()} tokens</span>
      </div>

      {/* Center section */}
      <div className="flex items-center gap-4">
        {/* Provider & Model */}
        <span>
          {currentProvider} / {currentModel || 'default'}
        </span>
      </div>

      {/* Right section */}
      <div className="flex items-center gap-4">
        {/* Sub-agents status */}
        <div className="flex items-center gap-2">
          <Bot size={14} className={config?.subAgents?.enabled ? 'text-bee-yellow' : 'text-gray-600'} />
          <span className={config?.subAgents?.enabled ? 'text-bee-yellow' : 'text-gray-600'}>
            Agents: {config?.subAgents?.enabled ? 'ON' : 'OFF'}
          </span>
        </div>

        {/* Keyboard shortcuts hint */}
        <span className="text-gray-600">
          Ctrl+K: Commands • Ctrl+Enter: Send
        </span>
      </div>
    </footer>
  );
}
