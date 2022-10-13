import withAuth from "components/withAuth";
import Link from "next/link";
import React, { useEffect, useState } from "react";
import useWebSocket from "react-use-websocket";
import { ServerWithStatus } from "types";

type WebsocketMessage = {
  data: any;
  action: String;
};

const parseWsMessage = (
  event: MessageEvent<any>
): WebsocketMessage | undefined => {
  let parsedMsg = JSON.parse(event.data);
  if (!parsedMsg.action || !parsedMsg.data) {
    return undefined;
  }
  parsedMsg.data = JSON.parse(parsedMsg.data);
  return parsedMsg;
};

export const Dashboard = () => {
  const [servers, setServers] = useState<ServerWithStatus[]>([]);
  const onWsMessage = (event: MessageEvent<any>) => {
    let msg = parseWsMessage(event);
    if (msg === undefined) return;

    switch (msg.action) {
      case "response_get_servers":
        setServers(msg.data);
        break;
    }
  };

  const { sendMessage, lastMessage, lastJsonMessage, readyState } =
    useWebSocket(
      process.env.WS_URL + "/user" + "?token=" + localStorage.getItem("token"),
      {
        onMessage: onWsMessage,
      }
    );

  useEffect(() => {
    if (readyState == 1) {
      sendMessage("admin_get_servers");
    }
  }, [readyState, sendMessage]);

  return (
    <div>
      <Link href="/">Home</Link>
      <div>
        {servers.map((server) => (
          <div key={server.id}>
            <p>
              Server id: {server.id} - Server IP: {server.ip} - Online:{" "}
              {server.online ? "Yes" : "No"}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
};

export default withAuth("admin")(Dashboard);
