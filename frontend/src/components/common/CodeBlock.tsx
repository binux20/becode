import { useState } from 'react';
import { motion } from 'framer-motion';
import { Copy, Check } from 'lucide-react';

interface Props {
  children: string;
  language?: string;
}

export function CodeBlock({ children, language }: Props) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(children);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const getLanguageLabel = () => {
    const labels: Record<string, string> = {
      js: 'JavaScript',
      ts: 'TypeScript',
      tsx: 'TypeScript React',
      jsx: 'JavaScript React',
      py: 'Python',
      rs: 'Rust',
      go: 'Go',
      java: 'Java',
      cpp: 'C++',
      c: 'C',
      css: 'CSS',
      html: 'HTML',
      json: 'JSON',
      yaml: 'YAML',
      yml: 'YAML',
      md: 'Markdown',
      sql: 'SQL',
      bash: 'Bash',
      sh: 'Shell',
      toml: 'TOML',
    };
    return labels[language || ''] || language?.toUpperCase() || 'Code';
  };

  return (
    <div className="relative group my-3">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-black/50 rounded-t-lg border-b border-gray-700/50">
        <span className="text-xs font-medium text-gray-400">{getLanguageLabel()}</span>
        <motion.button
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
          onClick={handleCopy}
          className="flex items-center gap-1 px-2 py-1 rounded text-xs text-gray-400 hover:text-white hover:bg-gray-700/50 transition-colors"
        >
          {copied ? (
            <>
              <Check size={12} className="text-green-400" />
              <span className="text-green-400">Copied!</span>
            </>
          ) : (
            <>
              <Copy size={12} />
              <span>Copy</span>
            </>
          )}
        </motion.button>
      </div>

      {/* Code content */}
      <pre className="bg-black/40 rounded-b-lg p-4 overflow-x-auto">
        <code className={`text-sm font-mono text-gray-200 language-${language}`}>
          {children}
        </code>
      </pre>
    </div>
  );
}
