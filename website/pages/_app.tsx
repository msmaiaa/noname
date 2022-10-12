import "../styles/reset.css";
import "../styles/globals.css";
import type { AppProps } from "next/app";
import { ReactNode, useEffect } from "react";
import { ApiProvider } from "@reduxjs/toolkit/dist/query/react";
import { baseApi } from "api";
import Navbar from "components/Navbar";
import { userStore } from "store/user";

const Layout = ({ children }: { children: ReactNode }) => {
  return (
    <div>
      <Navbar />
      {children}
    </div>
  );
};

type CustomPageProps = {
  requiredRole?: "user" | "admin";
};

function MyApp({ Component, pageProps }: AppProps<CustomPageProps>) {
  const { loggedIn, setLoggedIn, setUser, setLoading, isLoading } = userStore();

  useEffect(() => {
    const localUser = localStorage.getItem("user_data");
    if (localUser && !loggedIn) {
      setUser(JSON.parse(localUser));
      setLoggedIn(true);
    }
    setLoading(false);
  }, []);

  if (isLoading) return <div></div>;
  return (
    <ApiProvider api={baseApi}>
      <Layout>
        <Component {...pageProps} />
      </Layout>
    </ApiProvider>
  );
}

export default MyApp;
