import { StoreStatus } from './model';

export const check = {
  error: (...states: StoreStatus[]) => {
    for (const state of states) {
      if (state.error) {
        return state.error;
      }
    }
  },

  pending: (...states: StoreStatus[]) => {
    for (const state of states) {
      if (state.pending) {
        return true;
      }
    }
    return false;
  },
} as const;
