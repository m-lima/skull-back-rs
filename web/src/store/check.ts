import { StoreStatus } from "./model";

export namespace check {
  export const error = (...states: StoreStatus[]) => {
    for (const state of states) {
      if (!!state.error) {
        return state.error;
      }
    }
  }

  export const pending = (...states: StoreStatus[]) => {
    for (const state of states) {
      if (state.pending) {
        return true;
      }
    }
    return false;
  }
}

