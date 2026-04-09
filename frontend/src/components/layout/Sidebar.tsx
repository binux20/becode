import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { FolderTree, History, ChevronDown, ChevronRight, Trash2, Download } from 'lucide-react';
import { useFileTreeStore } from '../../store/fileTreeStore';
import { useSessionStore } from '../../store/sessionStore';
import { useSettingsStore } from '../../store/settingsStore';
import { useChatStore } from '../../store/chatStore';
import { FileNode } from '../../types';

type SidebarTab = 'files' | 'sessions';

export function Sidebar() {
  const [activeTab, setActiveTab] = useState<SidebarTab>('files');

  return (
    <div className="h-full flex flex-col bg-bee-darker">
      {/* Tab buttons */}
      <div className="flex border-b border-panel-border">
        <TabButton
          active={activeTab === 'files'}
          onClick={() => setActiveTab('files')}
          icon={<FolderTree size={16} />}
          label="Files"
        />
        <TabButton
          active={activeTab === 'sessions'}
          onClick={() => setActiveTab('sessions')}
          icon={<History size={16} />}
          label="Sessions"
        />
      </div>

      {/* Tab content */}
      <div className="flex-1 overflow-hidden">
        <AnimatePresence mode="wait">
          {activeTab === 'files' ? (
            <motion.div
              key="files"
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
              className="h-full"
            >
              <FileTreePanel />
            </motion.div>
          ) : (
            <motion.div
              key="sessions"
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 20 }}
              className="h-full"
            >
              <SessionsPanel />
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}

function TabButton({ active, onClick, icon, label }: {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  label: string;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex-1 flex items-center justify-center gap-2 py-3 text-sm transition-colors ${
        active
          ? 'text-bee-yellow border-b-2 border-bee-yellow bg-bee-light/30'
          : 'text-gray-400 hover:text-white hover:bg-bee-light/20'
      }`}
    >
      {icon}
      <span>{label}</span>
    </button>
  );
}

function FileTreePanel() {
  const { files, expandedDirs, toggleDir, selectedFile, selectFile, isLoading } = useFileTreeStore();
  const { projectPath } = useSettingsStore();

  if (!projectPath) {
    return (
      <div className="p-4 text-center text-gray-500">
        <FolderTree size={48} className="mx-auto mb-3 opacity-50" />
        <p className="text-sm">No project selected</p>
        <p className="text-xs mt-1">Open settings to select a project folder</p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="p-4 text-center text-gray-500">
        <div className="animate-spin w-8 h-8 border-2 border-bee-yellow border-t-transparent rounded-full mx-auto mb-3" />
        <p className="text-sm">Loading files...</p>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto p-2">
      {files.map((node) => (
        <FileTreeNode
          key={node.path}
          node={node}
          level={0}
          expandedDirs={expandedDirs}
          selectedFile={selectedFile}
          onToggle={toggleDir}
          onSelect={selectFile}
        />
      ))}
    </div>
  );
}

function FileTreeNode({ node, level, expandedDirs, selectedFile, onToggle, onSelect }: {
  node: FileNode;
  level: number;
  expandedDirs: Set<string>;
  selectedFile: string | null;
  onToggle: (path: string) => void;
  onSelect: (path: string) => void;
}) {
  const isExpanded = expandedDirs.has(node.path);
  const isSelected = selectedFile === node.path;

  const getFileIcon = () => {
    if (node.isDir) {
      return isExpanded ? '📂' : '📁';
    }
    const ext = node.extension?.toLowerCase();
    switch (ext) {
      case 'rs': return '🦀';
      case 'ts':
      case 'tsx': return '💠';
      case 'js':
      case 'jsx': return '💛';
      case 'py': return '🐍';
      case 'md': return '📝';
      case 'json': return '📋';
      case 'toml':
      case 'yaml':
      case 'yml': return '⚙️';
      case 'css':
      case 'scss': return '🎨';
      case 'html': return '🌐';
      default: return '📄';
    }
  };

  return (
    <div>
      <div
        onClick={() => node.isDir ? onToggle(node.path) : onSelect(node.path)}
        className={`flex items-center gap-2 py-1 px-2 rounded cursor-pointer text-sm transition-colors ${
          isSelected
            ? 'bg-bee-yellow/20 text-bee-yellow'
            : 'hover:bg-bee-light text-gray-300 hover:text-white'
        }`}
        style={{ paddingLeft: `${level * 16 + 8}px` }}
      >
        {node.isDir && (
          <span className="text-gray-500">
            {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
          </span>
        )}
        <span>{getFileIcon()}</span>
        <span className="truncate">{node.name}</span>
      </div>

      <AnimatePresence>
        {node.isDir && isExpanded && node.children && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.15 }}
          >
            {node.children.map((child) => (
              <FileTreeNode
                key={child.path}
                node={child}
                level={level + 1}
                expandedDirs={expandedDirs}
                selectedFile={selectedFile}
                onToggle={onToggle}
                onSelect={onSelect}
              />
            ))}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function SessionsPanel() {
  const { sessions, loadSessions, loadSession, deleteSession, exportSession, isLoading } = useSessionStore();
  const { loadMessages } = useChatStore();

  const handleLoadSession = async (id: string) => {
    try {
      const session = await loadSession(id);
      loadMessages(session.messages.map(m => ({
        id: m.id,
        role: m.role as 'user' | 'assistant' | 'system',
        content: m.content,
        timestamp: m.timestamp,
        toolCalls: [],
      })));
    } catch (error) {
      console.error('Failed to load session:', error);
    }
  };

  const handleExport = async (id: string) => {
    try {
      const markdown = await exportSession(id, 'markdown');
      // Copy to clipboard
      await navigator.clipboard.writeText(markdown);
    } catch (error) {
      console.error('Failed to export session:', error);
    }
  };

  const handleDelete = async (id: string) => {
    if (confirm('Are you sure you want to delete this session?')) {
      try {
        await deleteSession(id);
      } catch (error) {
        console.error('Failed to delete session:', error);
      }
    }
  };

  // Load sessions on mount
  useState(() => {
    loadSessions();
  });

  if (isLoading) {
    return (
      <div className="p-4 text-center text-gray-500">
        <div className="animate-spin w-8 h-8 border-2 border-bee-yellow border-t-transparent rounded-full mx-auto mb-3" />
        <p className="text-sm">Loading sessions...</p>
      </div>
    );
  }

  if (sessions.length === 0) {
    return (
      <div className="p-4 text-center text-gray-500">
        <History size={48} className="mx-auto mb-3 opacity-50" />
        <p className="text-sm">No saved sessions</p>
        <p className="text-xs mt-1">Use /save to save your chat</p>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto p-2">
      {sessions.map((session) => (
        <div
          key={session.id}
          className="p-3 rounded-lg bg-bee-light/30 hover:bg-bee-light/50 transition-colors mb-2 group"
        >
          <div
            onClick={() => handleLoadSession(session.id)}
            className="cursor-pointer"
          >
            <div className="font-medium text-sm text-white truncate">{session.name}</div>
            <div className="text-xs text-gray-500 mt-1">
              {new Date(session.updatedAt).toLocaleDateString()} • {session.messageCount} messages
            </div>
          </div>
          <div className="flex gap-2 mt-2 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              onClick={() => handleExport(session.id)}
              className="p-1 rounded hover:bg-bee-light text-gray-400 hover:text-white"
              title="Export to Markdown"
            >
              <Download size={14} />
            </button>
            <button
              onClick={() => handleDelete(session.id)}
              className="p-1 rounded hover:bg-red-900/50 text-gray-400 hover:text-red-400"
              title="Delete session"
            >
              <Trash2 size={14} />
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
