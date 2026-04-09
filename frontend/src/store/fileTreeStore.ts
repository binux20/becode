import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { FileNode } from '../types';

interface FileTreeState {
  files: FileNode[];
  selectedFile: string | null;
  expandedDirs: Set<string>;
  isLoading: boolean;
  error: string | null;

  // Actions
  loadFileTree: (path: string) => Promise<void>;
  refreshTree: () => Promise<void>;
  toggleDir: (path: string) => void;
  expandDir: (path: string) => void;
  collapseDir: (path: string) => void;
  selectFile: (path: string | null) => void;
  readFile: (path: string) => Promise<string>;
  getFilePreview: (path: string, lines?: number) => Promise<string>;
}

export const useFileTreeStore = create<FileTreeState>((set, get) => ({
  files: [],
  selectedFile: null,
  expandedDirs: new Set(),
  isLoading: false,
  error: null,

  loadFileTree: async (path) => {
    set({ isLoading: true, error: null });
    try {
      const files = await invoke<FileNode[]>('load_file_tree', { path, maxDepth: 3 });
      set({ files, isLoading: false });
    } catch (error) {
      console.error('Failed to load file tree:', error);
      set({ files: [], isLoading: false, error: String(error) });
    }
  },

  refreshTree: async () => {
    // Re-load the current tree
    const { files } = get();
    if (files.length > 0) {
      // Get root path from first file's parent
      const firstPath = files[0]?.path;
      if (firstPath) {
        const rootPath = firstPath.substring(0, firstPath.lastIndexOf('\\') || firstPath.lastIndexOf('/'));
        if (rootPath) {
          await get().loadFileTree(rootPath);
        }
      }
    }
  },

  toggleDir: (path) => {
    set((state) => {
      const newExpanded = new Set(state.expandedDirs);
      if (newExpanded.has(path)) {
        newExpanded.delete(path);
      } else {
        newExpanded.add(path);
      }
      return { expandedDirs: newExpanded };
    });
  },

  expandDir: (path) => {
    set((state) => {
      const newExpanded = new Set(state.expandedDirs);
      newExpanded.add(path);
      return { expandedDirs: newExpanded };
    });
  },

  collapseDir: (path) => {
    set((state) => {
      const newExpanded = new Set(state.expandedDirs);
      newExpanded.delete(path);
      return { expandedDirs: newExpanded };
    });
  },

  selectFile: (path) => {
    set({ selectedFile: path });
  },

  readFile: async (path) => {
    try {
      return await invoke<string>('read_file', { path });
    } catch (error) {
      console.error('Failed to read file:', error);
      throw error;
    }
  },

  getFilePreview: async (path, lines = 50) => {
    try {
      return await invoke<string>('get_file_preview', { path, lines });
    } catch (error) {
      console.error('Failed to get file preview:', error);
      throw error;
    }
  },
}));
