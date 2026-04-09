// Chat types
export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  toolCalls?: ToolCallInfo[];
}

export interface ToolCallInfo {
  id: string;
  tool: string;
  args: Record<string, unknown>;
  status: 'pending' | 'running' | 'success' | 'error';
  output?: string;
  error?: string;
  durationMs?: number;
}

export interface StreamChunk {
  chunkType: 'text' | 'tool_start' | 'tool_output' | 'tool_error';
  content?: string;
  toolCall?: ToolCallInfo;
}

// Settings types
export interface AppConfig {
  defaultProvider: string;
  defaultModel?: string;
  projectDir?: string;
  permission: 'read-only' | 'workspace-write' | 'danger';
  theme: 'dark' | 'light' | 'bee-yellow';
  subAgents: SubAgentSettings;
  providers: Record<string, ProviderConfig>;
}

export interface SubAgentSettings {
  enabled: boolean;
  autoCompact: boolean;
  autoCompactThreshold: number;
  useExplorer: boolean;
  usePlanner: boolean;
  useReviewer: boolean;
}

export interface ProviderConfig {
  providerType?: string;
  baseUrl?: string;
  apiKey?: string;
  model?: string;
}

export interface ProviderInfo {
  id: string;
  name: string;
  description: string;
  supportsTools: boolean;
  supportsVision: boolean;
  supportsStreaming: boolean;
}

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  contextWindow: number;
}

// File types
export interface FileNode {
  name: string;
  path: string;
  isDir: boolean;
  children?: FileNode[];
  size?: number;
  extension?: string;
}

// Session types
export interface SessionMetadata {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  messageCount: number;
  projectPath?: string;
  provider?: string;
}

export interface Session {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  projectPath?: string;
  provider?: string;
  model?: string;
  messages: ChatMessage[];
}

// UI types
export type PanelFocus = 'chat' | 'input' | 'sidebar' | 'settings';

export interface CommandPaletteItem {
  id: string;
  label: string;
  description?: string;
  icon?: string;
  shortcut?: string;
  action: () => void;
}
