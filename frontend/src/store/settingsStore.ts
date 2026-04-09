import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { AppConfig, ProviderInfo, ModelInfo, SubAgentSettings } from '../types';

interface SettingsState {
  config: AppConfig | null;
  providers: ProviderInfo[];
  models: ModelInfo[];
  currentProvider: string;
  currentModel: string;
  projectPath: string;
  isLoading: boolean;
  error: string | null;

  // Actions
  loadConfig: () => Promise<void>;
  saveConfig: (config: Partial<AppConfig>) => Promise<void>;
  setProvider: (provider: string) => void;
  setModel: (model: string) => void;
  setProjectPath: (path: string) => void;
  loadProviders: () => Promise<void>;
  loadModels: (provider: string) => Promise<void>;
  setApiKey: (provider: string, key: string) => Promise<void>;
  getApiKey: (provider: string) => Promise<string | null>;
  updateSubAgents: (settings: Partial<SubAgentSettings>) => Promise<void>;
}

const defaultConfig: AppConfig = {
  defaultProvider: 'anthropic',
  defaultModel: 'claude-sonnet-4-20250514',
  permission: 'workspace-write',
  theme: 'dark',
  subAgents: {
    enabled: true,
    autoCompact: true,
    autoCompactThreshold: 80,
    useExplorer: true,
    usePlanner: true,
    useReviewer: false,
  },
  providers: {},
};

export const useSettingsStore = create<SettingsState>((set, get) => ({
  config: null,
  providers: [],
  models: [],
  currentProvider: 'anthropic',
  currentModel: 'claude-sonnet-4-20250514',
  projectPath: '',
  isLoading: false,
  error: null,

  loadConfig: async () => {
    set({ isLoading: true, error: null });
    try {
      const config = await invoke<AppConfig>('get_config');
      set({
        config,
        currentProvider: config.defaultProvider,
        currentModel: config.defaultModel || '',
        projectPath: config.projectDir || '',
        isLoading: false,
      });
    } catch (error) {
      console.error('Failed to load config:', error);
      set({
        config: defaultConfig,
        currentProvider: defaultConfig.defaultProvider,
        currentModel: defaultConfig.defaultModel || '',
        isLoading: false,
        error: String(error),
      });
    }
  },

  saveConfig: async (updates) => {
    const { config } = get();
    const newConfig = { ...config, ...updates } as AppConfig;
    try {
      await invoke('save_config', { config: newConfig });
      set({ config: newConfig });
    } catch (error) {
      console.error('Failed to save config:', error);
      set({ error: String(error) });
    }
  },

  setProvider: (provider) => {
    set({ currentProvider: provider });
    get().loadModels(provider);
  },

  setModel: (model) => {
    set({ currentModel: model });
  },

  setProjectPath: (path) => {
    set({ projectPath: path });
    get().saveConfig({ projectDir: path });
  },

  loadProviders: async () => {
    try {
      const providers = await invoke<ProviderInfo[]>('list_providers');
      set({ providers });
    } catch (error) {
      console.error('Failed to load providers:', error);
    }
  },

  loadModels: async (provider) => {
    try {
      const models = await invoke<ModelInfo[]>('list_models', { provider });
      set({ models });
      if (models.length > 0 && !get().currentModel) {
        set({ currentModel: models[0].id });
      }
    } catch (error) {
      console.error('Failed to load models:', error);
      set({ models: [] });
    }
  },

  setApiKey: async (provider, key) => {
    try {
      await invoke('set_api_key', { provider, key });
    } catch (error) {
      console.error('Failed to set API key:', error);
      throw error;
    }
  },

  getApiKey: async (provider) => {
    try {
      return await invoke<string | null>('get_api_key', { provider });
    } catch (error) {
      console.error('Failed to get API key:', error);
      return null;
    }
  },

  updateSubAgents: async (settings) => {
    const { config } = get();
    if (!config) return;

    const newSubAgents = { ...config.subAgents, ...settings };
    await get().saveConfig({ subAgents: newSubAgents });
  },
}));
