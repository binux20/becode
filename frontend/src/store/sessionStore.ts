import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { SessionMetadata, Session, ChatMessage } from '../types';

interface SessionState {
  sessions: SessionMetadata[];
  currentSession: Session | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  loadSessions: () => Promise<void>;
  loadSession: (id: string) => Promise<Session>;
  saveSession: (name: string, messages: ChatMessage[], projectPath?: string, provider?: string, model?: string) => Promise<string>;
  deleteSession: (id: string) => Promise<void>;
  exportSession: (id: string, format: 'markdown' | 'json') => Promise<string>;
  setCurrentSession: (session: Session | null) => void;
}

export const useSessionStore = create<SessionState>((set, get) => ({
  sessions: [],
  currentSession: null,
  isLoading: false,
  error: null,

  loadSessions: async () => {
    set({ isLoading: true, error: null });
    try {
      const sessions = await invoke<SessionMetadata[]>('list_sessions');
      set({ sessions, isLoading: false });
    } catch (error) {
      console.error('Failed to load sessions:', error);
      set({ sessions: [], isLoading: false, error: String(error) });
    }
  },

  loadSession: async (id) => {
    set({ isLoading: true, error: null });
    try {
      const session = await invoke<Session>('load_session', { id });
      set({ currentSession: session, isLoading: false });
      return session;
    } catch (error) {
      console.error('Failed to load session:', error);
      set({ isLoading: false, error: String(error) });
      throw error;
    }
  },

  saveSession: async (name, messages, projectPath, provider, model) => {
    try {
      const sessionMessages = messages.map((msg) => ({
        id: msg.id,
        role: msg.role,
        content: msg.content,
        timestamp: msg.timestamp,
        tool_calls: msg.toolCalls || [],
      }));

      const id = await invoke<string>('save_session', {
        name,
        messages: sessionMessages,
        projectPath,
        provider,
        model,
      });

      // Refresh sessions list
      await get().loadSessions();

      return id;
    } catch (error) {
      console.error('Failed to save session:', error);
      throw error;
    }
  },

  deleteSession: async (id) => {
    try {
      await invoke('delete_session', { id });
      set((state) => ({
        sessions: state.sessions.filter((s) => s.id !== id),
        currentSession: state.currentSession?.id === id ? null : state.currentSession,
      }));
    } catch (error) {
      console.error('Failed to delete session:', error);
      throw error;
    }
  },

  exportSession: async (id, format) => {
    try {
      return await invoke<string>('export_session', { id, format });
    } catch (error) {
      console.error('Failed to export session:', error);
      throw error;
    }
  },

  setCurrentSession: (session) => set({ currentSession: session }),
}));
