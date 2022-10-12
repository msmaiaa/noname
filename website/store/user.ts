import { devtools } from "zustand/middleware";
import create from "zustand";
import { User } from "types";

type UserStore = {
  data?: User;
  loggedIn: boolean;
  isLoading: boolean;
  setUser: (data: User) => void;
  setLoading: (loading: boolean) => void;
  setLoggedIn: (logged: boolean) => void;
};

export const userStore = create(
  devtools<UserStore>((set) => ({
    data: undefined,
    loggedIn: false,
    isLoading: true,
    setLoading(loading) {
      set({ isLoading: loading });
    },
    setUser(data) {
      set(() => ({ data }));
    },
    setLoggedIn(loggedIn) {
      set(() => ({ loggedIn }));
    },
  }))
);
