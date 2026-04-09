import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { X, Key, Palette, Bot, FolderOpen, Shield } from 'lucide-react';
import { useSettingsStore } from '../../store/settingsStore';
import { useFileTreeStore } from '../../store/fileTreeStore';
import { invoke } from '@tauri-apps/api/core';

interface Props {
  onClose: () => void;
}

type SettingsTab = 'api-keys' | 'providers' | 'agents' | 'appearance' | 'project';

export function SettingsModal({ onClose }: Props) {
  const [activeTab, setActiveTab] = useState<SettingsTab>('api-keys');

  // Close on Escape
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onClose]);

  const tabs: { id: SettingsTab; label: string; icon: React.ReactNode }[] = [
    { id: 'api-keys', label: 'API Keys', icon: <Key size={16} /> },
    { id: 'agents', label: 'Sub-Agents', icon: <Bot size={16} /> },
    { id: 'project', label: 'Project', icon: <FolderOpen size={16} /> },
    { id: 'appearance', label: 'Appearance', icon: <Palette size={16} /> },
  ];

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.95 }}
        transition={{ duration: 0.2 }}
        className="w-full max-w-2xl max-h-[80vh] bg-bee-darker border border-panel-border rounded-xl shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-panel-border">
          <h2 className="text-xl font-bold gradient-text">Settings</h2>
          <button
            onClick={onClose}
            className="p-2 rounded-lg hover:bg-bee-light text-gray-400 hover:text-white transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        {/* Content */}
        <div className="flex h-[60vh]">
          {/* Sidebar */}
          <div className="w-48 border-r border-panel-border p-2">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors ${
                  activeTab === tab.id
                    ? 'bg-bee-yellow/20 text-bee-yellow'
                    : 'text-gray-400 hover:bg-bee-light hover:text-white'
                }`}
              >
                {tab.icon}
                {tab.label}
              </button>
            ))}
          </div>

          {/* Main content */}
          <div className="flex-1 p-6 overflow-y-auto">
            {activeTab === 'api-keys' && <ApiKeysSettings />}
            {activeTab === 'agents' && <AgentsSettings />}
            {activeTab === 'project' && <ProjectSettings />}
            {activeTab === 'appearance' && <AppearanceSettings />}
          </div>
        </div>
      </motion.div>
    </motion.div>
  );
}

function ApiKeysSettings() {
  const { providers, setApiKey, getApiKey } = useSettingsStore();
  const [keys, setKeys] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState<string | null>(null);
  const [saved, setSaved] = useState<string | null>(null);

  // Load existing keys
  useEffect(() => {
    const loadKeys = async () => {
      const loadedKeys: Record<string, string> = {};
      for (const p of providers) {
        const key = await getApiKey(p.id);
        if (key) {
          loadedKeys[p.id] = '••••••••' + key.slice(-4);
        }
      }
      setKeys(loadedKeys);
    };
    loadKeys();
  }, [providers, getApiKey]);

  const handleSave = async (providerId: string, key: string) => {
    if (!key || key.startsWith('••••')) return;
    setSaving(providerId);
    try {
      await setApiKey(providerId, key);
      setSaved(providerId);
      setKeys((prev) => ({ ...prev, [providerId]: '••••••••' + key.slice(-4) }));
      setTimeout(() => setSaved(null), 2000);
    } catch (error) {
      console.error('Failed to save API key:', error);
    }
    setSaving(null);
  };

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold mb-2">API Keys</h3>
        <p className="text-sm text-gray-500">
          Enter your API keys for each provider. Keys are stored securely on your device.
        </p>
      </div>

      <div className="space-y-4">
        {providers.slice(0, 5).map((provider) => (
          <div key={provider.id} className="space-y-2">
            <label className="block text-sm font-medium text-gray-300">
              {provider.name}
            </label>
            <div className="flex gap-2">
              <input
                type="password"
                value={keys[provider.id] || ''}
                onChange={(e) => setKeys((prev) => ({ ...prev, [provider.id]: e.target.value }))}
                placeholder={`Enter ${provider.name} API key`}
                className="flex-1 input-bee"
              />
              <button
                onClick={() => handleSave(provider.id, keys[provider.id] || '')}
                disabled={saving === provider.id || !keys[provider.id] || keys[provider.id]?.startsWith('••••')}
                className="px-4 py-2 rounded-lg bg-bee-yellow/20 text-bee-yellow hover:bg-bee-yellow/30 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {saving === provider.id ? 'Saving...' : saved === provider.id ? 'Saved!' : 'Save'}
              </button>
            </div>
          </div>
        ))}
      </div>

      <div className="p-4 rounded-lg bg-bee-light/30 border border-panel-border">
        <div className="flex items-start gap-3">
          <Shield size={20} className="text-bee-yellow mt-0.5" />
          <div>
            <h4 className="font-medium text-white">Security Note</h4>
            <p className="text-sm text-gray-400 mt-1">
              API keys are stored locally in your system's secure storage. They are never sent to our servers.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

function AgentsSettings() {
  const { config, updateSubAgents } = useSettingsStore();
  const subAgents = config?.subAgents;

  if (!subAgents) return null;

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold mb-2">Sub-Agents</h3>
        <p className="text-sm text-gray-500">
          Configure AI sub-agents that help with complex tasks.
        </p>
      </div>

      <div className="space-y-4">
        {/* Main toggle */}
        <ToggleSetting
          label="Enable Sub-Agents"
          description="Use specialized AI agents for exploration, planning, and review"
          checked={subAgents.enabled}
          onChange={(enabled) => updateSubAgents({ enabled })}
        />

        <hr className="border-panel-border" />

        {/* Individual agents */}
        <ToggleSetting
          label="Auto-Compact Context"
          description="Automatically summarize old messages when context fills up"
          checked={subAgents.autoCompact}
          onChange={(autoCompact) => updateSubAgents({ autoCompact })}
          disabled={!subAgents.enabled}
        />

        <div className="space-y-2">
          <label className="block text-sm font-medium text-gray-300">
            Auto-Compact Threshold: {subAgents.autoCompactThreshold}%
          </label>
          <input
            type="range"
            min="50"
            max="95"
            value={subAgents.autoCompactThreshold}
            onChange={(e) => updateSubAgents({ autoCompactThreshold: parseInt(e.target.value) })}
            disabled={!subAgents.enabled || !subAgents.autoCompact}
            className="w-full accent-bee-yellow"
          />
        </div>

        <ToggleSetting
          label="Explorer Agent"
          description="Use for complex codebase exploration"
          checked={subAgents.useExplorer}
          onChange={(useExplorer) => updateSubAgents({ useExplorer })}
          disabled={!subAgents.enabled}
        />

        <ToggleSetting
          label="Planner Agent"
          description="Use for planning multi-step tasks"
          checked={subAgents.usePlanner}
          onChange={(usePlanner) => updateSubAgents({ usePlanner })}
          disabled={!subAgents.enabled}
        />

        <ToggleSetting
          label="Reviewer Agent"
          description="Automatically review generated code"
          checked={subAgents.useReviewer}
          onChange={(useReviewer) => updateSubAgents({ useReviewer })}
          disabled={!subAgents.enabled}
        />
      </div>
    </div>
  );
}

function ProjectSettings() {
  const { projectPath, setProjectPath } = useSettingsStore();
  const { loadFileTree } = useFileTreeStore();

  const handleSelectFolder = async () => {
    try {
      const path = await invoke<string | null>('select_project_folder');
      if (path) {
        setProjectPath(path);
        loadFileTree(path);
      }
    } catch (error) {
      console.error('Failed to select folder:', error);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold mb-2">Project</h3>
        <p className="text-sm text-gray-500">
          Configure the current project directory.
        </p>
      </div>

      <div className="space-y-4">
        <div className="space-y-2">
          <label className="block text-sm font-medium text-gray-300">
            Project Directory
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={projectPath}
              onChange={(e) => setProjectPath(e.target.value)}
              placeholder="Select a project folder"
              className="flex-1 input-bee"
              readOnly
            />
            <button
              onClick={handleSelectFolder}
              className="px-4 py-2 rounded-lg bg-bee-yellow/20 text-bee-yellow hover:bg-bee-yellow/30 transition-colors flex items-center gap-2"
            >
              <FolderOpen size={16} />
              Browse
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function AppearanceSettings() {
  const { config, saveConfig } = useSettingsStore();
  const theme = config?.theme || 'dark';

  const themes = [
    { id: 'dark', label: 'Dark', color: '#1E1E1E' },
    { id: 'light', label: 'Light', color: '#FFFFFF' },
    { id: 'bee-yellow', label: 'Bee Yellow', color: '#FFC800' },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold mb-2">Appearance</h3>
        <p className="text-sm text-gray-500">
          Customize the look and feel of BeCode.
        </p>
      </div>

      <div className="space-y-4">
        <label className="block text-sm font-medium text-gray-300">
          Theme
        </label>
        <div className="flex gap-3">
          {themes.map((t) => (
            <button
              key={t.id}
              onClick={() => saveConfig({ theme: t.id as any })}
              className={`flex items-center gap-2 px-4 py-3 rounded-lg border-2 transition-colors ${
                theme === t.id
                  ? 'border-bee-yellow bg-bee-yellow/10'
                  : 'border-panel-border hover:border-gray-600'
              }`}
            >
              <div
                className="w-6 h-6 rounded-full border border-gray-600"
                style={{ backgroundColor: t.color }}
              />
              <span className={theme === t.id ? 'text-bee-yellow' : 'text-gray-300'}>
                {t.label}
              </span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

function ToggleSetting({
  label,
  description,
  checked,
  onChange,
  disabled,
}: {
  label: string;
  description: string;
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <div className={`flex items-start justify-between ${disabled ? 'opacity-50' : ''}`}>
      <div>
        <div className="font-medium text-white">{label}</div>
        <div className="text-sm text-gray-500">{description}</div>
      </div>
      <button
        onClick={() => !disabled && onChange(!checked)}
        disabled={disabled}
        className={`relative w-12 h-6 rounded-full transition-colors ${
          checked ? 'bg-bee-yellow' : 'bg-gray-600'
        } ${disabled ? 'cursor-not-allowed' : 'cursor-pointer'}`}
      >
        <motion.div
          animate={{ x: checked ? 24 : 2 }}
          transition={{ duration: 0.15 }}
          className="absolute top-1 w-4 h-4 rounded-full bg-white shadow"
        />
      </button>
    </div>
  );
}
