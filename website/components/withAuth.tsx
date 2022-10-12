import { useRouter } from "next/router";
import React, { useEffect } from "react";
import { userStore } from "store/user";

const LoadingComponent = (props: any) => {
  const Component = (props: any) => {
    <div {...props}></div>;
  };
  Component.displayName = `withAuth(LoadingComponent)`;
};

const withAuth = (auth_level: "user" | "admin") => (Component: React.FC) => {
  const NewComponent = (props: any) => {
    const { loggedIn, data: userData, isLoading } = userStore();
    const router = useRouter();
    useEffect(() => {
      if (!isLoading) {
        if (!loggedIn || !userData) {
          router.push("/");
        }

        if (auth_level === "admin" && !userData?.is_admin) {
          router.push("/");
        }
      }
    }, [router, isLoading, loggedIn, userData]);

    if (isLoading) return LoadingComponent(props);
    return <Component {...props} />;
  };

  NewComponent.displayName = `withAuth(${
    Component.displayName || Component.name || "Component"
  })`;
  return NewComponent;
};

export default withAuth;
