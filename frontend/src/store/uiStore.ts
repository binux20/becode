import { create } from 'zustand';
import { PanelFocus, CommandPaletteItem } from '../types';

interface UIState {
  sidebarOpen: boolean;
  sidebarWidth: number;
  settingsOpen: boolean;
  commandPaletteOpen: boolean;
  focusedPanel: PanelFocus;
  commandPaletteItems: CommandPaletteItem[];
  theme: 'dark' | 'light' | 'bee-yellow';

  // Actions
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;
  setSidebarWidth: (width: number) => void;
  openSettings: () => void;
  closeSettings: () => void;
  toggleSettings: () => void;
  openCommandPalette: () => void;
  closeCommandPalette: () => void;
  toggleCommandPalette: () => void;
  setFocusedPanel: (panel: PanelFocus) => void;
  setTheme: (theme: UIState['theme']) => void;
  registerCommand: (item: CommandPaletteItem) => void;
  unregisterCommand: (id: string) => void;
}

export const useUIStore = create<UIState>((set, get) => ({
  sidebarOpen: true,
  sidebarWidth: 280,
  settingsOpen: false,
  commandPaletteOpen: false,
  focusedPanel: 'input',
  commandPaletteItems: [],
  theme: 'dark',

  toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),

  setSidebarOpen: (open) => set({ sidebarOpen: open }),

  setSidebarWidth: (width) => set({ sidebarWidth: Math.max(200, Math.min(500, width)) }),

  openSettings: () => set({ settingsOpen: true, focusedPanel: 'settings' }),

  closeSettings: () => set({ settingsOpen: false, focusedPanel: 'input' }),

  toggleSettings: () => set((state) => ({
    settingsOpen: !state.settingsOpen,
    focusedPanel: state.settingsOpen ? 'input' : 'settings',
  })),

  openCommandPalette: () => set({ commandPaletteOpen: true }),

  closeCommandPalette: () => set({ commandPaletteOpen: false }),

  toggleCommandPalette: () => set((state) => ({ commandPaletteOpen: !state.commandPaletteOpen })),

  setFocusedPanel: (panel) => set({ focusedPanel: panel }),

  setTheme: (theme) => set({ theme }),

  registerCommand: (item) => set((state) => ({
    commandPaletteItems: [...state.commandPaletteItems.filter((i) => i.id !== item.id), item],
  })),

  unregisterCommand: (id) => set((state) => ({
    commandPaletteItems: state.commandPaletteItems.filter((i) => i.id !== id),
  })),
}));
