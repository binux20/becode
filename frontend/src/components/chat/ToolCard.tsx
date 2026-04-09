import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronDown, ChevronRight, Terminal, FileText, Search, Globe, CheckCircle, XCircle, Loader, Clock } from 'lucide-react';
import { ToolCallInfo } from '../../types';

interface Props {
  toolCall: ToolCallInfo;
}

export function ToolCard({ toolCall }: Props) {
  const [expanded, setExpanded] = useState(false);

  const getToolIcon = () => {
    switch (toolCall.tool) {
      case 'bash':
        return <Terminal size={14} />;
      case 'read_file':
      case 'write_file':
      case 'edit_file':
        return <FileText size={14} />;
      case 'glob_search':
      case 'grep_search':
        return <Search size={14} />;
      case 'web_fetch':
      case 'web_search':
        return <Globe size={14} />;
      default:
        return <Terminal size={14} />;
    }
  };

  const getStatusIcon = () => {
    switch (toolCall.status) {
      case 'pending':
        return <Clock size={14} className="text-gray-400" />;
      case 'running':
        return <Loader size={14} className="text-blue-400 animate-spin" />;
      case 'success':
        return <CheckCircle size={14} className="text-green-400" />;
      case 'error':
        return <XCircle size={14} className="text-red-400" />;
    }
  };

  const getStatusStyles = () => {
    switch (toolCall.status) {
      case 'pending':
        return 'border-l-gray-500';
      case 'running':
        return 'border-l-blue-500 animate-pulse';
      case 'success':
        return 'border-l-green-500';
      case 'error':
        return 'border-l-red-500';
    }
  };

  const formatArgs = () => {
    const args = toolCall.args;
    if (args.path) return args.path as string;
    if (args.command) return (args.command as string).substring(0, 50) + ((args.command as string).length > 50 ? '...' : '');
    if (args.pattern) return `Pattern: ${args.pattern}`;
    if (args.url) return args.url as string;
    return JSON.stringify(args).substring(0, 50) + '...';
  };

  return (
    <div className={`tool-card ${getStatusStyles()}`}>
      {/* Header */}
      <div
        onClick={() => setExpanded(!expanded)}
        className="flex items-center justify-between cursor-pointer"
      >
        <div className="flex items-center gap-2">
          <span className="text-bee-yellow">{getToolIcon()}</span>
          <span className="font-mono font-medium text-sm text-white">{toolCall.tool}</span>
          <span className="text-gray-500 text-xs truncate max-w-[200px]">{formatArgs()}</span>
        </div>
        <div className="flex items-center gap-2">
          {toolCall.durationMs && (
            <span className="text-xs text-gray-500">{toolCall.durationMs}ms</span>
          )}
          {getStatusIcon()}
          <motion.span
            animate={{ rotate: expanded ? 90 : 0 }}
            transition={{ duration: 0.15 }}
          >
            <ChevronRight size={14} className="text-gray-500" />
          </motion.span>
        </div>
      </div>

      {/* Expanded content */}
      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="overflow-hidden"
          >
            <div className="mt-3 pt-3 border-t border-gray-700/50 space-y-3">
              {/* Input */}
              <div>
                <div className="text-xs font-medium text-gray-500 mb-1">Input</div>
                <pre className="bg-black/30 rounded p-2 text-xs text-gray-300 overflow-x-auto">
                  {JSON.stringify(toolCall.args, null, 2)}
                </pre>
              </div>

              {/* Output */}
              {toolCall.output && (
                <div>
                  <div className="text-xs font-medium text-gray-500 mb-1">Output</div>
                  <pre className="bg-black/30 rounded p-2 text-xs text-gray-300 overflow-x-auto max-h-60">
                    {toolCall.output.length > 1000
                      ? toolCall.output.substring(0, 1000) + '\n... (truncated)'
                      : toolCall.output}
                  </pre>
                </div>
              )}

              {/* Error */}
              {toolCall.error && (
                <div>
                  <div className="text-xs font-medium text-red-400 mb-1">Error</div>
                  <pre className="bg-red-900/20 rounded p-2 text-xs text-red-300 overflow-x-auto">
                    {toolCall.error}
                  </pre>
                </div>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
