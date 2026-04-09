import { useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChatMessage } from './ChatMessage';
import { MessageInput } from './MessageInput';
import { TypingIndicator } from './TypingIndicator';
import { useChatStore } from '../../store/chatStore';

export function ChatPanel() {
  const { messages, isThinking, streamingText } = useChatStore();
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages, streamingText]);

  return (
    <div className="flex-1 flex flex-col min-h-0">
      {/* Messages area */}
      <div
        ref={containerRef}
        className="flex-1 overflow-y-auto p-4 space-y-4"
      >
        {/* Welcome message if empty */}
        {messages.length === 0 && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="text-center py-20"
          >
            <div className="text-6xl mb-4">🐝</div>
            <h2 className="text-2xl font-bold gradient-text mb-2">Welcome to BeCode!</h2>
            <p className="text-gray-400 max-w-md mx-auto">
              I'm your AI coding assistant. Ask me to help with code, fix bugs,
              explain concepts, or work on your project.
            </p>
            <div className="mt-6 flex flex-wrap justify-center gap-2">
              {[
                'Fix the bug in my code',
                'Explain this function',
                'Refactor for performance',
                'Write tests for this',
              ].map((suggestion) => (
                <button
                  key={suggestion}
                  className="px-4 py-2 rounded-full bg-bee-light hover:bg-bee-yellow/20 text-sm text-gray-300 hover:text-bee-yellow transition-colors"
                >
                  {suggestion}
                </button>
              ))}
            </div>
          </motion.div>
        )}

        {/* Messages */}
        <AnimatePresence mode="popLayout">
          {messages.map((message, index) => (
            <motion.div
              key={message.id}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.95 }}
              transition={{ duration: 0.2, delay: index * 0.05 }}
            >
              <ChatMessage message={message} />
            </motion.div>
          ))}
        </AnimatePresence>

        {/* Streaming text */}
        {streamingText && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
          >
            <ChatMessage
              message={{
                id: 'streaming',
                role: 'assistant',
                content: streamingText,
                timestamp: new Date().toISOString(),
              }}
              isStreaming
            />
          </motion.div>
        )}

        {/* Typing indicator */}
        {isThinking && !streamingText && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
          >
            <TypingIndicator />
          </motion.div>
        )}

        {/* Scroll anchor */}
        <div ref={messagesEndRef} />
      </div>

      {/* Input area */}
      <MessageInput />
    </div>
  );
}
