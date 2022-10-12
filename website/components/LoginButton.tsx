import React, { useEffect } from "react";
import styled from "styled-components";
import { userStore } from "../store/user";

type LoginResponse = {
  token: string;
  personaname: string;
  avatar: string;
  is_admin: boolean;
};

const isLoginResponseData = (
  messageData: any
): messageData is LoginResponse => {
  return (
    messageData.token !== undefined &&
      messageData.personaname !== undefined &&
      messageData.avatar !== undefined,
    messageData.is_admin !== undefined
  );
};

const $SteamButton = styled.img`
  &:hover {
    cursor: pointer;
  }
`;

const LoginButton = (props: React.ComponentPropsWithoutRef<"img">) => {
  const { setUser, setLoggedIn } = userStore();

  const onLogin = () => {
    const popup = window.open(
      process.env.API_URL + "/auth/login",
      "_blank",
      "width=500,height=500"
    );
    popup?.focus();
  };

  useEffect(() => {
    window.addEventListener("message", (event) => {
      if (event.origin !== process.env.API_URL?.replace("/api", "")) return;
      if (!isLoginResponseData(event.data)) return;
      const { token, personaname, avatar, is_admin } = event.data;
      setUser({
        avatar: avatar,
        personaname: personaname,
        is_admin: is_admin,
      });
      localStorage.setItem("token", token);
      localStorage.setItem(
        "user_data",
        JSON.stringify({
          avatar,
          personaname,
          is_admin,
        })
      );
      setLoggedIn(true);
    });
    return () => {
      window.removeEventListener("message", () => {});
    };
  }, [setLoggedIn, setUser]);

  return (
    <$SteamButton
      src="https://community.cloudflare.steamstatic.com/public/images/signinthroughsteam/sits_01.png"
      alt="login button"
      onClick={onLogin}
      {...props}
    />
  );
};

export default LoginButton;
