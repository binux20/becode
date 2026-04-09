import { create } from 'zustand';
import { ChatMessage, ToolCallInfo } from '../types';

interface ChatState {
  messages: ChatMessage[];
  isThinking: boolean;
  streamingText: string;
  status: 'ready' | 'thinking' | 'compacting' | 'cancelled' | 'error';
  error: string | null;

  // Actions
  addMessage: (message: ChatMessage) => void;
  updateLastMessage: (updates: Partial<ChatMessage>) => void;
  appendToLastMessage: (text: string) => void;
  setThinking: (thinking: boolean) => void;
  setStatus: (status: ChatState['status']) => void;
  setError: (error: string | null) => void;
  setStreamingText: (text: string) => void;
  appendStreamingText: (text: string) => void;
  clearStreamingText: () => void;
  addToolCall: (messageId: string, toolCall: ToolCallInfo) => void;
  updateToolCall: (messageId: string, toolCallId: string, updates: Partial<ToolCallInfo>) => void;
  clearMessages: () => void;
  loadMessages: (messages: ChatMessage[]) => void;
  compactMessages: (summary: string, keepLast: number) => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  messages: [],
  isThinking: false,
  streamingText: '',
  status: 'ready',
  error: null,

  addMessage: (message) => set((state) => ({
    messages: [...state.messages, message],
  })),

  updateLastMessage: (updates) => set((state) => {
    const messages = [...state.messages];
    if (messages.length > 0) {
      messages[messages.length - 1] = { ...messages[messages.length - 1], ...updates };
    }
    return { messages };
  }),

  appendToLastMessage: (text) => set((state) => {
    const messages = [...state.messages];
    if (messages.length > 0) {
      messages[messages.length - 1] = {
        ...messages[messages.length - 1],
        content: messages[messages.length - 1].content + text,
      };
    }
    return { messages };
  }),

  setThinking: (thinking) => set({
    isThinking: thinking,
    status: thinking ? 'thinking' : 'ready',
  }),

  setStatus: (status) => set({ status }),

  setError: (error) => set({ error, status: error ? 'error' : 'ready' }),

  setStreamingText: (text) => set({ streamingText: text }),

  appendStreamingText: (text) => set((state) => ({
    streamingText: state.streamingText + text,
  })),

  clearStreamingText: () => set({ streamingText: '' }),

  addToolCall: (messageId, toolCall) => set((state) => {
    const messages = state.messages.map((msg) => {
      if (msg.id === messageId) {
        return {
          ...msg,
          toolCalls: [...(msg.toolCalls || []), toolCall],
        };
      }
      return msg;
    });
    return { messages };
  }),

  updateToolCall: (messageId, toolCallId, updates) => set((state) => {
    const messages = state.messages.map((msg) => {
      if (msg.id === messageId && msg.toolCalls) {
        return {
          ...msg,
          toolCalls: msg.toolCalls.map((tc) =>
            tc.id === toolCallId ? { ...tc, ...updates } : tc
          ),
        };
      }
      return msg;
    });
    return { messages };
  }),

  clearMessages: () => set({ messages: [], streamingText: '', error: null }),

  loadMessages: (messages) => set({ messages, streamingText: '', error: null }),

  compactMessages: (summary, keepLast) => set((state) => {
    const { messages } = state;
    if (messages.length <= keepLast) return state;

    const toKeep = messages.slice(-keepLast);
    const summaryMessage: ChatMessage = {
      id: `summary-${Date.now()}`,
      role: 'system',
      content: summary,
      timestamp: new Date().toISOString(),
    };

    return { messages: [summaryMessage, ...toKeep] };
  }),
}));
