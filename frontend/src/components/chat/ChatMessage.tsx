import { useState } from 'react';
import { motion } from 'framer-motion';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { User, Bot, Info, Copy, Check } from 'lucide-react';
import { ChatMessage as ChatMessageType } from '../../types';
import { ToolCard } from './ToolCard';
import { CodeBlock } from '../common/CodeBlock';

interface Props {
  message: ChatMessageType;
  isStreaming?: boolean;
}

export function ChatMessage({ message, isStreaming }: Props) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(message.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const getRoleStyles = () => {
    switch (message.role) {
      case 'user':
        return {
          container: 'message-user',
          icon: <User size={18} />,
          iconBg: 'bg-cyan-600',
          label: 'You',
        };
      case 'assistant':
        return {
          container: 'message-assistant',
          icon: <Bot size={18} />,
          iconBg: 'bg-gradient-to-br from-bee-yellow to-bee-orange',
          label: 'BeCode',
        };
      case 'system':
        return {
          container: 'message-system',
          icon: <Info size={18} />,
          iconBg: 'bg-gray-600',
          label: 'System',
        };
      default:
        return {
          container: '',
          icon: null,
          iconBg: '',
          label: '',
        };
    }
  };

  const styles = getRoleStyles();

  return (
    <div className={`group relative ${message.role === 'user' ? 'ml-12' : 'mr-12'}`}>
      <div className={`flex gap-3 ${message.role === 'user' ? 'flex-row-reverse' : ''}`}>
        {/* Avatar */}
        <div className={`flex-shrink-0 w-8 h-8 rounded-full ${styles.iconBg} flex items-center justify-center text-white`}>
          {message.role === 'assistant' ? (
            <span className="text-bee-dark">🐝</span>
          ) : (
            styles.icon
          )}
        </div>

        {/* Content */}
        <div className={`flex-1 min-w-0 ${styles.container} px-4 py-3`}>
          {/* Header */}
          <div className="flex items-center justify-between mb-1">
            <span className={`text-xs font-medium ${
              message.role === 'assistant' ? 'text-bee-yellow' : 'text-gray-400'
            }`}>
              {styles.label}
            </span>
            <span className="text-xs text-gray-500">
              {new Date(message.timestamp).toLocaleTimeString()}
            </span>
          </div>

          {/* Message content */}
          <div className="prose prose-invert prose-sm max-w-none">
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              components={{
                code({ node, inline, className, children, ...props }: any) {
                  const match = /language-(\w+)/.exec(className || '');
                  const language = match ? match[1] : '';

                  if (!inline && language) {
                    return (
                      <CodeBlock language={language}>
                        {String(children).replace(/\n$/, '')}
                      </CodeBlock>
                    );
                  }

                  return (
                    <code
                      className="bg-black/30 px-1.5 py-0.5 rounded text-bee-yellow font-mono text-sm"
                      {...props}
                    >
                      {children}
                    </code>
                  );
                },
                pre({ children }) {
                  return <>{children}</>;
                },
                p({ children }) {
                  return <p className="mb-2 last:mb-0">{children}</p>;
                },
                ul({ children }) {
                  return <ul className="list-disc list-inside mb-2 space-y-1">{children}</ul>;
                },
                ol({ children }) {
                  return <ol className="list-decimal list-inside mb-2 space-y-1">{children}</ol>;
                },
                a({ href, children }) {
                  return (
                    <a
                      href={href}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-bee-yellow hover:underline"
                    >
                      {children}
                    </a>
                  );
                },
                blockquote({ children }) {
                  return (
                    <blockquote className="border-l-2 border-bee-yellow pl-4 italic text-gray-400">
                      {children}
                    </blockquote>
                  );
                },
              }}
            >
              {message.content}
            </ReactMarkdown>
          </div>

          {/* Streaming cursor */}
          {isStreaming && (
            <motion.span
              animate={{ opacity: [1, 0] }}
              transition={{ repeat: Infinity, duration: 0.8 }}
              className="inline-block w-2 h-4 bg-bee-yellow ml-1"
            />
          )}

          {/* Tool calls */}
          {message.toolCalls && message.toolCalls.length > 0 && (
            <div className="mt-3 space-y-2">
              {message.toolCalls.map((toolCall) => (
                <ToolCard key={toolCall.id} toolCall={toolCall} />
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Copy button */}
      <button
        onClick={handleCopy}
        className="absolute top-2 right-2 p-1.5 rounded opacity-0 group-hover:opacity-100 transition-opacity bg-bee-light hover:bg-bee-yellow/20 text-gray-400 hover:text-bee-yellow"
        title="Copy message"
      >
        {copied ? <Check size={14} /> : <Copy size={14} />}
      </button>
    </div>
  );
}
