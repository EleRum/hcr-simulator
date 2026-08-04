import { create } from 'zustand';

interface WorkbenchUiState {
  leftPanelOpen: boolean;
  rightPanelOpen: boolean;
  logOpen: boolean;
  showTarget: boolean;
  realisticHead: boolean;
  toggleLeftPanel: () => void;
  toggleRightPanel: () => void;
  toggleLog: () => void;
  toggleTarget: () => void;
  toggleRealisticHead: () => void;
}

export const useWorkbenchStore = create<WorkbenchUiState>((set) => ({
  leftPanelOpen: true,
  rightPanelOpen: true,
  logOpen: false,
  showTarget: true,
  realisticHead: false,
  toggleLeftPanel: () =>
    set((state) => ({ leftPanelOpen: !state.leftPanelOpen })),
  toggleRightPanel: () =>
    set((state) => ({ rightPanelOpen: !state.rightPanelOpen })),
  toggleLog: () => set((state) => ({ logOpen: !state.logOpen })),
  toggleTarget: () =>
    set((state) => ({ showTarget: !state.showTarget })),
  toggleRealisticHead: () =>
    set((state) => ({ realisticHead: !state.realisticHead })),
}));
