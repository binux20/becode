import { useState, useRef, useEffect, KeyboardEvent } from 'react';
import { motion } from 'framer-motion';
import { Send, Paperclip, X, Loader } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useChatStore } from '../../store/chatStore';
import { useSettingsStore } from '../../store/settingsStore';
import { ChatMessage, StreamChunk } from '../../types';

export function MessageInput() {
  const [input, setInput] = useState('');
  const [inputHistory, setInputHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const {
    addMessage,
    setThinking,
    setStatus,
    setStreamingText,
    appendStreamingText,
    clearStreamingText,
    isThinking,
  } = useChatStore();

  const { currentProvider, currentModel, projectPath } = useSettingsStore();

  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`;
    }
  }, [input]);

  // Listen for streaming events
  useEffect(() => {
    const unlistenChunk = listen<StreamChunk>('message-chunk', (event) => {
      if (event.payload.content) {
        appendStreamingText(event.payload.content);
      }
    });

    const unlistenDone = listen('message-done', () => {
      // Move streaming text to a proper message
      const { streamingText } = useChatStore.getState();
      if (streamingText) {
        const message: ChatMessage = {
          id: `msg-${Date.now()}`,
          role: 'assistant',
          content: streamingText,
          timestamp: new Date().toISOString(),
        };
        addMessage(message);
        clearStreamingText();
      }
      setThinking(false);
    });

    const unlistenStatus = listen<string>('chat-status', (event) => {
      setStatus(event.payload as any);
    });

    return () => {
      unlistenChunk.then((fn) => fn());
      unlistenDone.then((fn) => fn());
      unlistenStatus.then((fn) => fn());
    };
  }, [addMessage, appendStreamingText, clearStreamingText, setThinking, setStatus]);

  const handleSubmit = async () => {
    const trimmedInput = input.trim();
    if (!trimmedInput || isThinking) return;

    // Check for slash commands
    if (trimmedInput.startsWith('/')) {
      handleSlashCommand(trimmedInput);
      setInput('');
      return;
    }

    // Add to history
    setInputHistory((prev) => [...prev, trimmedInput]);
    setHistoryIndex(-1);

    // Add user message
    const userMessage: ChatMessage = {
      id: `msg-${Date.now()}`,
      role: 'user',
      content: trimmedInput,
      timestamp: new Date().toISOString(),
    };
    addMessage(userMessage);
    setInput('');
    setThinking(true);
    clearStreamingText();

    try {
      await invoke('send_message', {
        message: trimmedInput,
        provider: currentProvider,
        model: currentModel || null,
        projectPath: projectPath || '.',
      });
    } catch (error) {
      console.error('Failed to send message:', error);
      setThinking(false);
      setStatus('error');
    }
  };

  const handleSlashCommand = (command: string) => {
    const [cmd, ...args] = command.slice(1).split(' ');
    const arg = args.join(' ');

    switch (cmd.toLowerCase()) {
      case 'clear':
        useChatStore.getState().clearMessages();
        break;
      case 'compact':
        handleCompact();
        break;
      case 'help':
        addMessage({
          id: `msg-${Date.now()}`,
          role: 'system',
          content: `**Available Commands:**
- \`/clear\` - Clear chat history
- \`/compact\` - Compact context (summarize old messages)
- \`/save [name]\` - Save current session
- \`/load\` - Load a saved session
- \`/model [name]\` - Change model
- \`/provider [name]\` - Change provider
- \`/agents on|off\` - Toggle sub-agents
- \`/help\` - Show this help`,
          timestamp: new Date().toISOString(),
        });
        break;
      case 'save':
        // TODO: Implement save
        addMessage({
          id: `msg-${Date.now()}`,
          role: 'system',
          content: `Saving session as "${arg || 'Untitled'}"...`,
          timestamp: new Date().toISOString(),
        });
        break;
      case 'model':
        if (arg) {
          useSettingsStore.getState().setModel(arg);
          addMessage({
            id: `msg-${Date.now()}`,
            role: 'system',
            content: `Model changed to: ${arg}`,
            timestamp: new Date().toISOString(),
          });
        }
        break;
      case 'provider':
        if (arg) {
          useSettingsStore.getState().setProvider(arg);
          addMessage({
            id: `msg-${Date.now()}`,
            role: 'system',
            content: `Provider changed to: ${arg}`,
            timestamp: new Date().toISOString(),
          });
        }
        break;
      case 'agents':
        const enabled = arg.toLowerCase() === 'on';
        useSettingsStore.getState().updateSubAgents({ enabled });
        addMessage({
          id: `msg-${Date.now()}`,
          role: 'system',
          content: `Sub-agents ${enabled ? 'enabled' : 'disabled'}`,
          timestamp: new Date().toISOString(),
        });
        break;
      default:
        addMessage({
          id: `msg-${Date.now()}`,
          role: 'system',
          content: `Unknown command: /${cmd}. Type /help for available commands.`,
          timestamp: new Date().toISOString(),
        });
    }
  };

  const handleCompact = async () => {
    const { messages } = useChatStore.getState();
    if (messages.length <= 5) {
      addMessage({
        id: `msg-${Date.now()}`,
        role: 'system',
        content: 'Not enough messages to compact.',
        timestamp: new Date().toISOString(),
      });
      return;
    }

    setStatus('compacting');
    try {
      const compacted = await invoke<ChatMessage[]>('compact_context', {
        messages,
        keepLast: 5,
      });
      useChatStore.getState().loadMessages(compacted);
      addMessage({
        id: `msg-${Date.now()}`,
        role: 'system',
        content: `Context compacted. ${messages.length - compacted.length} messages summarized.`,
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      console.error('Failed to compact:', error);
    }
    setStatus('ready');
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    // Submit on Ctrl+Enter or Cmd+Enter
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      handleSubmit();
      return;
    }

    // Navigate history with Up/Down when input is empty or at start
    if (e.key === 'ArrowUp' && (input === '' || textareaRef.current?.selectionStart === 0)) {
      e.preventDefault();
      if (historyIndex < inputHistory.length - 1) {
        const newIndex = historyIndex + 1;
        setHistoryIndex(newIndex);
        setInput(inputHistory[inputHistory.length - 1 - newIndex]);
      }
    }

    if (e.key === 'ArrowDown' && historyIndex >= 0) {
      e.preventDefault();
      if (historyIndex > 0) {
        const newIndex = historyIndex - 1;
        setHistoryIndex(newIndex);
        setInput(inputHistory[inputHistory.length - 1 - newIndex]);
      } else {
        setHistoryIndex(-1);
        setInput('');
      }
    }
  };

  const handleCancel = async () => {
    try {
      await invoke('cancel_execution');
    } catch (error) {
      console.error('Failed to cancel:', error);
    }
  };

  return (
    <div className="p-4 border-t border-panel-border bg-bee-darker">
      <div className="relative flex items-end gap-2">
        {/* Textarea */}
        <div className="flex-1 relative">
          <textarea
            ref={textareaRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type your message... (Ctrl+Enter to send)"
            disabled={isThinking}
            rows={1}
            className="w-full input-bee resize-none pr-10"
            style={{ minHeight: '44px', maxHeight: '200px' }}
          />

          {/* Slash command hint */}
          {input.startsWith('/') && (
            <div className="absolute bottom-full left-0 mb-2 bg-bee-light rounded-lg shadow-lg p-2 text-sm">
              <div className="text-gray-400">Commands: /help, /clear, /compact, /save, /model, /provider, /agents</div>
            </div>
          )}
        </div>

        {/* Action buttons */}
        <div className="flex gap-2">
          {isThinking ? (
            <motion.button
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              onClick={handleCancel}
              className="p-3 rounded-lg bg-red-600 hover:bg-red-700 text-white transition-colors"
              title="Cancel"
            >
              <X size={20} />
            </motion.button>
          ) : (
            <motion.button
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              onClick={handleSubmit}
              disabled={!input.trim()}
              className="btn-bee p-3 disabled:opacity-50 disabled:cursor-not-allowed"
              title="Send (Ctrl+Enter)"
            >
              <Send size={20} />
            </motion.button>
          )}
        </div>
      </div>
    </div>
  );
}
